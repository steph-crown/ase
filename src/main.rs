use ase::{
  SHELL_NAME,
  commands::{Cmd, RunResult, complete_command, needs_more_input},
  utils::get_prompt,
};

use anyhow::Context;
use rustyline::completion::{Completer, Pair, extract_word};
use rustyline::config::{BellStyle, CompletionType, Configurer};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::Validator;
use rustyline::validate::{ValidationContext, ValidationResult};
use rustyline::{Config, Context as RlContext, Editor, Helper};

const CONTINUATION_PROMPT: &str = "> ";

struct AseHelper;

impl Default for AseHelper {
  fn default() -> Self {
    Self
  }
}

impl Completer for AseHelper {
  type Candidate = Pair;

  fn complete(
    &self,
    line: &str,
    pos: usize,
    _ctx: &RlContext<'_>,
  ) -> rustyline::Result<(usize, Vec<Pair>)> {
    // Only complete the first word (no space before cursor).
    let (start, word) = extract_word(line, pos, None, |c| c == ' ' || c == '\t');
    if start > 0 {
      return Ok((pos, vec![]));
    }
    let candidates = complete_command(word)
      .into_iter()
      .map(|s| Pair {
        display: s.clone(),
        replacement: s,
      })
      .collect();
    Ok((start, candidates))
  }
}

struct EmptyHint;

impl rustyline::hint::Hint for EmptyHint {
  fn display(&self) -> &str {
    ""
  }
  fn completion(&self) -> Option<&str> {
    None
  }
}

impl Hinter for AseHelper {
  type Hint = EmptyHint;

  fn hint(&self, _line: &str, _pos: usize, _ctx: &RlContext<'_>) -> Option<EmptyHint> {
    None
  }
}

impl Highlighter for AseHelper {}

impl Validator for AseHelper {
  fn validate(&self, _ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
    Ok(ValidationResult::Valid(None))
  }
}

impl Helper for AseHelper {}

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
  let config = Config::default();
  let history = FileHistory::new();
  let mut editor = Editor::<AseHelper, FileHistory>::with_history(config, history)
    .context("create readline editor")?;
  editor.set_helper(Some(AseHelper));
  editor.set_completion_type(CompletionType::List);
  editor.set_bell_style(BellStyle::Audible);

  let mut buffer = String::new();

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
