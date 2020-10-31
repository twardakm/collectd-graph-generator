pub mod config;
pub mod rrdtool;

use anyhow::{Context, Result};
use config::Config;
use rrdtool::Rrdtool;

pub fn run(config: Config) -> Result<()> {
    Rrdtool::new()
        .with_subcommand(String::from("graph"))
        .with_output_file(String::from(config.output_filename))
        .with_start(config.start)
        .with_end(config.end)
        .with_width(config.width)
        .with_height(config.height)
        .with_process_rss(
            config.input_dir,
            String::from("firefox"),
            String::from("#ff0000"),
        )
        .with_process_rss(
            config.input_dir,
            String::from("dolphin"),
            String::from("#00ff00"),
        )
        .with_process_rss(
            config.input_dir,
            String::from("spotify"),
            String::from("#0000ff"),
        )
        .exec()
        .context("Failed to execute rrdtool")?;

    Ok(())
}
