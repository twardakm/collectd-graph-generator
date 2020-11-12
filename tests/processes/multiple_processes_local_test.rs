use super::multiple_processes_common;
use anyhow::Result;

#[test]
#[ignore]
fn multiple_processes_local() -> Result<()> {
    Ok(multiple_processes_common::multiple_processes(
        &std::env::current_dir()?.join("tests/processes/data"),
    )?)
}

#[test]
#[ignore]
fn multiple_processes_local_multiple_files() -> Result<()> {
    Ok(
        multiple_processes_common::multiple_processes_multiple_files(
            &std::env::current_dir()?.join("tests/processes/data"),
        )?,
    )
}

#[test]
#[ignore]
fn multiple_processes_local_filtered_names() -> Result<()> {
    Ok(
        multiple_processes_common::multiple_processes_local_filtered_names(
            &std::env::current_dir()?.join("tests/processes/data"),
        )?,
    )
}
