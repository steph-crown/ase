use std::str::SplitWhitespace;

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
      None => format!("{cmd} not found"),
    })
    .collect::<Vec<String>>()
    .join("\n")
}
