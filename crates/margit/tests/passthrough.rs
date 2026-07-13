//! Integration tests for passthrough: these run the compiled `margit` binary
//! with `MARGIT_GIT` pointed at a stub, because the Unix `exec` handoff can't
//! be unit-tested in-process (it would replace the test runner).
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Write an executable `/bin/sh` stub into `dir` and return its path.
fn stub(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("git-stub");
    fs::write(&path, body).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// A unique temp dir for one test, so parallel tests never collide.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("margit-it-{tag}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn forwards_git_exit_code() {
    let dir = scratch("exit");
    let git = stub(&dir, "#!/bin/sh\nexit 42\n");

    let status = Command::new(env!("CARGO_BIN_EXE_margit"))
        .arg("status")
        .env("MARGIT_GIT", &git)
        .status()
        .unwrap();

    assert_eq!(status.code(), Some(42));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn passes_arguments_through() {
    let dir = scratch("args");
    // Exits 0 only if it received exactly the args margit was given.
    let git = stub(
        &dir,
        "#!/bin/sh\n[ \"$1\" = log ] && [ \"$2\" = --oneline ]\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_margit"))
        .args(["log", "--oneline"])
        .env("MARGIT_GIT", &git)
        .status()
        .unwrap();

    assert!(status.success());
    fs::remove_dir_all(&dir).ok();
}
