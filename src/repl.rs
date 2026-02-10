use crate::commands::complete_command;

use anyhow::Context;
use rustyline::completion::{Completer, FilenameCompleter, Pair, extract_word};
use rustyline::config::{BellStyle, CompletionType, Configurer};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Config, Context as RlContext, Editor, Helper};

pub type ReplEditor = Editor<AseHelper, FileHistory>;

pub fn create_editor() -> anyhow::Result<ReplEditor> {
  let config = Config::default();
  let history = FileHistory::new();
  let mut editor = Editor::<AseHelper, FileHistory>::with_history(config, history)
    .context("create readline editor")?;
  editor.set_helper(Some(AseHelper));
  editor.set_completion_type(CompletionType::List);
  editor.set_bell_style(BellStyle::Audible);
  Ok(editor)
}

pub struct AseHelper;

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
    ctx: &RlContext<'_>,
  ) -> rustyline::Result<(usize, Vec<Pair>)> {
    let (start, word) = extract_word(line, pos, None, |c| c == ' ' || c == '\t');
    let before = &line[..start];
    let mut parts = before.split_whitespace();
    let first = parts.next();

    if first.is_none() {
      let candidates = complete_command(word)
        .into_iter()
        .map(|s| Pair {
          display: s.clone(),
          replacement: s,
        })
        .collect();
      return Ok((start, candidates));
    }

    let cmd_name = first.unwrap();
    if cmd_name == "cd" {
      let file_completer = FilenameCompleter::new();
      return file_completer.complete(line, pos, ctx);
    }

    Ok((pos, Vec::new()))
  }
}

pub struct EmptyHint;

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
