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

const BUILTIN_NAMES: &[&str] = &["cd", "echo", "exit", "type", "pwd", "history", "ls"];

pub fn is_builtin(name: &str) -> bool {
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
  Ls {
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
      "ls" => Cmd::Ls {
        cmd: Command::new(cmd_name, None, args),
        stdout,
        stderr,
      },
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
      Cmd::Ls {
        cmd,
        stdout,
        stderr,
      } => {
        let status = match run_ls(&cmd.args, stdout) {
          Ok(()) => 0,
          Err(err) => {
            let mut err_out = open_stderr_writer(stderr)?;
            writeln!(err_out, "{shell_name}: ls: {err}")?;
            1
          }
        };
        Ok((RunResult::Continue, status))
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
  if cmd.is_empty() {
    return None;
  }
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

const COLOR_DIR: &str = "\x1b[38;5;208m"; // #fa912a (256-color for broad terminal support)
const COLOR_HIDDEN: &str = "\x1b[38;5;245m"; // grey
const COLOR_RESET: &str = "\x1b[0m";

fn terminal_width() -> usize {
  #[cfg(unix)]
  {
    unsafe {
      let mut ws: libc::winsize = std::mem::zeroed();
      if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
        return ws.ws_col as usize;
      }
    }
  }
  80
}

fn display_name(name: &str, is_dir: bool) -> String {
  if is_dir {
    format!("{name}/")
  } else {
    name.to_string()
  }
}

fn colorize_ls_entry(display: &str, is_dir: bool, is_hidden: bool) -> String {
  if is_dir && is_hidden {
    format!("{COLOR_HIDDEN}{display}{COLOR_RESET}")
  } else if is_dir {
    format!("{COLOR_DIR}{display}{COLOR_RESET}")
  } else if is_hidden {
    format!("{COLOR_HIDDEN}{display}{COLOR_RESET}")
  } else {
    display.to_string()
  }
}

/// Print entries in a column grid that fills top-to-bottom, left-to-right (like system `ls`).
fn print_columns<W: Write>(
  out: &mut W,
  entries: &[(String, bool)],
  term_width: usize,
) -> anyhow::Result<()> {
  if entries.is_empty() {
    return Ok(());
  }

  let displays: Vec<String> = entries
    .iter()
    .map(|(name, is_dir)| display_name(name, *is_dir))
    .collect();

  let col_gap = 14usize;
  let count = displays.len();

  // Try increasing number of columns until they no longer fit
  let mut best_ncols = 1usize;
  let mut best_col_widths: Vec<usize> = vec![0];

  for ncols in 1..=count {
    let nrows = (count + ncols - 1) / ncols;
    let mut col_widths = vec![0usize; ncols];

    for (i, d) in displays.iter().enumerate() {
      let col = i / nrows;
      col_widths[col] = col_widths[col].max(d.len());
    }

    let total: usize = col_widths.iter().sum::<usize>() + col_gap * ncols.saturating_sub(1);
    if total <= term_width {
      best_ncols = ncols;
      best_col_widths = col_widths;
    } else {
      break;
    }
  }

  let nrows = (count + best_ncols - 1) / best_ncols;

  for row in 0..nrows {
    let mut line = String::new();
    for col in 0..best_ncols {
      let idx = col * nrows + row;
      if idx >= count {
        break;
      }
      let (name, is_dir) = &entries[idx];
      let d = &displays[idx];
      let is_hidden = name.starts_with('.');
      let colored = colorize_ls_entry(d, *is_dir, is_hidden);

      if col + 1 < best_ncols && (col + 1) * nrows + row < count {
        let pad = best_col_widths[col] - d.len() + col_gap;
        line.push_str(&colored);
        line.extend(std::iter::repeat(' ').take(pad));
      } else {
        line.push_str(&colored);
      }
    }
    writeln!(out, "{line}")?;
  }

  Ok(())
}

fn run_ls(args: &[String], stdout_target: &StdoutTarget) -> anyhow::Result<()> {
  let mut show_all = false;
  let mut long_format = false;
  let mut paths: Vec<String> = Vec::new();

  for arg in args {
    if arg.starts_with('-') && !arg.starts_with("--") {
      for ch in arg[1..].chars() {
        match ch {
          'a' => show_all = true,
          'l' => long_format = true,
          _ => {}
        }
      }
    } else {
      paths.push(arg.clone());
    }
  }

  if paths.is_empty() {
    paths.push(".".to_string());
  }

  let multiple = paths.len() > 1;
  let mut out = open_writer(stdout_target)?;
  let tw = terminal_width();

  for (i, path_str) in paths.iter().enumerate() {
    let path = Path::new(path_str);
    if !path.exists() {
      writeln!(
        out,
        "ls: cannot access '{path_str}': No such file or directory"
      )?;
      continue;
    }

    if !path.is_dir() {
      let name = path.file_name().unwrap_or_default().to_string_lossy();
      writeln!(out, "{name}")?;
      continue;
    }

    if multiple {
      if i > 0 {
        writeln!(out)?;
      }
      writeln!(out, "{path_str}:")?;
    }

    let mut entries: Vec<(String, bool)> = Vec::new();
    if show_all {
      entries.push((".".to_string(), true));
      entries.push(("..".to_string(), true));
    }
    for entry in fs::read_dir(path)? {
      let entry = entry?;
      let name = entry.file_name().to_string_lossy().into_owned();
      let is_hidden = name.starts_with('.');
      if !show_all && is_hidden {
        continue;
      }
      let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
      entries.push((name, is_dir));
    }
    entries.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

    if long_format {
      for (name, is_dir) in &entries {
        let is_hidden = name.starts_with('.');
        let d = display_name(name, *is_dir);
        let colored = colorize_ls_entry(&d, *is_dir, is_hidden);
        writeln!(out, "{colored}")?;
      }
    } else {
      print_columns(&mut out, &entries, tw)?;
    }
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
  use std::fs;
  use std::io::Read;
  use std::path::PathBuf;

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

  #[test]
  fn split_by_semicolon_respects_quotes() {
    let input = r#"echo "a;b"; echo c; echo 'd;e'"#;
    let parts = split_by_semicolon(input);
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], r#"echo "a;b""#);
    assert_eq!(parts[1], "echo c");
    assert_eq!(parts[2], r#"echo 'd;e'"#);
  }

  #[test]
  fn split_by_and_or_respects_quotes() {
    let input = r#"echo "a && b" && echo c || echo 'd || e'"#;
    let (parts, ops) = split_by_and_or(input);
    assert_eq!(
      parts,
      vec![r#"echo "a && b""#, "echo c", r#"echo 'd || e'"#]
    );
    assert_eq!(ops, vec![ControlOp::AndAnd, ControlOp::OrOr]);
  }

  #[test]
  fn run_one_part_uses_builtin_status() {
    let (result, status) = run_one_part("echo ok", "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));
    assert_eq!(status, 0);

    let (result, status) = run_one_part("definitely-does-not-exist-xyz", "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));
    assert_eq!(status, 127);
  }

  #[test]
  fn run_one_part_pipeline_command_not_found_gives_127() {
    let (result, status) =
      run_one_part("no-such-cmd-abc | also-no-such-cmd-def", "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));
    assert_eq!(status, 127);
  }

  fn tmp_path(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("ase_test_{name}_{}", std::process::id()));
    p
  }

  #[test]
  fn control_and_and_runs_second_only_on_success() {
    let path = tmp_path("and_and");
    let line = format!(
      "echo first > {} && echo second >> {}",
      path.display(),
      path.display()
    );

    let result = run_line(&line, "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));

    let mut contents = String::new();
    let mut file = fs::File::open(&path).unwrap();
    file.read_to_string(&mut contents).unwrap();
    fs::remove_file(&path).ok();

    assert!(contents.contains("first"));
    assert!(contents.contains("second"));
  }

  #[test]
  fn control_and_and_skips_on_failure() {
    let path = tmp_path("and_and_skip");
    let line = format!(
      "no-such-cmd-xyz && echo should-not-run > {}",
      path.display()
    );

    let result = run_line(&line, "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));
    assert!(!path.exists());
  }

  #[test]
  fn control_or_or_runs_on_failure() {
    let path = tmp_path("or_or");
    let line = format!(
      "no-such-cmd-xyz || echo ran-after-failure > {}",
      path.display()
    );

    let result = run_line(&line, "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));

    let contents = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).ok();
    assert!(contents.contains("ran-after-failure"));
  }

  #[test]
  fn control_or_or_skips_on_success() {
    let path = tmp_path("or_or_skip");
    let line = format!("echo ok || echo should-not-run > {}", path.display());

    let result = run_line(&line, "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));
    assert!(!path.exists());
  }

  #[test]
  fn semicolon_always_runs_both() {
    let path = tmp_path("semicolon");
    let line = format!(
      "echo one > {}; echo two >> {}",
      path.display(),
      path.display()
    );

    let result = run_line(&line, "ase-test", &[]).unwrap();
    assert!(matches!(result, RunResult::Continue));

    let contents = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).ok();
    assert!(contents.contains("one"));
    assert!(contents.contains("two"));
  }

  #[test]
  fn history_builtin_respects_count_and_writes_to_file() {
    let path = tmp_path("history");
    let history = vec!["ls".to_string(), "echo a".to_string(), "echo b".to_string()];
    let line = format!("history 2 > {}", path.display());

    let result = run_line(&line, "ase-test", &history).unwrap();
    assert!(matches!(result, RunResult::Continue));

    let contents = fs::read_to_string(&path).unwrap();
    fs::remove_file(&path).ok();

    assert!(contents.contains("echo a"));
    assert!(contents.contains("echo b"));
    assert!(!contents.contains("ls"));
  }

  #[test]
  fn complete_command_includes_builtins_and_path_executables() {
    let names = complete_command("ec");
    assert!(names.contains(&"echo".to_string()));
  }
}
