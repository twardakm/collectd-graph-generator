pub mod config;
pub mod rrdtool;

use anyhow::{Context, Result};
use config::Config;
use rrdtool::Rrdtool;

pub fn run(config: Config) -> Result<()> {
    Rrdtool::new()
        .with_subcommand(String::from("graph"))
        .with_output_file(String::from(config.output_filename), config.input_dir)
        .with_start(config.start)
        .with_end(config.end)
        .with_width(config.width)
        .with_height(config.height)
        .with_all_processes_rss(config.input_dir)
        .exec()
        .context("Failed to execute rrdtool")?;

    Ok(())
}
