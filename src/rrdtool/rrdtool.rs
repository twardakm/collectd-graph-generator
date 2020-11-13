use super::super::config;
use super::graph_arguments::GraphArguments;

use anyhow::{Context, Result};
use log::{debug, error, info, trace};
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

/// Wrapper holding rrdtool command and parameters
pub struct Rrdtool {
    /// Local or Remote
    pub target: Target,
    /// Path to collectd data
    pub input_dir: String,
    /// Main rrdtool command, e.g. rrdtool
    command: String,
    /// rrdtool subcommand, e.g. graph
    subcommand: String,
    /// Output filename
    output_filename: String,
    /// Common arguments in case of multiple charts
    pub common_args: Vec<String>,
    /// Vector of vectors of parameters, passed later to system wide command
    /// 2D vector is used in case of e.g. too much processes in one chart,
    /// each dimension keeps arguments for one chart.
    pub graph_args: GraphArguments,
    /// In case of SSH connection
    pub username: Option<String>,
    /// In case of SSH connection
    pub hostname: Option<String>,
    /// In case of SSH connection
    remote_filename: Option<String>,
}

/// Trait for different plugins
pub trait Plugin<T> {
    /// Entry point for all plugins
    fn enter_plugin(&mut self, data: T) -> Result<&mut Self>;
}

/// Enum used to choose between local and remote data
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Target {
    Local,
    Remote,
}

/// Enum for choosing collectd plugins
#[derive(Copy, Clone, PartialEq)]
pub enum Plugins {
    Processes,
    Memory,
}

impl FromStr for Plugins {
    type Err = ();

    fn from_str(input: &str) -> Result<Plugins, Self::Err> {
        match input {
            "processes" => Ok(Plugins::Processes),
            "memory" => Ok(Plugins::Memory),
            _ => Err(()),
        }
    }
}

impl Rrdtool {
    pub const COLORS: &'static [&'static str] = &[
        "#e6194b", "#3cb44b", "#ffe119", "#4363d8", "#f58231", "#911eb4", "#46f0f0", "#f032e6",
        "#bcf60c", "#fabebe", "#008080", "#e6beff", "#9a6324", "#800000", "#aaffc3", "#808000",
        "#ffd8b1", "#000075", "#808080", "#000000",
    ];

    pub fn new<'a>(input_dir: &'a Path) -> Rrdtool {
        let (target, input_dir, username, hostname) = Rrdtool::parse_input_path(input_dir).unwrap();

        Rrdtool {
            target: target,
            input_dir: input_dir,
            command: String::from("rrdtool"),
            subcommand: String::from(""),
            output_filename: String::from(""),
            common_args: Vec::new(),
            graph_args: GraphArguments::new(target),
            username: username,
            hostname: hostname,
            remote_filename: None,
        }
    }

    /// Add subcommand to rrdtool, e.g. graph
    pub fn with_subcommand(&mut self, subcommand: String) -> Result<&mut Self> {
        self.subcommand = subcommand;
        Ok(self)
    }

    /// Add output file
    pub fn with_output_file<'a>(&mut self, output: String) -> Result<&mut Self> {
        match self.target {
            Target::Local => self.output_filename = output,
            Target::Remote => {
                self.remote_filename = Some(String::from("/tmp/cgg-out.png"));
                self.output_filename = output;
            }
        }
        Ok(self)
    }

    /// Add width of output file
    pub fn with_width(&mut self, width: u32) -> Result<&mut Self> {
        self.common_args.push(String::from("-w"));
        self.common_args.push(width.to_string());
        Ok(self)
    }

    /// Add height of output file
    pub fn with_height(&mut self, height: u32) -> Result<&mut Self> {
        self.common_args.push(String::from("-h"));
        self.common_args.push(height.to_string());
        Ok(self)
    }

    /// Add start timestamp
    pub fn with_start(&mut self, start: u64) -> Result<&mut Self> {
        self.common_args.push(String::from("--start"));
        self.common_args.push(start.to_string());
        Ok(self)
    }

    /// Add end timestamp
    pub fn with_end(&mut self, end: u64) -> Result<&mut Self> {
        self.common_args.push(String::from("--end"));
        self.common_args.push(end.to_string());
        Ok(self)
    }

    /// Run all plugins
    pub fn with_plugins(&mut self, plugins_config: config::PluginsConfig) -> Result<&mut Self> {
        for plugin in plugins_config.plugins {
            match plugin {
                Plugins::Processes => {
                    self.enter_plugin(plugins_config.processes.as_ref().unwrap())
                        .context("Failed \"processes\" plugin")?;
                }
                Plugins::Memory => {
                    self.enter_plugin(plugins_config.memory.as_ref().unwrap())
                        .context("Failed \"memory\" plugin")?;
                }
            };
        }

        Ok(self)
    }

    /// Add custom argument to rrdtool
    pub fn with_custom_argument(&mut self, arg: String) -> Result<&mut Self> {
        self.common_args.push(arg);
        Ok(self)
    }

    /// Execute command
    pub fn exec(&mut self) -> Result<()> {
        match self.target {
            Target::Local => {
                info!("Executing {} locally...", self.command);

                self.exec_local().context("Failed in exec_local")
            }
            Target::Remote => {
                info!("Executing {} remotely...", self.command);

                self.exec_remote().context("Failed in exec_remote")
            }
        }
    }

    /// Execute rrdtool locally
    fn exec_local(&self) -> Result<()> {
        let commands = self.build_rrdtool_args();

        for args in commands {
            trace!("Executing locally: {} {:?}", self.command, args);

            let output = Command::new(&self.command)
                .args(&args)
                .output()
                .context(format!(
                    "Failed to execute rrdtool: {}, args: {:?}",
                    self.command, args
                ))?;

            if output.status.success() == false {
                Rrdtool::print_process_command_output(output);

                anyhow::bail!(
                    "Local rrdtool returned some errors! {} {:?}",
                    self.command,
                    args
                )
            }

            info!("Successfully saved {}", args[1]);
        }

        Ok(())
    }

    /// Execute rrdtool remotely
    fn exec_remote(&self) -> Result<()> {
        let commands = self.build_rrdtool_args();

        let network_address = String::from(self.username.as_ref().unwrap().as_str())
            + "@"
            + self.hostname.as_ref().unwrap();

        let mut index = 0 as usize;
        for mut args in commands {
            // Insert network address
            args.insert(0, String::from(network_address.as_str()));

            // Insert command
            args.insert(1, String::from(self.command.as_str()));

            trace!("Executing remotely: ssh {:?}", args);

            // Execute rrdtool remotely
            let output = Command::new("ssh")
                .args(&args)
                .output()
                .context("Failed to execute SSH command")?;

            if output.status.success() == false {
                Rrdtool::print_process_command_output(output);

                anyhow::bail!("Failed to execute ssh command: ssh {:?}", args)
            }

            let output_filename = self.get_output_filename(index);

            // scp result back to host
            let args = &[
                String::from(&network_address) + ":" + self.remote_filename.as_ref().unwrap(),
                String::from(output_filename.as_str()),
            ];

            trace!("Executing remotely: scp {:?}", args);

            let output = Command::new("scp")
                .args(args)
                .output()
                .context("Failed to execute SSH")?;

            if output.status.success() == false {
                Rrdtool::print_process_command_output(output);

                anyhow::bail!("Failed to scp result image back to host: scp {:?}", args)
            }

            info!("Successfully saved {}", output_filename);

            index += 1;
        }

        Ok(())
    }

    /// Build vector of rrdtool arguments based on data in self
    fn build_rrdtool_args(&self) -> Vec<Vec<String>> {
        let mut commands = Vec::new();

        let no_of_output_files = self.graph_args.args.len();

        debug!("Building arguments for {} files.", no_of_output_files);

        for i in 0..no_of_output_files {
            let index = i as usize;
            commands.push(Vec::new());

            commands[index].push(String::from(self.subcommand.as_str()));

            let output_filename = self.get_output_filename(index);

            match self.target {
                Target::Local => {
                    commands[index].push(String::from(output_filename.as_str()));
                    debug!("Building arguments for local {} file.", output_filename);
                }
                Target::Remote => {
                    commands[index].push(String::from(self.remote_filename.as_ref().unwrap()));
                    debug!(
                        "Building arguments for remote {} file.",
                        self.remote_filename.as_ref().unwrap()
                    );
                }
            }

            for common_arg in &self.common_args {
                commands[index].push(String::from(common_arg));
            }

            for graph_arg in &self.graph_args.args[index] {
                commands[index].push(String::from(graph_arg));
            }

            trace!(
                "Built arguments for {} filename: {:?}",
                output_filename,
                commands
            );
        }

        commands
    }

    /// Build output filename based on current index and number of expected output files
    fn get_output_filename(&self, index: usize) -> String {
        match self.graph_args.args.len() {
            1 => String::from(self.output_filename.as_str()),
            _ => {
                let mut output_filename = String::from(self.output_filename.as_str());
                let appendix = String::from("_") + (index + 1).to_string().as_str();

                output_filename.insert_str(output_filename.rfind(".").unwrap(), appendix.as_str());

                trace!("Returning output filename: {}", output_filename);

                output_filename
            }
        }
    }

    /// Parse input path to get target type, path, username and hostname
    fn parse_input_path<'a>(
        input_dir: &'a Path,
    ) -> Result<(Target, String, Option<String>, Option<String>)> {
        let re = regex::Regex::new(".*@.*:.*").context("Failed to create regex")?;

        match re.is_match(input_dir.to_str().context("Failed to parse regex")?) {
            // Remote
            true => {
                let target = Target::Remote;

                let re = regex::Regex::new("(.*)@(.*):(.*)").unwrap();
                let captures = re.captures(input_dir.to_str().unwrap()).unwrap();
                let username = captures[1].to_string();
                let hostname = captures[2].to_string();
                let remote_path = captures.get(3).unwrap().as_str();

                trace!(
                    "Parsed remote path, username: {}, hostname: {}, path: {}",
                    username,
                    hostname,
                    remote_path
                );

                Ok((
                    target,
                    String::from(remote_path),
                    Some(username),
                    Some(hostname),
                ))
            }

            // Local
            false => {
                let target = Target::Local;
                Ok((
                    target,
                    String::from(input_dir.to_str().unwrap()),
                    None,
                    None,
                ))
            }
        }
    }

    /// Print output of system command
    pub fn print_process_command_output(output: std::process::Output) {
        error!("status: {}", output.status);
        error!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        error!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;
    use std::path::Path;

    #[test]
    pub fn rrdtool_builder() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/local/"));

        rrd.with_output_file(String::from("out.png"))?
            .with_subcommand(String::from("graph"))?
            .with_start(123456)?
            .with_end(1234567)?;

        assert_eq!("rrdtool", rrd.command);
        assert_eq!("out.png", rrd.output_filename);
        assert_eq!("graph", rrd.subcommand);
        assert_eq!(4, rrd.common_args.len());
        assert_eq!(0, rrd.graph_args.args.len());
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn rrdtool_simple_exec() -> Result<()> {
        Rrdtool::new(Path::new("/some/local"))
            .with_subcommand(String::from("graph"))?
            .exec()
            .context("Failed to exec rrdtool")?;
        Ok(())
    }

    #[test]
    pub fn rrdtool_with_output_file_local() -> Result<()> {
        let path = Path::new("/some/local/path");
        let mut rrd = Rrdtool::new(path);
        rrd.with_output_file(String::from("out.png"))?;

        assert_eq!("out.png", rrd.output_filename);
        Ok(())
    }

    #[test]
    #[ignore]
    pub fn rrdtool_with_output_file_remote() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("marcin@10.0.0.1:/some/remote/path"));
        rrd.with_output_file(String::from("out.png"))?;

        assert_eq!("/tmp/cgg-out.png", rrd.remote_filename.unwrap());
        Ok(())
    }

    #[test]
    pub fn rrdtool_parse_input_path_local() -> Result<()> {
        let original_path = Path::new("/some/local/path");
        let (target, path, username, hostname) = Rrdtool::parse_input_path(&original_path)?;

        assert!(Target::Local == target);
        assert_eq!(original_path.to_str().unwrap(), path);
        assert!(username.is_none());
        assert!(hostname.is_none());

        Ok(())
    }

    #[test]
    pub fn rrdtool_parse_input_path_remote_hostname() -> Result<()> {
        let original_path = Path::new("marcin@localhost:/some/remote/path");
        let (target, path, username, hostname) = Rrdtool::parse_input_path(&original_path)?;

        assert!(Target::Remote == target);
        assert_eq!("/some/remote/path", path);
        assert_eq!("marcin", username.unwrap());
        assert_eq!("localhost", hostname.unwrap());

        Ok(())
    }

    #[test]
    pub fn rrdtool_parse_input_path_remote_ip() -> Result<()> {
        let original_path = Path::new("twardak@10.0.0.52:/some/remote/path/");
        let (target, path, username, hostname) = Rrdtool::parse_input_path(&original_path)?;

        assert!(Target::Remote == target);
        assert_eq!("/some/remote/path/", path);
        assert_eq!("twardak", username.unwrap());
        assert_eq!("10.0.0.52", hostname.unwrap());

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_output_filename_single_file() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_output_file(String::from("some_file.png"))?;
        rrd.graph_args.new_graph();

        let filename = rrd.get_output_filename(0);

        assert_eq!("some_file.png", filename);

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_output_filename_multiple_files() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_output_file(String::from("some other file.png"))?;
        rrd.graph_args.new_graph();
        rrd.graph_args.new_graph();
        rrd.graph_args.new_graph();

        assert_eq!("some other file_1.png", rrd.get_output_filename(0));
        assert_eq!("some other file_2.png", rrd.get_output_filename(1));
        assert_eq!("some other file_3.png", rrd.get_output_filename(2));

        Ok(())
    }
}
