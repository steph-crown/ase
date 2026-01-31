use std::{env, path::PathBuf, str::SplitWhitespace};

use pathsearch::find_executable_in_path;

enum Builtin {
  Cd,
  Type,
  Pwd,
}

impl Builtin {
  fn from_str(s: &str) -> Option<Self> {
    match s {
      "cd" => Some(Builtin::Cd),
      // "ódàbọ̀" | "odabo" => Some(Builtin::Odabo),
      "type" => Some(Builtin::Type),
      "pwd" => Some(Builtin::Pwd),
      _ => None,
    }
  }
}

pub fn resolve_types(commands: SplitWhitespace<'_>) -> String {
  commands
    .map(|cmd| match Builtin::from_str(cmd) {
      Some(_) => format!("{cmd} is a shell builtin"),
      None => {
        let Some(path) = find_executable(cmd) else {
          return format!("{cmd}: not found");
        };
        format!("{cmd} is {}", path.display())
      }
    })
    .collect::<Vec<String>>()
    .join("\n")
}

fn find_executable(cmd: &str) -> Option<PathBuf> {
  find_executable_in_path(cmd)
}
