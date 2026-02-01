use ase::{PROMPT, SHELL_NAME, commands::*, utils::get_pwd};
use std::io::{self, Write};

use anyhow::Context;

fn main() {
  let res_code = match run() {
    Ok(_) => 0,
    Err(err) => {
      eprintln!("{SHELL_NAME}: {err:#}");
      1
    }
  };

  std::process::exit(res_code);
}

fn run() -> anyhow::Result<()> {
  loop {
    print!("{}", PROMPT);
    io::stdout().flush().context("flush stdout")?;

    let mut input = String::new();
    io::stdin().read_line(&mut input).context("read stdin")?;

    let mut input = input.trim().split_whitespace();
    let Some(command) = input.next() else {
      continue;
    };

    let args: Vec<String> = input.map(|s| s.to_string()).collect();
    let cmd = Cmd::from_parts(command, args);

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
        c.run()?;
      }
      Cmd::Pwd => {
        let dir = get_pwd().context("get current directory")?;
        println!("{}", dir.display());
      }
      Cmd::Unknown(c) => {
        println!("{SHELL_NAME}: command not found: {}", c.name);
      }
    }
  }
}
