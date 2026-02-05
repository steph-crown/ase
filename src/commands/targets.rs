use std::{
  fs::File,
  io::{self, Write},
  path::PathBuf,
};

#[derive(Debug, PartialEq, Clone)]
pub enum StdoutTarget {
  Stdout,
  File(PathBuf),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StderrTarget {
  Stderr,
  File(PathBuf),
}

pub fn open_writer(target: &StdoutTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StdoutTarget::Stdout => Ok(Box::new(io::stdout())),
    StdoutTarget::File(path) => Ok(Box::new(File::create(path)?)),
  }
}

pub fn open_stderr_writer(target: &StderrTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StderrTarget::Stderr => Ok(Box::new(io::stderr())),
    StderrTarget::File(path) => Ok(Box::new(File::create(path)?)),
  }
}
