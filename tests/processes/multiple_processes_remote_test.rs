use super::multiple_processes_common;
use anyhow::Result;
use serial_test::serial;

#[test]
#[serial]
fn multiple_processes_remote() -> Result<()> {
    let local = std::env::current_dir()?.join("tests/processes/data");
    let remote = whoami::username() + "@localhost:" + local.to_str().unwrap();

    Ok(multiple_processes_common::multiple_processes(
        std::path::Path::new(&remote),
    )?)
}

#[test]
#[serial]
fn multiple_processes_remote_multiple_files() -> Result<()> {
    let local = std::env::current_dir()?.join("tests/processes/data");
    let remote = whoami::username() + "@localhost:" + local.to_str().unwrap();

    Ok(
        multiple_processes_common::multiple_processes_multiple_files(std::path::Path::new(
            &remote,
        ))?,
    )
}

#[test]
#[serial]
fn multiple_processes_remote_filtered_names() -> Result<()> {
    let local = std::env::current_dir()?.join("tests/processes/data");
    let remote = whoami::username() + "@localhost:" + local.to_str().unwrap();

    Ok(
        multiple_processes_common::multiple_processes_local_filtered_names(std::path::Path::new(
            &remote,
        ))?,
    )
}
