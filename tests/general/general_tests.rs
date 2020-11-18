use super::super::common;

use anyhow::Result;

use std::process::Command;

#[test]
fn main_print_help() -> Result<()> {
    common::init()?;

    let status = Command::new(common::get_cgg_exec_path()?)
        .arg("--help")
        .status()?;

    assert!(status.success());

    Ok(())
}

#[test]
fn main_wrong_directory() -> Result<()> {
    common::init()?;

    let status = Command::new(common::get_cgg_exec_path()?)
        .arg("-i")
        .arg("/tmp")
        .status()?;

    assert!(!status.success());

    Ok(())
}

#[test]
fn main_failed_run() -> Result<()> {
    common::init()?;

    let status = Command::new(common::get_cgg_exec_path()?)
        .arg("-i")
        .arg(&std::env::current_dir()?.join("tests/processes/data"))
        .arg("-t")
        .arg("last 1 hour")
        .arg("--plugins")
        .arg("memory")
        .status()?;

    assert!(!status.success());

    Ok(())
}
