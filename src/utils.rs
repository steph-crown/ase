use std::{env, path::PathBuf};

pub fn get_pwd() -> PathBuf {
  env::current_dir().expect("àṣẹ: could not retrieve current working directory")
}
