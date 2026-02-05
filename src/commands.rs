//! Command parsing and execution.

use std::{
  env,
  fs::File,
  io::{self, Write},
  path::{Path, PathBuf},
  process::Stdio,
};

use anyhow::Context;
use pathsearch::find_executable_in_path;
use strum::{Display, EnumIs, EnumTryAs};

#[derive(Debug, PartialEq)]
pub enum RunResult {
  Continue,
  Exit(u8),
}

const BUILTIN_NAMES: &[&str] = &["cd", "echo", "exit", "type", "pwd"];

fn is_builtin(name: &str) -> bool {
  BUILTIN_NAMES.contains(&name)
}

#[derive(Debug, PartialEq, Clone)]
pub enum StdoutTarget {
  Stdout,
  File(PathBuf),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StderrTarget {
  Stderr,
  File(PathBuf),
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
  Unknown {
    cmd: Command,
    stderr: StderrTarget,
  },
}

struct ParsedInvocation {
  cmd_name: String,
  args: Vec<String>,
  stdout: StdoutTarget,
  stderr: StderrTarget,
}

impl ParsedInvocation {
  fn from_tokens(tokens: Vec<String>) -> Option<Self> {
    let mut iter = tokens.into_iter();
    let cmd_name = iter.next()?;
    let rest: Vec<String> = iter.collect();

    let mut stdout = StdoutTarget::Stdout;
    let mut stderr = StderrTarget::Stderr;
    let mut args = Vec::new();
    let mut i = 0;

    while i < rest.len() {
      match rest[i].as_str() {
        ">" | "1>" => {
          if i + 1 < rest.len() {
            stdout = StdoutTarget::File(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        "2>" => {
          if i + 1 < rest.len() {
            stderr = StderrTarget::File(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        _ => {
          args.push(rest[i].clone());
          i += 1;
        }
      }
    }

    if args.is_empty() && rest.is_empty() {
      return None;
    }

    Some(ParsedInvocation {
      cmd_name,
      args,
      stdout,
      stderr,
    })
  }
}

/// True when input has unclosed quote(s); caller should show continuation prompt and read more.
pub fn needs_more_input(raw: &str) -> bool {
  let r = raw.trim();
  !r.is_empty() && shlex::split(r).is_none()
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

  pub fn run(&self, shell_name: &str) -> anyhow::Result<RunResult> {
    match self {
      Cmd::Echo {
        cmd,
        stdout,
        stderr: _,
      } => {
        let mut out = open_writer(stdout)?;
        echo_args(&cmd.args, &mut out)?;
        Ok(RunResult::Continue)
      }
      Cmd::Exit(code) => Ok(RunResult::Exit(*code)),
      Cmd::Type {
        cmd,
        stdout,
        stderr: _,
      } => {
        let mut out = open_writer(stdout)?;
        writeln!(out, "{}", resolve_types(&cmd.args))?;
        Ok(RunResult::Continue)
      }
      Cmd::Exec {
        cmd,
        stdout,
        stderr,
      } => {
        if let Err(err) = cmd.run_with_stdio(stdout, stderr) {
          let mut err_out = open_stderr_writer(stderr)?;
          writeln!(err_out, "{shell_name}: {err}")?;
        }
        Ok(RunResult::Continue)
      }
      Cmd::Cd { cmd, stderr } => {
        let target = cmd.args.first().map(String::as_str).unwrap_or("");
        if let Err(err) = change_dir(target) {
          let mut err_out = open_stderr_writer(stderr)?;
          writeln!(err_out, "{shell_name}: {err}")?;
        }
        Ok(RunResult::Continue)
      }
      Cmd::Pwd { stdout, stderr: _ } => {
        let mut out = open_writer(stdout)?;
        let dir = env::current_dir().context("get current directory")?;
        writeln!(out, "{}", dir.display())?;
        Ok(RunResult::Continue)
      }
      Cmd::Unknown { cmd, stderr } => {
        let mut err_out = open_stderr_writer(stderr)?;
        writeln!(err_out, "{shell_name}: command not found: {}", cmd.name)?;
        Ok(RunResult::Continue)
      }
    }
  }
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

  pub fn run_with_stdio(&self, stdout: &StdoutTarget, stderr: &StderrTarget) -> anyhow::Result<()> {
    let program = self.path.as_deref().unwrap_or(&self.name);
    let mut command = std::process::Command::new(program);
    command.args(&self.args);

    match stdout {
      StdoutTarget::Stdout => {}
      StdoutTarget::File(path) => {
        let file = File::create(path)?;
        command.stdout(Stdio::from(file));
      }
    }

    match stderr {
      StderrTarget::Stderr => {}
      StderrTarget::File(path) => {
        let file = File::create(path)?;
        command.stderr(Stdio::from(file));
      }
    }

    let mut child = command
      .spawn()
      .with_context(|| format!("failed to spawn `{program}`"))?;
    child
      .wait()
      .with_context(|| format!("failed to wait for `{program}`"))?;
    Ok(())
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

fn open_writer(target: &StdoutTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StdoutTarget::Stdout => Ok(Box::new(io::stdout())),
    StdoutTarget::File(path) => Ok(Box::new(File::create(path)?)),
  }
}

fn open_stderr_writer(target: &StderrTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StderrTarget::Stderr => Ok(Box::new(io::stderr())),
    StderrTarget::File(path) => Ok(Box::new(File::create(path)?)),
  }
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
