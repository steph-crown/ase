use ase::PROMPT;
use std::io::{self, Write};

fn main() {
  let res_code = match run() {
    Ok(_) => 0,
    Err(err) => {
      println!("{err}");
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
    let mut command = "".to_string();
    io::stdin().read_line(&mut command).unwrap();

    // Eval

    // Print
    println!("{}: command not found", command.trim());
  }
}
