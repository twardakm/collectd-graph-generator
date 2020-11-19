use super::super::common;

use anyhow::{Context, Result};
use log::debug;
use serial_test::serial;

use cgg::config::PluginsConfig;
use std::collections::HashMap;
use std::process::Command;

use cgg::memory::{memory_data::MemoryData, memory_type::MemoryType};
use cgg::rrdtool::common::{Plugins, Rrdtool};

fn system_memory_from_binary(input_dir: &str) -> Result<()> {
    let output_directory = common::init()?;

    let exec_dir = common::get_cgg_exec_path()?;

    let status = Command::new(exec_dir)
        .arg("-i")
        .arg(input_dir)
        .arg("-p")
        .arg("memory")
        .arg("-o")
        .arg(output_directory.path().join("out.png").to_str().unwrap())
        .arg("-t")
        .arg("last 5 minutes")
        .status()?;

    assert!(status.success());

    Ok(())
}

#[test]
fn system_memory_local_from_binary() -> Result<()> {
    system_memory_from_binary(
        std::env::current_dir()?
            .join("tests/memory/data")
            .to_str()
            .unwrap(),
    )
}

#[test]
#[serial]
fn system_memory_remote_from_binary() -> Result<()> {
    system_memory_from_binary(
        &(whoami::username()
            + "@localhost:"
            + std::env::current_dir()?
                .join("tests/memory/data")
                .to_str()
                .unwrap()),
    )
}

#[test]
fn system_memory_local() -> Result<()> {
    let output_directory = common::init()?;

    let output_file = output_directory.path().join("my output file.png");

    let width = 2048;
    let height = 1024;

    let end = 1605275295;
    let start = end - 3600;

    let mut plugins_config = PluginsConfig {
        data: HashMap::new(),
    };

    plugins_config.data.insert(
        Plugins::Memory,
        Box::new(MemoryData::new(vec![
            MemoryType::Buffered,
            MemoryType::Cached,
            MemoryType::Free,
            MemoryType::SlabRecl,
            MemoryType::SlabUnrecl,
            MemoryType::Used,
        ])),
    );

    let input_dir = std::env::current_dir()?.join("tests/memory/data");

    debug!(
        "TEST: Calling rrdtool with input dir: {}, output file: {}, width: {}, height: {}, start: {}, end: {}",
        input_dir.display(), output_file.to_str().unwrap(), width, height, start, end
    );

    Rrdtool::new(&input_dir)
        .with_subcommand(String::from("graph"))
        .context("Failed with_subcommand")?
        .with_output_file(String::from(output_file.to_str().unwrap()))
        .context("Failed with_output_file")?
        .with_start(start)
        .context("Failed with_start")?
        .with_end(end)
        .context("Failed with_end")?
        .with_width(width)
        .context("Failed with_width")?
        .with_height(height)
        .context("Failed with_height")?
        .with_plugins(plugins_config)
        .context("Failed to execute plugin")?
        .exec()
        .context("Failed to execute rrdtool")?;

    assert!(output_file.exists());

    let metadata = std::fs::metadata(output_file)?;

    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    Ok(())
}
