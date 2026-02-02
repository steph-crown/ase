//! Command parsing and execution.

use std::{
  env,
  path::{Path, PathBuf},
};

use anyhow::Context;
use pathsearch::find_executable_in_path;
use strum::{Display, EnumIs, EnumTryAs};

#[derive(Debug, PartialEq)]
pub enum RunResult {
  Continue,
  Exit(u8),
}

#[derive(Debug, PartialEq, EnumIs, EnumTryAs, Display)]
pub enum Cmd {
  Cd(Command),
  Echo(Command),
  Exit(u8),
  Type(Command),
  Exec(Command),
  Pwd,
  Unknown(Command),
}

const BUILTIN_NAMES: &[&str] = &["cd", "echo", "exit", "type", "pwd"];

fn is_builtin(name: &str) -> bool {
  BUILTIN_NAMES.contains(&name)
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
    let (cmd_name, args) = match tokens.split_first() {
      Some((name, rest)) => (name.as_str(), rest.to_vec()),
      None => return Ok(None),
    };
    Ok(Some(Self::from_parts(cmd_name, args)))
  }

  pub fn from_parts(cmd_name: &str, args: Vec<String>) -> Self {
    match cmd_name {
      "cd" => Cmd::Cd(Command::new(cmd_name, None, args)),
      "exit" => {
        let code = args.first().and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
        Cmd::Exit(code)
      }
      "echo" => Cmd::Echo(Command::new(cmd_name, None, args)),
      "type" => Cmd::Type(Command::new(cmd_name, None, args)),
      "pwd" => Cmd::Pwd,
      _ => {
        if let Some(path_buf) = find_executable(cmd_name) {
          let path_str = path_buf
            .into_os_string()
            .into_string()
            .unwrap_or_else(|_| String::new());
          Cmd::Exec(Command::new(cmd_name, Some(path_str), args))
        } else {
          Cmd::Unknown(Command::new(cmd_name, None, args))
        }
      }
    }
  }

  pub fn run(&self, shell_name: &str) -> anyhow::Result<RunResult> {
    match self {
      Cmd::Echo(c) => {
        echo_args(&c.args)?;
        Ok(RunResult::Continue)
      }
      Cmd::Exit(code) => Ok(RunResult::Exit(*code)),
      Cmd::Type(c) => {
        println!("{}", resolve_types(&c.args));
        Ok(RunResult::Continue)
      }
      Cmd::Exec(c) => {
        c.run()?;
        Ok(RunResult::Continue)
      }
      Cmd::Cd(c) => {
        let target = c.args.first().map(String::as_str).unwrap_or("");
        change_dir(target)?;
        Ok(RunResult::Continue)
      }
      Cmd::Pwd => {
        let dir = env::current_dir().context("get current directory")?;
        println!("{}", dir.display());
        Ok(RunResult::Continue)
      }
      Cmd::Unknown(c) => {
        println!("{shell_name}: command not found: {}", c.name);
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

pub fn echo_args(args: &[String]) -> anyhow::Result<()> {
  println!("{}", args.join(" "));
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
    let Cmd::Echo(c) = cmd else { unreachable!() };
    assert_eq!(c.args, vec!["hello   world"]);
  }

  #[test]
  fn from_input_echo_multiple_args() {
    let cmd = Cmd::from_input("echo a b c").unwrap().unwrap();
    let Cmd::Echo(c) = cmd else { unreachable!() };
    assert_eq!(c.args, vec!["a", "b", "c"]);
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
    let cmd = Cmd::from_parts("exit", vec![]);
    assert!(matches!(cmd, Cmd::Exit(0)));
  }

  #[test]
  fn from_parts_exit_with_code() {
    let cmd = Cmd::from_parts("exit", vec!["42".into()]);
    assert!(matches!(cmd, Cmd::Exit(42)));
  }

  #[test]
  fn from_parts_pwd() {
    let cmd = Cmd::from_parts("pwd", vec![]);
    assert!(matches!(cmd, Cmd::Pwd));
  }

  #[test]
  fn from_parts_type_args() {
    let cmd = Cmd::from_parts("type", vec!["cd".into(), "ls".into()]);
    let Cmd::Type(c) = cmd else { unreachable!() };
    assert_eq!(c.args, vec!["cd", "ls"]);
  }

  #[test]
  fn from_parts_cd_args() {
    let cmd = Cmd::from_parts("cd", vec!["/tmp".into()]);
    let Cmd::Cd(c) = cmd else { unreachable!() };
    assert_eq!(c.args, vec!["/tmp"]);
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
