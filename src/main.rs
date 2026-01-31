use ase::{PROMPT, SHELL_NAME, commands::*};
use std::io::{self, Write};

fn main() {
  let res_code = match run() {
    Ok(_) => 0,
    Err(err) => {
      println!("{SHELL_NAME}: {err}");
      1
    }
  };

  std::process::exit(res_code);
}

fn run() -> Result<(), String> {
  loop {
    print!("{}", PROMPT);
    io::stdout().flush().unwrap();

    let mut input = "".to_string();
    io::stdin().read_line(&mut input).unwrap();

    let mut input = input.trim().split_whitespace();
    let Some(command) = input.next() else {
      continue;
    };

    let args: Vec<String> = input.map(|s| s.to_string()).collect();
    let cmd = cmd_from(command, args);

    match cmd {
      Cmd::Exit(_) => {
        println!("Ó dà bọ̀! \n{SHELL_NAME} has finished");
        return Ok(());
      }
      Cmd::Echo(c) => {
        println!("{}", c.args.join(" "));
      }
      Cmd::Type(c) => {
        println!("{}", resolve_types(c.args.join(" ").split_whitespace()));
      }
      Cmd::Exec(c) => {
        c.run().map_err(|e| e.to_string())?;
      }
      Cmd::Unknown(c) => {
        println!("{SHELL_NAME}: command not found: {}", c.name);
      }
    }
  }
}
