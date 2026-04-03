extern crate libc;
use fs2::FileExt;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

pub struct Lock {
    _file: File,
    path: PathBuf,
}

#[derive(Debug)]
pub enum LockError {
    AlreadyRunning,
    CannotCreate(String),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockError::AlreadyRunning => write!(f, "another instance of zync is already running"),
            LockError::CannotCreate(msg) => write!(f, "cannot create lock file: {msg}"),
        }
    }
}

impl Lock {
    pub fn acquire() -> Result<Self, LockError> {
        let uid = get_uid();
        let path = PathBuf::from(format!("/tmp/zync-{uid}.lock"));

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .map_err(|e| LockError::CannotCreate(e.to_string()))?;

        // fs2 provides a safe flock interface
        file.try_lock_exclusive()
            .map_err(|_| LockError::AlreadyRunning)?;

        Ok(Lock { _file: file, path })
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Return effective UID of the current process.
fn get_uid() -> u32 {
    // SAFETY: getuid() is a pure POSIX query — no side effects, always succeeds.
    unsafe { libc::getuid() }
}