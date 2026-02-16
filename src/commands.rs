//! Command parsing and execution.

use std::{
  env,
  fs::{self, File},
  io::Write,
  path::{Path, PathBuf},
  process::{ChildStdout, Command as OsCommand, Stdio},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use anyhow::Context;
use pathsearch::find_executable_in_path;
use strum::{Display, EnumIs, EnumTryAs};

mod parse;
mod targets;

pub use parse::{ParsedInvocation, needs_more_input};
pub use targets::{StderrTarget, StdoutTarget, open_stderr_writer, open_writer};

#[derive(Debug, PartialEq)]
pub enum RunResult {
  Continue,
  Exit(u8),
}

const BUILTIN_NAMES: &[&str] = &["cd", "echo", "exit", "type", "pwd", "history"];

fn is_builtin(name: &str) -> bool {
  BUILTIN_NAMES.contains(&name)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ControlOp {
  AndAnd,
  OrOr,
}

/// Split a line by `;`, respecting quotes. Returns segments (trimmed).
fn split_by_semicolon(s: &str) -> Vec<String> {
  let mut result = Vec::new();
  let mut start = 0;
  let mut in_double = false;
  let mut in_single = false;
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let c = bytes[i] as char;
    match c {
      '"' if !in_single => in_double = !in_double,
      '\'' if !in_double => in_single = !in_single,
      ';' if !in_double && !in_single => {
        result.push(s[start..i].trim().to_string());
        start = i + 1;
      }
      _ => {}
    }
    i += 1;
  }
  result.push(s[start..].trim().to_string());
  result
}

/// Split a segment by `&&` and `||`, respecting quotes. Returns (segments, operators).
/// Does not split on single `|` (pipeline).
fn split_by_and_or(s: &str) -> (Vec<String>, Vec<ControlOp>) {
  let mut segments = Vec::new();
  let mut ops = Vec::new();
  let mut start = 0;
  let mut in_double = false;
  let mut in_single = false;
  let bytes = s.as_bytes();
  let mut i = 0;
  while i < bytes.len() {
    let c = bytes[i] as char;
    match c {
      '"' if !in_single => in_double = !in_double,
      '\'' if !in_double => in_single = !in_single,
      '&' if !in_double && !in_single && i + 1 < bytes.len() && bytes[i + 1] == b'&' => {
        segments.push(s[start..i].trim().to_string());
        ops.push(ControlOp::AndAnd);
        i += 1;
        start = i + 1;
      }
      '|' if !in_double && !in_single && i + 1 < bytes.len() && bytes[i + 1] == b'|' => {
        segments.push(s[start..i].trim().to_string());
        ops.push(ControlOp::OrOr);
        i += 1;
        start = i + 1;
      }
      _ => {}
    }
    i += 1;
  }
  segments.push(s[start..].trim().to_string());
  (segments, ops)
}

/// Run a single part (may contain pipelines) and return (RunResult, exit_status).
fn run_one_part(
  raw: &str,
  shell_name: &str,
  history: &[String],
) -> anyhow::Result<(RunResult, u8)> {
  let raw = raw.trim();
  if raw.is_empty() {
    return Ok((RunResult::Continue, 0));
  }

  let tokens = match shlex::split(raw) {
    Some(t) if !t.is_empty() => t,
    _ => return Ok((RunResult::Continue, 0)),
  };

  if tokens.iter().any(|t| t == "|") {
    let status = run_pipeline_for_status(tokens, shell_name)?;
    return Ok((RunResult::Continue, status));
  }

  let Some(cmd) = Cmd::from_input(raw)? else {
    return Ok((RunResult::Continue, 0));
  };
  cmd.run_with_status(shell_name, history)
}

pub fn run_line(raw: &str, shell_name: &str, history: &[String]) -> anyhow::Result<RunResult> {
  let raw = raw.trim();
  if raw.is_empty() {
    return Ok(RunResult::Continue);
  }

  // Split by `;` first (lowest precedence)
  let semicolon_segments = split_by_semicolon(raw);

  let mut last_status = 0u8;
  for segment in semicolon_segments {
    let seg = segment.trim();
    if seg.is_empty() {
      continue;
    }
    // Split by `&&` and `||` within this segment
    let (parts, ops) = split_by_and_or(seg);
    if parts.is_empty() || parts.iter().all(|p| p.is_empty()) {
      continue;
    }

    for (idx, part) in parts.iter().enumerate() {
      let part = part.trim();
      if part.is_empty() {
        continue;
      }
      // Operator *before* this part (between previous and this)
      let op = if idx == 0 { None } else { ops.get(idx - 1) };
      let should_run = match op {
        None => true, // first part
        Some(ControlOp::AndAnd) => last_status == 0,
        Some(ControlOp::OrOr) => last_status != 0,
      };
      if !should_run {
        if op == Some(&ControlOp::AndAnd) {
          continue; // keep last_status
        }
        if op == Some(&ControlOp::OrOr) {
          break; // OrOr: we skipped because last_status was 0
        }
        continue;
      }

      let (result, status) = run_one_part(part, shell_name, history)?;
      last_status = status;

      if let RunResult::Exit(code) = result {
        return Ok(RunResult::Exit(code));
      }
    }
  }

  Ok(RunResult::Continue)
}

#[derive(Debug, PartialEq, EnumIs, EnumTryAs, Display)]
pub enum Cmd {
  Cd {
    cmd: Command,
    stderr: StderrTarget,
  },
  Echo {
    cmd: Command,
    stdout: StdoutTarget,
    stderr: StderrTarget,
  },
  Exit(u8),
  Type {
    cmd: Command,
    stdout: StdoutTarget,
    stderr: StderrTarget,
  },
  Exec {
    cmd: Command,
    stdout: StdoutTarget,
    stderr: StderrTarget,
  },
  Pwd {
    stdout: StdoutTarget,
    stderr: StderrTarget,
  },
  History {
    cmd: Command,
    stdout: StdoutTarget,
    stderr: StderrTarget,
  },
  Unknown {
    cmd: Command,
    stderr: StderrTarget,
  },
}

impl Cmd {
  pub fn from_input(raw: &str) -> anyhow::Result<Option<Self>> {
    let raw = raw.trim();
    if raw.is_empty() {
      return Ok(None);
    }
    let tokens = shlex::split(raw).unwrap_or_default();
    if tokens.is_empty() {
      return Ok(None);
    }
    let Some(inv) = ParsedInvocation::from_tokens(tokens) else {
      return Ok(None);
    };
    Ok(Some(Self::from_parts(
      &inv.cmd_name,
      inv.args,
      inv.stdout,
      inv.stderr,
    )))
  }

  pub fn from_parts(
    cmd_name: &str,
    args: Vec<String>,
    stdout: StdoutTarget,
    stderr: StderrTarget,
  ) -> Self {
    match cmd_name {
      "cd" => Cmd::Cd {
        cmd: Command::new(cmd_name, None, args),
        stderr,
      },
      "exit" => {
        let code = args.first().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        Cmd::Exit(code)
      }
      "echo" => Cmd::Echo {
        cmd: Command::new(cmd_name, None, args),
        stdout,
        stderr,
      },
      "history" => Cmd::History {
        cmd: Command::new(cmd_name, None, args),
        stdout,
        stderr,
      },
      "type" => Cmd::Type {
        cmd: Command::new(cmd_name, None, args),
        stdout,
        stderr,
      },
      "pwd" => Cmd::Pwd { stdout, stderr },
      _ => {
        if cmd_name.contains('/') {
          Cmd::Exec {
            cmd: Command::new(cmd_name, Some(cmd_name.to_string()), args),
            stdout,
            stderr,
          }
        } else if let Some(path_buf) = find_executable(cmd_name) {
          let path_str = path_buf
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| String::new());
          Cmd::Exec {
            cmd: Command::new(cmd_name, Some(path_str), args),
            stdout,
            stderr,
          }
        } else {
          Cmd::Unknown {
            cmd: Command::new(cmd_name, None, args),
            stderr,
          }
        }
      }
    }
  }

  /// Run the command and return (RunResult, exit_status). Exit status is used for `&&` / `||`.
  pub fn run_with_status(
    &self,
    shell_name: &str,
    history: &[String],
  ) -> anyhow::Result<(RunResult, u8)> {
    match self {
      Cmd::Echo {
        cmd,
        stdout,
        stderr: _,
      } => {
        let mut out = open_writer(stdout)?;
        echo_args(&cmd.args, &mut out)?;
        Ok((RunResult::Continue, 0))
      }
      Cmd::Exit(code) => Ok((RunResult::Exit(*code), *code)),
      Cmd::Type {
        cmd,
        stdout,
        stderr: _,
      } => {
        let mut out = open_writer(stdout)?;
        writeln!(out, "{}", resolve_types(&cmd.args))?;
        Ok((RunResult::Continue, 0))
      }
      Cmd::Exec {
        cmd,
        stdout,
        stderr,
      } => {
        let status = match cmd.run_with_stdio(stdout, stderr) {
          Ok(s) => s,
          Err(err) => {
            let mut err_out = open_stderr_writer(stderr)?;
            writeln!(err_out, "{shell_name}: {err}")?;
            127
          }
        };
        Ok((RunResult::Continue, status))
      }
      Cmd::Cd { cmd, stderr } => {
        let target = cmd.args.first().map(String::as_str).unwrap_or("");
        let status = match change_dir(target) {
          Ok(()) => 0,
          Err(err) => {
            let mut err_out = open_stderr_writer(stderr)?;
            writeln!(err_out, "{shell_name}: {err}")?;
            1
          }
        };
        Ok((RunResult::Continue, status))
      }
      Cmd::Pwd { stdout, stderr: _ } => {
        let mut out = open_writer(stdout)?;
        let dir = env::current_dir().context("get current directory")?;
        writeln!(out, "{}", dir.display())?;
        Ok((RunResult::Continue, 0))
      }
      Cmd::History {
        cmd,
        stdout,
        stderr: _,
      } => {
        let count = cmd.args.get(0).and_then(|s| s.parse::<usize>().ok());
        let total = history.len();
        let start = match count {
          Some(n) if n < total => total - n,
          Some(_) => 0,
          None => 0,
        };

        let mut out = open_writer(stdout)?;
        for (idx, entry) in history.iter().enumerate().skip(start) {
          writeln!(out, "  {:>4}  {entry}", idx + 1)?;
        }
        Ok((RunResult::Continue, 0))
      }
      Cmd::Unknown { cmd, stderr } => {
        let mut err_out = open_stderr_writer(stderr)?;
        writeln!(err_out, "{shell_name}: command not found: {}", cmd.name)?;
        Ok((RunResult::Continue, 127))
      }
    }
  }

  pub fn run(&self, shell_name: &str, history: &[String]) -> anyhow::Result<RunResult> {
    self.run_with_status(shell_name, history).map(|(r, _)| r)
  }
}

fn split_pipeline(tokens: Vec<String>) -> Option<Vec<Vec<String>>> {
  let mut segments = Vec::new();
  let mut current = Vec::new();

  for tok in tokens {
    if tok == "|" {
      if current.is_empty() {
        return None;
      }
      segments.push(std::mem::take(&mut current));
    } else {
      current.push(tok);
    }
  }

  if current.is_empty() {
    return None;
  }

  segments.push(current);
  Some(segments)
}

fn run_pipeline_for_status(tokens: Vec<String>, shell_name: &str) -> anyhow::Result<u8> {
  use anyhow::anyhow;

  let segments = match split_pipeline(tokens) {
    Some(segs) => segs,
    None => {
      eprintln!("{shell_name}: invalid pipeline");
      return Ok(127);
    }
  };

  let mut invocations = Vec::new();
  for seg in segments {
    let Some(inv) = ParsedInvocation::from_tokens(seg) else {
      eprintln!("{shell_name}: invalid command in pipeline");
      return Ok(127);
    };
    invocations.push(inv);
  }

  if invocations.is_empty() {
    return Ok(127);
  }

  let mut children = Vec::new();
  let mut prev_stdout: Option<ChildStdout> = None;

  for (idx, inv) in invocations.iter().enumerate() {
    let is_last = idx == invocations.len() - 1;

    let program_path = if inv.cmd_name.contains('/') {
      PathBuf::from(&inv.cmd_name)
    } else if let Some(p) = find_executable(&inv.cmd_name) {
      p
    } else {
      eprintln!("{shell_name}: command not found: {}", inv.cmd_name);
      return Ok(127);
    };

    let mut cmd = OsCommand::new(&program_path);
    cmd.args(&inv.args);

    if let Some(stdin) = prev_stdout.take() {
      cmd.stdin(Stdio::from(stdin));
    }

    if is_last {
      match &inv.stdout {
        StdoutTarget::Stdout => {}
        StdoutTarget::Overwrite(path) => {
          let file = File::create(path)?;
          cmd.stdout(Stdio::from(file));
        }
        StdoutTarget::Append(path) => {
          let file = File::options().append(true).create(true).open(path)?;
          cmd.stdout(Stdio::from(file));
        }
      }
    } else {
      cmd.stdout(Stdio::piped());
    }

    if is_last {
      match &inv.stderr {
        StderrTarget::Stderr => {}
        StderrTarget::Overwrite(path) => {
          let file = File::create(path)?;
          cmd.stderr(Stdio::from(file));
        }
        StderrTarget::Append(path) => {
          let file = File::options().append(true).create(true).open(path)?;
          cmd.stderr(Stdio::from(file));
        }
      }
    }

    let mut child = cmd
      .spawn()
      .with_context(|| format!("failed to spawn `{}`", inv.cmd_name))?;

    if !is_last {
      let child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture stdout for pipeline stage"))?;
      prev_stdout = Some(child_stdout);
    }

    children.push(child);
  }

  let mut last_status = 127u8;
  for (i, mut child) in children.into_iter().enumerate() {
    let exit_status = child
      .wait()
      .with_context(|| "failed to wait for pipeline stage")?;
    if i == invocations.len() - 1 {
      last_status = (exit_status.code().unwrap_or(1) & 0xFF) as u8;
    }
  }

  Ok(last_status)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
  pub name: String,
  pub path: Option<String>,
  pub args: Vec<String>,
}

impl Command {
  pub fn new(name: &str, path: Option<String>, args: Vec<String>) -> Self {
    Self {
      name: name.to_owned(),
      path,
      args,
    }
  }

  pub fn run(&self) -> anyhow::Result<()> {
    let program = self.path.as_deref().unwrap_or(&self.name);
    let mut child = std::process::Command::new(program)
      .args(&self.args)
      .spawn()
      .with_context(|| format!("failed to spawn `{program}`"))?;
    child
      .wait()
      .with_context(|| format!("failed to wait for `{program}`"))?;
    Ok(())
  }

  pub fn run_with_stdio(&self, stdout: &StdoutTarget, stderr: &StderrTarget) -> anyhow::Result<u8> {
    let program = self.path.as_deref().unwrap_or(&self.name);
    let mut command = std::process::Command::new(program);
    command.args(&self.args);

    match stdout {
      StdoutTarget::Stdout => {}
      StdoutTarget::Overwrite(path) => {
        let file = File::create(path)?;
        command.stdout(Stdio::from(file));
      }
      StdoutTarget::Append(path) => {
        let file = File::options().append(true).create(true).open(path)?;
        command.stdout(Stdio::from(file));
      }
    }

    match stderr {
      StderrTarget::Stderr => {}
      StderrTarget::Overwrite(path) => {
        let file = File::create(path)?;
        command.stderr(Stdio::from(file));
      }
      StderrTarget::Append(path) => {
        let file = File::options().append(true).create(true).open(path)?;
        command.stderr(Stdio::from(file));
      }
    }

    let mut child = command
      .spawn()
      .with_context(|| format!("failed to spawn `{program}`"))?;
    let status = child
      .wait()
      .with_context(|| format!("failed to wait for `{program}`"))?;
    let code = status.code().unwrap_or(1);
    Ok((code & 0xFF) as u8)
  }
}

pub fn resolve_types(args: &[String]) -> String {
  args
    .iter()
    .map(|name| {
      if is_builtin(name) {
        format!("{name} is a shell builtin")
      } else {
        match find_executable(name) {
          Some(path) => format!("{name} is {}", path.display()),
          None => format!("{name}: not found"),
        }
      }
    })
    .collect::<Vec<String>>()
    .join("\n")
}

pub fn find_executable(cmd: &str) -> Option<PathBuf> {
  find_executable_in_path(cmd)
}

/// Returns sorted, deduplicated command names (builtins + PATH executables) that start with `prefix`.
/// Used for tab completion; only the first word of the line should be completed.
pub fn complete_command(prefix: &str) -> Vec<String> {
  let mut names: Vec<String> = BUILTIN_NAMES
    .iter()
    .filter(|n| n.starts_with(prefix))
    .map(|s| (*s).to_string())
    .collect();

  let path_var = env::var("PATH").unwrap_or_default();
  for dir in env::split_paths(&path_var) {
    let Ok(entries) = fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        continue;
      };
      if !name.starts_with(prefix) {
        continue;
      }
      let meta = match entry.metadata() {
        Ok(m) => m,
        Err(_) => continue,
      };
      if meta.is_dir() {
        continue;
      }
      #[cfg(unix)]
      if meta.permissions().mode() & 0o111 == 0 {
        continue;
      }
      names.push(name.to_string());
    }
  }

  names.sort_unstable();
  names.dedup();
  names
}

pub fn change_dir(target: &str) -> anyhow::Result<()> {
  let new_path = if target.is_empty() || target == "~" {
    env::var("HOME").context("HOME not set")?
  } else {
    target.to_string()
  };

  let path = Path::new(&new_path);

  if !path.exists() {
    println!("cd: {target}: No such file or directory");
    return Ok(());
  }

  env::set_current_dir(path).with_context(|| format!("cd: {target}"))?;

  let updated_cwd = env::current_dir().context("get cwd after cd")?;
  unsafe {
    env::set_var("PWD", updated_cwd);
  }

  Ok(())
}

pub fn echo_args<W: Write>(args: &[String], out: &mut W) -> anyhow::Result<()> {
  writeln!(out, "{}", args.join(" "))?;
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_input_empty_returns_none() {
    assert!(matches!(Cmd::from_input("").unwrap(), None));
    assert!(matches!(Cmd::from_input("   ").unwrap(), None));
  }

  #[test]
  fn from_input_echo_preserves_whitespace() {
    let cmd = Cmd::from_input(r#"echo "hello   world""#).unwrap().unwrap();
    assert!(cmd.is_echo());
    let Cmd::Echo { cmd, .. } = cmd else {
      unreachable!()
    };
    assert_eq!(cmd.args, vec!["hello   world"]);
  }

  #[test]
  fn from_input_echo_multiple_args() {
    let cmd = Cmd::from_input("echo a b c").unwrap().unwrap();
    let Cmd::Echo { cmd, .. } = cmd else {
      unreachable!()
    };
    assert_eq!(cmd.args, vec!["a", "b", "c"]);
  }

  #[test]
  fn needs_more_input_unclosed_quotes() {
    assert!(needs_more_input(r#"echo "hello"#));
    assert!(needs_more_input("echo 'hello"));
    assert!(!needs_more_input(r#"echo "hello""#));
    assert!(!needs_more_input(""));
  }

  #[test]
  fn from_parts_exit_no_args_is_zero() {
    let cmd = Cmd::from_parts("exit", vec![], StdoutTarget::Stdout, StderrTarget::Stderr);
    assert!(matches!(cmd, Cmd::Exit(0)));
  }

  #[test]
  fn from_parts_exit_with_code() {
    let cmd = Cmd::from_parts(
      "exit",
      vec!["42".into()],
      StdoutTarget::Stdout,
      StderrTarget::Stderr,
    );
    assert!(matches!(cmd, Cmd::Exit(42)));
  }

  #[test]
  fn from_parts_pwd() {
    let cmd = Cmd::from_parts("pwd", vec![], StdoutTarget::Stdout, StderrTarget::Stderr);
    assert!(matches!(cmd, Cmd::Pwd { .. }));
  }

  #[test]
  fn from_parts_type_args() {
    let cmd = Cmd::from_parts(
      "type",
      vec!["cd".into(), "ls".into()],
      StdoutTarget::Stdout,
      StderrTarget::Stderr,
    );
    let Cmd::Type { cmd, .. } = cmd else {
      unreachable!()
    };
    assert_eq!(cmd.args, vec!["cd", "ls"]);
  }

  #[test]
  fn from_parts_cd_args() {
    let cmd = Cmd::from_parts(
      "cd",
      vec!["/tmp".into()],
      StdoutTarget::Stdout,
      StderrTarget::Stderr,
    );
    let Cmd::Cd { cmd, .. } = cmd else {
      unreachable!()
    };
    assert_eq!(cmd.args, vec!["/tmp"]);
  }

  #[test]
  fn is_builtin_known() {
    for name in BUILTIN_NAMES {
      assert!(is_builtin(name), "{name} should be builtin");
    }
  }

  #[test]
  fn is_builtin_unknown() {
    assert!(!is_builtin("ls"));
    assert!(!is_builtin(""));
  }
}
