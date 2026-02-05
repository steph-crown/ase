use std::{
  fs::File,
  io::{self, Write},
  path::PathBuf,
};

#[derive(Debug, PartialEq, Clone)]
pub enum StdoutTarget {
  Stdout,
  Overwrite(PathBuf),
  Append(PathBuf),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StderrTarget {
  Stderr,
  Overwrite(PathBuf),
  Append(PathBuf),
}

pub fn open_writer(target: &StdoutTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StdoutTarget::Stdout => Ok(Box::new(io::stdout())),
    StdoutTarget::Overwrite(path) => Ok(Box::new(File::create(path)?)),
    StdoutTarget::Append(path) => Ok(Box::new(
      File::options().append(true).create(true).open(path)?,
    )),
  }
}

pub fn open_stderr_writer(target: &StderrTarget) -> anyhow::Result<Box<dyn Write>> {
  match target {
    StderrTarget::Stderr => Ok(Box::new(io::stderr())),
    StderrTarget::Overwrite(path) => Ok(Box::new(File::create(path)?)),
    StderrTarget::Append(path) => Ok(Box::new(
      File::options().append(true).create(true).open(path)?,
    )),
  }
}
