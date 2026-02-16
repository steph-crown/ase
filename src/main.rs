use ase_shell::{
  SHELL_NAME,
  commands::{RunResult, needs_more_input, run_line},
  repl::create_editor,
  utils::get_prompt,
};

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
  let mut editor = create_editor()?;

  let mut buffer = String::new();
  let mut history: Vec<String> = Vec::new();

  loop {
    let prompt = if buffer.is_empty() {
      get_prompt()
    } else {
      CONTINUATION_PROMPT.to_string()
    };
    let line = editor.readline(&prompt).context("readline")?;
    buffer.push_str(&line);

    if needs_more_input(&buffer) {
      continue;
    }

    if !buffer.trim().is_empty() {
      history.push(buffer.clone());
      let _ = editor.add_history_entry(&buffer);
    }

    let result = run_line(&buffer, SHELL_NAME, &history)?;
    buffer.clear();

    match result {
      RunResult::Continue => {}
      RunResult::Exit(code) => {
        println!("Ó dà bọ̀! \n{SHELL_NAME} has finished");
        return Ok(code);
      }
    }
  }
}
