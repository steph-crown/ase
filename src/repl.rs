use crate::commands::{complete_command, find_executable, is_builtin};

use anyhow::Context;
use rustyline::completion::{Completer, FilenameCompleter, Pair, extract_word};
use rustyline::config::{BellStyle, CompletionType, Configurer};
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::history::FileHistory;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Config, Context as RlContext, Editor, Helper};
use std::borrow::Cow;

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

/// List local git branch names that start with `prefix`.
fn git_branches(prefix: &str) -> Vec<String> {
  let Ok(cwd) = std::env::current_dir() else {
    return Vec::new();
  };
  let output = std::process::Command::new("git")
    .args(["branch", "--format=%(refname:short)"])
    .current_dir(&cwd)
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::null())
    .output();
  let Ok(output) = output else {
    return Vec::new();
  };
  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout
    .lines()
    .map(|l| l.trim().to_string())
    .filter(|b| b.starts_with(prefix))
    .collect()
}

const GIT_SUBCOMMANDS: &[&str] = &[
  "add",
  "bisect",
  "branch",
  "checkout",
  "cherry-pick",
  "clone",
  "commit",
  "diff",
  "fetch",
  "grep",
  "init",
  "log",
  "merge",
  "mv",
  "pull",
  "push",
  "rebase",
  "reflog",
  "remote",
  "reset",
  "restore",
  "revert",
  "rm",
  "show",
  "stash",
  "status",
  "switch",
  "tag",
  "worktree",
];

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

    // First token: complete command names
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

    // `cd` and `ls`: complete file/directory names
    if cmd_name == "cd" || cmd_name == "ls" {
      let file_completer = FilenameCompleter::new();
      return file_completer.complete(line, pos, ctx);
    }

    // `git`: complete subcommands, then branch names for branch-taking subcommands
    if cmd_name == "git" {
      let remaining: Vec<&str> = parts.collect();

      if remaining.is_empty() {
        // Completing the git subcommand itself
        let candidates = GIT_SUBCOMMANDS
          .iter()
          .filter(|s| s.starts_with(word))
          .map(|s| Pair {
            display: s.to_string(),
            replacement: s.to_string(),
          })
          .collect();
        return Ok((start, candidates));
      }

      let sub = remaining[0];
      let branch_subs = [
        "checkout",
        "switch",
        "merge",
        "rebase",
        "branch",
        "diff",
        "log",
        "cherry-pick",
        "reset",
      ];
      if branch_subs.contains(&sub) {
        let branches: Vec<Pair> = git_branches(word)
          .into_iter()
          .map(|b| Pair {
            display: b.clone(),
            replacement: b,
          })
          .collect();
        return Ok((start, branches));
      }

      // For other git subcommands, fall through to file completion
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

/// Highlight the first token (command name) in brand color (#fa912a ≈ ANSI 208).
impl Highlighter for AseHelper {
  fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
    let trimmed_start = line.len() - line.trim_start().len();
    let trimmed = &line[trimmed_start..];
    let cmd_end = trimmed.find(|c: char| c == ' ' || c == '\t');

    match cmd_end {
      Some(end) => {
        let cmd = &trimmed[..end];
        let rest = &line[trimmed_start + end..];
        let is_valid = is_builtin(cmd) || find_executable(cmd).is_some();
        let color = if is_valid { "38;5;208" } else { "38;5;196" };
        Cow::Owned(format!(
          "{}\x1b[{color}m{cmd}\x1b[0m{rest}",
          &line[..trimmed_start]
        ))
      }
      None if !trimmed.is_empty() => {
        let cmd = trimmed;
        let is_valid = is_builtin(cmd) || find_executable(cmd).is_some();
        let color = if is_valid { "38;5;208" } else { "38;5;196" };
        Cow::Owned(format!(
          "{}\x1b[{color}m{cmd}\x1b[0m",
          &line[..trimmed_start]
        ))
      }
      _ => Cow::Borrowed(line),
    }
  }

  fn highlight_char(&self, _line: &str, _pos: usize, _forced: CmdKind) -> bool {
    true
  }
}

impl Validator for AseHelper {
  fn validate(&self, _ctx: &mut ValidationContext<'_>) -> rustyline::Result<ValidationResult> {
    Ok(ValidationResult::Valid(None))
  }
}

impl Helper for AseHelper {}
