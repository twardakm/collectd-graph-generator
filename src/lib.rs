pub mod config;
pub mod rrdtool;

use anyhow::{Context, Result};
use config::Config;
use rrdtool::Rrdtool;

pub fn run(config: Config) -> Result<()> {
    let result = Rrdtool::new()
        .with_subcommand(String::from("graph"))
        .with_output_file(String::from(config.output_filename))
        .with_start(config.start)
        .with_end(config.end)
        .with_width(config.width)
        .with_height(config.height)
        .with_custom_argument(
            String::from("DEF:firefox=")
                + config.input_dir.as_os_str().to_str().unwrap()
                + ":value:AVERAGE",
        )
        .with_custom_argument(String::from("LINE1:firefox#008080:\"Firefox\""))
        .exec()
        .context("Failed to execute rrdtool")?;

    Ok(())
}
