////////////////////////////////////////////////////////////////////////////////
// Stall -- a simple local configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stall test module.
////////////////////////////////////////////////////////////////////////////////
// NOTE: Run the following command to get tracing output:
// RUST_LOG=[test_name]=TRACE cargo test test_name -- --nocapture > .trace

// Internal library imports.


// External library imports.
use pretty_assertions::assert_eq;
use temp_dir::TempDir;
use test_log::test;

// Standard library imports.
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;


fn file_exists(path: &Path) -> bool {
    File::options()
        .read(true)
        .open(path)
        .is_ok()
}


#[test]
#[tracing::instrument]
pub fn default_init() {
    let stall_exec = std::env::current_dir()
        .unwrap()
        .join("target/debug/stall");

    let temp_dir = TempDir::new().expect("create temp dir");
    let stall_path = temp_dir.path();

    let output = std::process::Command::new(stall_exec)
        .arg("init")
        .arg(stall_path)
        .output()
        .expect("execute stall");

    assert!(output.status.success());
    assert!(file_exists(stall_path));
}

