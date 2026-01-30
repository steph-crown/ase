use std::io::{self, Write};

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = "".to_string();

    io::stdin().read_line(&mut command).unwrap();
    println!("{}: command not found", command.trim());
}
