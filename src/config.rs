use anyhow::Context;
use std::path::Path;

/// Struct with all available options
pub struct Config<'a> {
    /// Path to directory with collectd results
    pub input_dir: &'a Path,
    /// Output filename
    pub output_filename: &'a str,
    /// Width of the generated graph
    pub width: u32,
    /// Height of the generated graph
    pub height: u32,
    /// Start timestamp
    pub start: u64,
    /// End timestamp
    pub end: u64,
}

impl<'a> Config<'a> {
    pub fn new(cli: &'a clap::ArgMatches) -> anyhow::Result<Config<'a>> {
        let input: &str;
        if let Some(input_dir) = cli.value_of("input") {
            input = input_dir;
        } else {
            unreachable!()
        }

        let output: &str;
        if let Some(output_filename) = cli.value_of("out") {
            output = output_filename;
        } else {
            unreachable!()
        }

        let width: u32;
        if let Some(w) = cli.value_of("width") {
            width = w.parse::<u32>().context("Cannot parse width argument")?;
        } else {
            unreachable!()
        }

        let height: u32;
        if let Some(h) = cli.value_of("height") {
            height = h.parse::<u32>().context("Cannot parse height argument")?;
        } else {
            unreachable!()
        }

        let start: u64;
        if let Some(s) = cli.value_of("start") {
            start = s.parse::<u64>().context("Cannot parse start argument")?;
        } else {
            unreachable!()
        }

        let end: u64;
        if let Some(s) = cli.value_of("end") {
            end = s.parse::<u64>().context("Cannot parse end argument")?;
        } else {
            unreachable!()
        }

        Ok(Config {
            input_dir: Path::new(input),
            output_filename: output,
            width: width,
            height: height,
            start: start,
            end: end,
        })
    }
}
