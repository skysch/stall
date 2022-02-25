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


// External library imports.
// use pretty_assertions::assert_eq;
use temp_dir::TempDir;
use test_log::test;

// Standard library imports.
use std::fs::File;
use std::path::Path;


fn file_exists<P>(path: P) -> bool where P: AsRef<Path> {
    File::options()
        .read(true)
        .open(path)
        .is_ok()
}


fn create_file<P>(path: P) where P: AsRef<Path> {
    let _ = File::options()
        .create_new(true)
        .write(true)
        .open(path.as_ref())
        .expect("create file");
}


#[test]
#[tracing::instrument]
pub fn init_plain() {
    let stall_exec = std::env::current_dir()
        .unwrap()
        .join("target/debug/stall");

    let temp_dir = TempDir::new().expect("create temp dir");
    let stall_path = temp_dir.path();

    println!("{stall_path:?}");

    let output = std::process::Command::new(stall_exec)
        .arg("init")
        .arg(stall_path)
        .output()
        .unwrap();

    println!("{}", String::from_utf8(output.stdout).unwrap());

    assert!(output.status.success());
    assert!(file_exists(stall_path));
}


#[test]
#[tracing::instrument]
pub fn add_multi_collect() {
    let stall_exec = std::env::current_dir()
        .unwrap()
        .join("target/debug/stall");

    let temp_dir_a = TempDir::new().expect("create temp dir");
    let temp_dir_b = TempDir::new().expect("create temp dir");
    let stall_path = temp_dir_a.path();
    let remote_path = temp_dir_b.path();

    println!("{stall_path:?}");
    println!("{remote_path:?}");

    create_file(remote_path.join("a"));
    create_file(remote_path.join("b"));

    println!("{:?}", remote_path.join("a"));
    println!("{:?}", remote_path.join("b"));

    // Init stall
    let output = std::process::Command::new(&stall_exec)
        .arg("init")
        .arg(stall_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(file_exists(stall_path));

    // Run command.
    let output = std::process::Command::new(&stall_exec)
        .args(["add", "--collect", "--stall"])
        .arg(stall_path)
        .args([remote_path.join("a"), remote_path.join("b")])
        .output()
        .unwrap();

    println!("{}", String::from_utf8(output.stdout).unwrap());
    println!("{}", String::from_utf8(output.stderr).unwrap());

    assert!(output.status.success());
    assert!(file_exists(stall_path.join("a")));
    assert!(file_exists(stall_path.join("b")));
}

