use super::super::common;

use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;
use std::path::Path;

use cgg::config::PluginsConfig;
use cgg::processes::processes_data::ProcessesData;
use cgg::rrdtool::{common::Plugins, common::Rrdtool};

pub fn multiple_processes(input_dir: &Path) -> Result<()> {
    let output_directory = common::init()?;
    let output_file = output_directory.path().join("my output file.png");

    let width = 2048;
    let height = 1024;

    let end = 1604957225;
    let start = end - 3600;

    let mut plugins_config = PluginsConfig {
        data: HashMap::new(),
    };

    plugins_config.data.insert(
        Plugins::Processes,
        Box::new(ProcessesData::new(Rrdtool::COLORS.len(), None)),
    );

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

pub fn multiple_processes_multiple_files(input_dir: &Path) -> Result<()> {
    let output_directory = common::init()?;
    let output_file = output_directory.path().join("other_output_file.png");

    let end = 1604957225;
    let start = end - 3600;

    let mut plugins_config = PluginsConfig {
        data: HashMap::new(),
    };

    plugins_config
        .data
        .insert(Plugins::Processes, Box::new(ProcessesData::new(3, None)));

    debug!(
        "TEST: Calling rrdtool with input dir: {}, output file: {}, start: {}, end: {}",
        input_dir.display(),
        output_file.to_str().unwrap(),
        start,
        end
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
        .with_width(1024)
        .context("Failed with_width")?
        .with_height(768)
        .context("Failed with_height")?
        .with_plugins(plugins_config)
        .context("Failed to execute plugins")?
        .exec()
        .context("Failed to execute rrdtool")?;

    assert!(!output_file.exists());

    let path = output_directory.path().join("other_output_file_1.png");
    assert!(path.exists());

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    let path = output_directory.path().join("other_output_file_2.png");
    assert!(path.exists());

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    let path = output_directory.path().join("other_output_file_3.png");
    assert!(path.exists());

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    let path = output_directory.path().join("other_output_file_4.png");
    assert!(path.exists());

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    let path = output_directory.path().join("other_output_file_5.png");
    assert!(path.exists());

    let metadata = std::fs::metadata(path)?;
    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    assert!(!output_directory
        .path()
        .join("other_output_file_6.png")
        .exists());

    Ok(())
}

pub fn multiple_processes_local_filtered_names(input_dir: &Path) -> Result<()> {
    let output_directory = common::init()?;
    let output_file = output_directory.path().join("my filtered processes.png");

    let width = 2048;
    let height = 1024;

    let end = 1604957225;
    let start = end - 3600;

    let mut plugins_config = PluginsConfig {
        data: HashMap::new(),
    };

    plugins_config.data.insert(
        Plugins::Processes,
        Box::new(ProcessesData::new(
            3,
            Some(vec![
                String::from("baloo_file"),
                String::from("kaccess"),
                String::from("synology note"),
                String::from("some non existing process"),
            ]),
        )),
    );

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
        .context("Failed to execute plugins")?
        .exec()
        .context("Failed to execute rrdtool")?;

    assert!(output_file.exists());

    let metadata = std::fs::metadata(output_file)?;

    assert!(metadata.is_file());
    assert!(metadata.len() > 10000);

    Ok(())
}
