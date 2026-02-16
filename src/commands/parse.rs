use std::{env, path::PathBuf};

use glob::glob;

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
            let target = expand_single_path(&rest[i + 1]);
            stdout = StdoutTarget::Append(PathBuf::from(target));
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
            let target = expand_single_path(&rest[i + 1]);
            stdout = StdoutTarget::Overwrite(PathBuf::from(target));
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
            let target = expand_single_path(&rest[i + 1]);
            stderr = StderrTarget::Append(PathBuf::from(target));
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
            let target = expand_single_path(&rest[i + 1]);
            stderr = StderrTarget::Overwrite(PathBuf::from(target));
            i += 2;
            continue;
          } else {
            args.push(rest[i].clone());
            i += 1;
            continue;
          }
        }
        _ => {
          let expanded = expand_arg(&rest[i]);
          args.extend(expanded);
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

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;

  #[test]
  fn expands_env_vars_and_tilde_in_args() {
    unsafe {
      std::env::set_var("FOO", "bar");
      std::env::set_var("HOME", "/home/testuser");
    }

    let tokens = vec![
      "echo".to_string(),
      "$FOO".to_string(),
      "x$FOO".to_string(),
      "~".to_string(),
      "~/dir".to_string(),
    ];

    let inv = ParsedInvocation::from_tokens(tokens).unwrap();
    assert_eq!(
      inv.args,
      vec![
        "bar".to_string(),
        "xbar".to_string(),
        "/home/testuser".to_string(),
        "/home/testuser/dir".to_string()
      ]
    );
  }

  #[test]
  fn expands_globs_in_args() {
    let dir = std::env::temp_dir().join(format!("ase_glob_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();

    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    let c = dir.join("c.log");
    fs::write(&a, "a").unwrap();
    fs::write(&b, "b").unwrap();
    fs::write(&c, "c").unwrap();

    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let tokens = vec!["echo".to_string(), "*.txt".to_string()];
    let inv = ParsedInvocation::from_tokens(tokens).unwrap();

    let mut args = inv.args.clone();
    args.sort();
    assert_eq!(args, vec!["a.txt".to_string(), "b.txt".to_string()]);

    std::env::set_current_dir(old_cwd).unwrap();
    fs::remove_file(&a).ok();
    fs::remove_file(&b).ok();
    fs::remove_file(&c).ok();
    fs::remove_dir_all(&dir).ok();
  }

  #[test]
  fn expands_tilde_and_vars_in_redirection_paths() {
    let home = std::env::var("HOME").unwrap_or_default();

    let tokens = vec![
      "echo".to_string(),
      "hi".to_string(),
      ">".to_string(),
      "~/out.txt".to_string(),
      "2>".to_string(),
      "/tmp/dummy.log".to_string(),
    ];

    let inv = ParsedInvocation::from_tokens(tokens).unwrap();

    match inv.stdout {
      StdoutTarget::Overwrite(p) => {
        let expected = PathBuf::from(&home).join("out.txt");
        assert_eq!(p, expected);
      }
      _ => panic!("unexpected stdout target"),
    }

    // stderr target is covered by other tests; here we just care that we don't
    // panic and that tilde expansion happened correctly for stdout.
  }
}

fn expand_arg(token: &str) -> Vec<String> {
  let token = expand_vars_and_tilde(token);

  if has_glob_meta(&token) {
    let mut results = Vec::new();
    if let Ok(paths) = glob(&token) {
      for entry in paths.flatten() {
        if let Some(s) = entry.to_str() {
          results.push(s.to_string());
        }
      }
    }
    if !results.is_empty() {
      return results;
    }
  }

  vec![token]
}

fn expand_single_path(token: &str) -> String {
  expand_vars_and_tilde(token)
}

fn expand_vars_and_tilde(token: &str) -> String {
  let mut s = token.to_string();

  if let Some(home) = env::var_os("HOME") {
    if let Some(stripped) = s.strip_prefix('~') {
      if stripped.is_empty() || stripped.starts_with('/') {
        if let Some(home_str) = home.to_str() {
          s = format!("{home_str}{stripped}");
        }
      }
    }
  }

  s = expand_vars(&s);
  s
}

fn expand_vars(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  let bytes = s.as_bytes();
  let mut i = 0;

  while i < bytes.len() {
    if bytes[i] == b'$' {
      let start = i + 1;
      let mut j = start;
      while j < bytes.len() && (bytes[j] == b'_' || bytes[j].is_ascii_alphanumeric()) {
        j += 1;
      }
      if j > start {
        let name = &s[start..j];
        if let Ok(val) = env::var(name) {
          out.push_str(&val);
        }
        i = j;
        continue;
      }
    }

    out.push(bytes[i] as char);
    i += 1;
  }

  out
}

fn has_glob_meta(s: &str) -> bool {
  s.chars().any(|c| matches!(c, '*' | '?' | '['))
}
