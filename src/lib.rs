pub mod config;
pub mod memory;
pub mod processes;
pub mod rrdtool;

use anyhow::{Context, Result};
use config::Config;
use rrdtool::rrdtool::Rrdtool;

pub fn run(config: Config) -> Result<()> {
    Rrdtool::new(config.input_dir)
        .with_subcommand(String::from("graph"))
        .context("Failed with_subcommand")?
        .with_output_file(String::from(config.output_filename))
        .context("Failed with_output_file")?
        .with_start(config.start)
        .context("Failed with_start")?
        .with_end(config.end)
        .context("Failed with_end")?
        .with_width(config.width)
        .context("Failed with_width")?
        .with_height(config.height)
        .context("Failed with_height")?
        .with_plugins(config.plugins_config)
        .context("Failed to execute plugins")?
        .exec()
        .context("Failed to execute rrdtool")?;

    Ok(())
}
