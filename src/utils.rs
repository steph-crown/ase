use std::path::PathBuf;

use anyhow::Context;

pub fn get_pwd() -> anyhow::Result<PathBuf> {
  std::env::current_dir().context("could not retrieve current working directory")
}
