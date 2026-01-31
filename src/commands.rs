use std::{path::PathBuf, str::SplitWhitespace};

use anyhow::anyhow;
use pathsearch::find_executable_in_path;
use strum::{Display, EnumIs, EnumTryAs};

#[derive(Debug, PartialEq, EnumIs, EnumTryAs, Display)]
pub enum Cmd {
  Echo(Command),
  Exit(u8),
  Type(Command),
  Exec(Command),
  Unknown(Command),
}

impl Cmd {
  pub fn try_as_command(&self) -> anyhow::Result<Command> {
    match self {
      Cmd::Exit(_) => Err(anyhow!("exit cmd")),
      Cmd::Echo(cmd) => Ok(cmd.clone()),
      Cmd::Type(cmd) => Ok(cmd.clone()),
      Cmd::Exec(cmd) => Ok(cmd.clone()),
      Cmd::Unknown(cmd) => Ok(cmd.clone()),
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Command {
  /// command alias name
  pub name: String,
  /// actual command path
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
      .spawn()?;
    child.wait()?;
    Ok(())
  }
}

/// Build a Cmd from already-parsed command name and args (e.g. from main).
pub fn cmd_from(cmd_name: &str, args: Vec<String>) -> Cmd {
  match cmd_name {
    "exit" => Cmd::Exit(0),
    "echo" => Cmd::Echo(Command::new(cmd_name, None, args)),
    "type" => Cmd::Type(Command::new(cmd_name, None, args)),
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

fn is_builtin(name: &str) -> bool {
  matches!(name, "cd" | "type" | "pwd")
}

pub fn resolve_types(commands: SplitWhitespace<'_>) -> String {
  commands
    .map(|cmd| {
      if is_builtin(cmd) {
        format!("{cmd} is a shell builtin")
      } else {
        match find_executable(cmd) {
          Some(path) => format!("{cmd} is {}", path.display()),
          None => format!("{cmd}: not found"),
        }
      }
    })
    .collect::<Vec<String>>()
    .join("\n")
}

pub fn find_executable(cmd: &str) -> Option<PathBuf> {
  find_executable_in_path(cmd)
}
