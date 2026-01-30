use ase::{PROMPT, SHELL_NAME};
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
  // REPL (Read-Eval-Print-Loop)

  // loop
  loop {
    // Print
    print!("{}", PROMPT);
    io::stdout().flush().unwrap();

    // Read
    let mut input = "".to_string();
    io::stdin().read_line(&mut input).unwrap();

    let mut input = input.trim().split_whitespace();
    let command = input.next().unwrap();

    if command == "exit" {
      println!("Ó dà bọ̀! \n{SHELL_NAME} has finished");
      return Ok(());
    }

    // Print
    println!("{SHELL_NAME}: command not found: {}", command);
  }
}
