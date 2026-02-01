use std::{
  env,
  path::{Path, PathBuf},
  str::SplitWhitespace,
};

use anyhow::{Context, anyhow};
use pathsearch::find_executable_in_path;
use strum::{Display, EnumIs, EnumTryAs};

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

impl Cmd {
  /// Build a Cmd from already-parsed command name and args (e.g. from main).
  pub fn from_parts(cmd_name: &str, args: Vec<String>) -> Self {
    match cmd_name {
      "cd" => Cmd::Cd(Command::new(cmd_name, None, args)),
      "exit" => Cmd::Exit(0),
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

  // pub fn try_as_command(&self) -> anyhow::Result<Command> {
  //   match self {
  //     Cmd::Exit(_) => Err(anyhow!("exit cmd")),
  //     Cmd::Echo(cmd) => Ok(cmd.clone()),
  //     Cmd::Type(cmd) => Ok(cmd.clone()),
  //     Cmd::Exec(cmd) => Ok(cmd.clone()),
  //     Cmd::Unknown(cmd) => Ok(cmd.clone()),
  //   }
  // }
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
      .spawn()
      .with_context(|| format!("failed to spawn `{program}`"))?;
    child
      .wait()
      .with_context(|| format!("failed to wait for `{program}`"))?;
    Ok(())
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

pub fn change_dir(target: &str) -> anyhow::Result<()> {
  let new_path = if target.is_empty() || target == "~" {
    env::var("HOME").map_err(|_| anyhow!("àṣẹ: HOME not set").context("reading HOME variable"))?
  } else {
    target.to_string()
  };

  let path = Path::new(&new_path);

  if !path.exists() {
    println!("cd: {target}: No such file or directory");
    return Ok(());
  }

  env::set_current_dir(path).map_err(|e| anyhow!("àṣẹ: cd: {}: {}", target, e))?;

  let updated_cwd = env::current_dir()?;

  unsafe {
    env::set_var("PWD", updated_cwd);
  }

  Ok(())
}
