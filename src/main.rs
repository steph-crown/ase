use ase::{
  SHELL_NAME,
  commands::{Cmd, RunResult, needs_more_input},
  utils::get_prompt,
};
use std::io::{self, Write};

use anyhow::Context;

const CONTINUATION_PROMPT: &str = "> ";

fn main() {
  let code = match run() {
    Ok(exit_code) => exit_code as i32,
    Err(err) => {
      eprintln!("{SHELL_NAME}: {err:#}");
      1
    }
  };
  std::process::exit(code);
}

fn run() -> anyhow::Result<u8> {
  let mut buffer = String::new();

  loop {
    let prompt = if buffer.is_empty() {
      get_prompt()
    } else {
      CONTINUATION_PROMPT.to_string()
    };
    print!("{prompt}");
    io::stdout().flush().context("flush stdout")?;

    let mut line = String::new();
    io::stdin().read_line(&mut line).context("read stdin")?;
    buffer.push_str(&line);

    if needs_more_input(&buffer) {
      continue;
    }

    let Some(cmd) = Cmd::from_input(&buffer)? else {
      buffer.clear();
      continue;
    };
    buffer.clear();

    match cmd.run(SHELL_NAME)? {
      RunResult::Continue => {}
      RunResult::Exit(code) => {
        println!("Ó dà bọ̀! \n{SHELL_NAME} has finished");
        return Ok(code);
      }
    }
  }
}
