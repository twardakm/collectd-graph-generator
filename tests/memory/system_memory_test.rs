use anyhow::{Context, Result};
use log::debug;
use tempfile::TempDir;

use cgg::config::PluginsConfig;
use cgg::memory::{memory_data::MemoryData, memory_type::MemoryType};
use cgg::rrdtool::rrdtool::{Plugins, Rrdtool};

#[test]
fn system_memory_local() -> Result<()> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
        .format_timestamp(None)
        .try_init();

    let output_directory = TempDir::new()?;
    let output_file = output_directory.path().join("my output file.png");

    let width = 2048;
    let height = 1024;

    let end = 1605275295;
    let start = end - 3600;

    let plugins_config = PluginsConfig {
        plugins: vec![Plugins::Memory],
        processes: None,
        memory: Some(MemoryData::new(vec![
            MemoryType::Buffered,
            MemoryType::Cached,
            MemoryType::Free,
            MemoryType::SlabRecl,
            MemoryType::SlabUnrecl,
            MemoryType::Used,
        ])),
    };

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
    assert!(metadata.len() > 50000);

    Ok(())
}
