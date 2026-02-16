use std::{
  env, fs,
  path::{Path, PathBuf},
};

use anyhow::Context;

use crate::SHELL_NAME;

pub fn get_pwd() -> anyhow::Result<PathBuf> {
  std::env::current_dir().context("could not retrieve current working directory")
}

pub fn get_prompt() -> String {
  let curr_dir = env::current_dir()
    .ok()
    .and_then(|path| path.file_name().map(|s| s.to_string_lossy().into_owned()))
    .unwrap_or_else(|| "".to_string());

  let git_branch = current_git_branch().unwrap_or_default();
  let branch_part = if git_branch.is_empty() {
    String::new()
  } else {
    format!(" ({git_branch})")
  };

  format!(
    "\x1b[32m{}\x1b[0m [{}{}] \x1b[1m>\x1b[0m ",
    SHELL_NAME, curr_dir, branch_part
  )
}

fn current_git_branch() -> Option<String> {
  let cwd = env::current_dir().ok()?;
  let git_dir = find_git_dir(&cwd)?;
  let head_path = git_dir.join("HEAD");
  let head_contents = fs::read_to_string(head_path).ok()?;

  if let Some(rest) = head_contents.trim().strip_prefix("ref:") {
    let ref_path = rest.trim();
    return Path::new(ref_path)
      .file_name()
      .and_then(|s| Some(s.to_string_lossy().into_owned()));
  }

  // Detached HEAD: show short SHA
  let sha = head_contents.trim();
  if sha.is_empty() {
    None
  } else {
    Some(sha.chars().take(7).collect())
  }
}

fn find_git_dir(start: &Path) -> Option<PathBuf> {
  let mut dir = Some(start);

  while let Some(current) = dir {
    let candidate = current.join(".git");
    if candidate.is_dir() {
      return Some(candidate);
    }
    dir = current.parent();
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn prompt_includes_current_directory_name() {
    let cwd = env::current_dir().unwrap();
    let name = cwd
      .file_name()
      .map(|s| s.to_string_lossy().into_owned())
      .unwrap_or_default();

    let prompt = get_prompt();
    assert!(prompt.contains(&name));
  }

  #[test]
  fn prompt_includes_git_branch_when_in_repo() {
    let tmp = env::temp_dir().join(format!("ase_git_test_{}", std::process::id()));
    fs::create_dir_all(&tmp).unwrap();
    let git_dir = tmp.join(".git");
    fs::create_dir_all(&git_dir).unwrap();

    let head_path = git_dir.join("HEAD");
    fs::write(&head_path, "ref: refs/heads/feature/test\n").unwrap();

    let old_cwd = env::current_dir().unwrap();
    env::set_current_dir(&tmp).unwrap();

    let prompt = get_prompt();

    env::set_current_dir(old_cwd).unwrap();
    fs::remove_file(&head_path).ok();
    fs::remove_dir_all(&git_dir).ok();
    fs::remove_dir_all(&tmp).ok();

    assert!(prompt.contains("(test)"));
  }
}
