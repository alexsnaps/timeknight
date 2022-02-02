use std::fs::{remove_file, OpenOptions};
use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

struct FsStorage {
  location: PathBuf,
}

impl FsStorage {
  pub fn new(location: &Path) -> Result<Self, ErrorKind> {
    if !location.is_dir() {
      return Err(ErrorKind::InvalidInput);
    }

    let lock_location = Self::lock_file(location);

    match OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(lock_location)
    {
      Ok(_) => Ok(FsStorage {
        location: location.to_path_buf(),
      }),
      Err(err) => Err(err.kind()),
    }
  }

  fn lock_file(location: &Path) -> PathBuf {
    location.join(".lock")
  }

  fn close(&mut self) -> Result<(), io::Error> {
    remove_file(Self::lock_file(self.location.as_path()))
  }
}

impl Drop for FsStorage {
  fn drop(&mut self) {
    if self.close().is_err() {
      eprintln!("Failed to remove lock file!")
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::storage::fs::FsStorage;
  use std::env;
  use std::fs::{create_dir, remove_dir};
  use std::io::ErrorKind;
  use std::io::ErrorKind::InvalidInput;
  use std::path::Path;

  #[test]
  fn test_create_errs_on_not_a_valid_dir() {
    assert_eq!(
      FsStorage::new(Path::new("/nowaythisexitsPleaseTellMeNo")).err(),
      Some(InvalidInput)
    );
  }

  #[test]
  fn test_succeeds_on_proper_dir() {
    let location = env::temp_dir().join("tracetimeTest_succeeds_on_proper_dir");
    create_dir(location.as_path()).expect("failed to create temp directory");
    {
      let _working_storage = FsStorage::new(location.as_path()).expect("Failed creating Storage");
      assert_eq!(
        FsStorage::new(location.as_path()).err(),
        Some(ErrorKind::AlreadyExists)
      );
    }
    remove_dir(location.as_path()).expect("couldn't cleanup our test directory!")
  }
}
