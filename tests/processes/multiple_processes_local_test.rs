use super::super::common;
use super::multiple_processes_common;

use anyhow::Result;

use std::process::Command;

#[test]
fn multiple_processes_local() -> Result<()> {
    Ok(multiple_processes_common::multiple_processes(
        &std::env::current_dir()?.join("tests/processes/data"),
    )?)
}

#[test]
fn multiple_processes_local_wrong_directory() -> Result<()> {
    let res = multiple_processes_common::multiple_processes(
        &std::env::current_dir()?.join("tests/processes/dat"),
    );

    assert!(res.is_err());

    Ok(())
}

#[test]
fn multiple_processes_local_multiple_files() -> Result<()> {
    Ok(
        multiple_processes_common::multiple_processes_multiple_files(
            &std::env::current_dir()?.join("tests/processes/data"),
        )?,
    )
}

#[test]
fn multiple_processes_local_filtered_names() -> Result<()> {
    Ok(
        multiple_processes_common::multiple_processes_local_filtered_names(
            &std::env::current_dir()?.join("tests/processes/data"),
        )?,
    )
}

#[test]
fn multiple_processes_local_from_binary() -> Result<()> {
    let output_directory = common::init()?;

    let exec_dir = common::get_cgg_exec_path()?;

    let status = Command::new(&exec_dir)
        .arg("-i")
        .arg(std::env::current_dir()?.join("tests/processes/data"))
        .arg("-p")
        .arg("processes")
        .arg("-o")
        .arg(output_directory.path().join("out.png").to_str().unwrap())
        .arg("-t")
        .arg("last month")
        .status()?;

    assert!(status.success());
    assert!(output_directory.path().join("out.png").exists());

    Ok(())
}

#[test]
fn multiple_processes_local_from_binary_wrong_separator() -> Result<()> {
    let output_directory = common::init()?;

    let exec_dir = common::get_cgg_exec_path()?;

    let status = Command::new(&exec_dir)
        .arg("-i")
        .arg(std::env::current_dir()?.join("tests/processes/data"))
        .arg("-p")
        .arg("processes")
        .arg("-o")
        .arg(output_directory.path().join("out.png").to_str().unwrap())
        .arg("-t")
        .arg("last 2 years")
        .arg("--processes")
        .arg("firefox;chrome;vstudio")
        .status()?;

    assert!(status.success());
    assert!(!output_directory.path().join("out.png").exists());

    Ok(())
}

#[test]
fn multiple_processes_local_from_binary_wrong_max_processes_param() -> Result<()> {
    let output_directory = common::init()?;

    let exec_dir = common::get_cgg_exec_path()?;

    let status = Command::new(&exec_dir)
        .arg("-i")
        .arg(std::env::current_dir()?.join("tests/processes/data"))
        .arg("-p")
        .arg("processes")
        .arg("-o")
        .arg(output_directory.path().join("out.png").to_str().unwrap())
        .arg("-t")
        .arg("last 6 weeks")
        .arg("--max_processes")
        .arg("few")
        .status()?;

    assert!(!status.success());

    Ok(())
}

#[test]
fn multiple_processes_local_from_binary_unix_timestamps() -> Result<()> {
    let output_directory = common::init()?;

    let exec_dir = common::get_cgg_exec_path()?;

    let status = Command::new(&exec_dir)
        .arg("-i")
        .arg(std::env::current_dir()?.join("tests/processes/data"))
        .arg("-p")
        .arg("processes")
        .arg("-o")
        .arg(output_directory.path().join("out.png").to_str().unwrap())
        .arg("--start")
        .arg("1605734459")
        .arg("--end")
        .arg("1605734470")
        .status()?;

    assert!(status.success());

    Ok(())
}
