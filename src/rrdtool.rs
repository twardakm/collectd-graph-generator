use anyhow::{Context, Result};
use std::process::{Command, Output};

/// Wrapper holding rrdtool command and parameters
pub struct Rrdtool {
    /// Main rrdtool command, e.g. rrdtool
    command: String,
    /// Vector of parameters, passed later to system wide command
    args: Vec<String>,
}

impl Rrdtool {
    pub fn new() -> Rrdtool {
        Rrdtool {
            command: String::from("rrdtool"),
            args: Vec::new(),
        }
    }

    /// Add subcommand to rrdtool, e.g. graph
    pub fn with_subcommand(mut self, subcommand: String) -> Self {
        self.args.push(subcommand);
        self
    }

    /// Add output file
    pub fn with_output_file(mut self, output: String) -> Self {
        self.args.push(output);
        self
    }

    /// Add width of output file
    pub fn with_width(mut self, width: u32) -> Self {
        self.args.push(String::from("-w"));
        self.args.push(width.to_string());
        self
    }

    /// Add height of output file
    pub fn with_height(mut self, height: u32) -> Self {
        self.args.push(String::from("-h"));
        self.args.push(height.to_string());
        self
    }

    /// Add start timestamp
    pub fn with_start(mut self, start: u64) -> Self {
        self.args.push(String::from("--start"));
        self.args.push(start.to_string());
        self
    }

    /// Add end timestamp
    pub fn with_end(mut self, end: u64) -> Self {
        self.args.push(String::from("--end"));
        self.args.push(end.to_string());
        self
    }

    /// Add custom argument to rrdtool
    pub fn with_custom_argument(mut self, arg: String) -> Self {
        self.args.push(arg);
        self
    }

    /// Execute command
    pub fn exec(self) -> Result<Output> {
        let output = Command::new(&self.command)
            .args(&self.args)
            .output()
            .context(format!(
                "Failed to execute rrdtool: {}, args: {:?}",
                self.command, self.args
            ))?;
        Ok(output)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    pub fn rrdtool_builder() -> Result<()> {
        let rrd = Rrdtool::new()
            .with_output_file(String::from("out.png"))
            .with_subcommand(String::from("graph"))
            .with_start(123456)
            .with_end(1234567);

        assert_eq!("rrdtool", rrd.command);
        assert_eq!(6, rrd.args.len());
        Ok(())
    }

    pub fn rrdtool_simple_exec() -> Result<()> {
        Rrdtool::new().exec().context("Failed to exec rrdtool")?;
        Ok(())
    }
}
