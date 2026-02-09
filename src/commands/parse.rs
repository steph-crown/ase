use std::path::PathBuf;

use super::targets::{StderrTarget, StdoutTarget};

pub fn needs_more_input(raw: &str) -> bool {
  let r = raw.trim();
  !r.is_empty() && shlex::split(r).is_none()
}

pub struct ParsedInvocation {
  pub cmd_name: String,
  pub args: Vec<String>,
  pub stdout: StdoutTarget,
  pub stderr: StderrTarget,
}

impl ParsedInvocation {
  pub fn from_tokens(tokens: Vec<String>) -> Option<Self> {
    let mut iter = tokens.into_iter();
    let cmd_name = iter.next()?;
    let rest: Vec<String> = iter.collect();

    let mut stdout = StdoutTarget::Stdout;
    let mut stderr = StderrTarget::Stderr;
    let mut args = Vec::new();
    let mut i = 0;

    while i < rest.len() {
      match rest[i].as_str() {
        ">>" | "1>>" => {
          if i + 1 < rest.len() {
            stdout = StdoutTarget::Append(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        ">" | "1>" => {
          if i + 1 < rest.len() {
            stdout = StdoutTarget::Overwrite(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        "2>>" => {
          if i + 1 < rest.len() {
            stderr = StderrTarget::Append(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        "2>" => {
          if i + 1 < rest.len() {
            stderr = StderrTarget::Overwrite(PathBuf::from(rest[i + 1].clone()));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        _ => {
          args.push(rest[i].clone());
          i += 1;
        }
      }
    }

    Some(ParsedInvocation {
      cmd_name,
      args,
      stdout,
      stderr,
    })
  }
}
