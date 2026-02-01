use std::{env, path::PathBuf};

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

  format!(
    "\x1b[32m{}\x1b[0m [{}] \x1b[1m>\x1b[0m ",
    SHELL_NAME, curr_dir
  )
}
