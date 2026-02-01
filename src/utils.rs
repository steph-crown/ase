use std::{env, path::PathBuf};

use anyhow::Context;

use crate::PROMPT;

pub fn get_pwd() -> anyhow::Result<PathBuf> {
  std::env::current_dir().context("could not retrieve current working directory")
}

pub fn get_prompt() -> String {
  let curr_dir = env::current_dir()
    .map(|path| path.display().to_string())
    .unwrap_or("".to_string());

  format!("{} [{}]", PROMPT, curr_dir)
}
