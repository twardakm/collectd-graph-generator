use anyhow::{Context, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper holding rrdtool command and parameters
pub struct Rrdtool {
    /// Local or Remote
    target: Target,
    /// Path to collectd data
    input_dir: String,
    /// Main rrdtool command, e.g. rrdtool
    command: String,
    /// rrdtool subcommand, e.g. graph
    subcommand: String,
    /// Output filename
    output_filename: String,
    /// Maximum number of processes in one graph
    max_processes: usize,
    /// Common arguments in case of multiple charts
    common_args: Vec<String>,
    /// Vector of vectors of parameters, passed later to system wide command
    /// 2D vector is used in case of e.g. too much processes in one chart,
    /// each dimension keeps arguments for one chart.
    graph_args: Vec<Vec<String>>,
    /// In case of SSH connection
    username: Option<String>,
    /// In case of SSH connection
    hostname: Option<String>,
    /// In case of SSH connection
    remote_filename: Option<String>,
}

/// Enumb used to choose between local and remote data
#[derive(Copy, Clone, PartialEq)]
pub enum Target {
    Local,
    Remote,
}

impl Rrdtool {
    const COLORS: &'static [&'static str] = &[
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
            max_processes: Rrdtool::COLORS.len(),
            output_filename: String::from(""),
            common_args: Vec::new(),
            graph_args: Vec::new(),
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

    /// Set maximum number of processes on a graph
    pub fn with_max_processes(&mut self, max_processes: Option<usize>) -> Result<&mut Self> {
        match max_processes {
            Some(max_processes) => self.max_processes = max_processes,
            None => self.max_processes = Rrdtool::COLORS.len(),
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

    /// Add RSS of all processes available in input_dir
    pub fn with_processes_rss<'a>(
        &mut self,
        processes_to_draw: Option<Vec<String>>,
    ) -> Result<&mut Self> {
        let processes = self.get_processes_names_from_directory();

        let processes = match processes {
            Ok(processes) => processes,
            Err(error) => anyhow::bail!(
                "Failed to read processes names from directory {}, error: {}",
                self.input_dir,
                error
            ),
        };

        let processes = Rrdtool::filter_processes(processes, processes_to_draw).unwrap();

        assert!(
            processes.len() < Rrdtool::COLORS.len(),
            "Too many processes! We are running out of colors to proceed."
        );

        let len = processes.len();
        let loops = math::round::ceil(len as f64 / self.max_processes as f64, 0) as u32;

        for i in 0..loops {
            let mut color = 0;

            let lower = i as usize * self.max_processes;
            let upper = std::cmp::min((i as usize + 1) * self.max_processes, processes.len());

            for process in &processes[lower..upper] {
                self.with_process_rss(
                    PathBuf::from(self.input_dir.as_str()),
                    String::from(process),
                    String::from(Rrdtool::COLORS[color]),
                    i as usize,
                );
                color += 1;
            }
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
        print!("Executing {} ", &self.command);

        match self.target {
            Target::Local => {
                println!("locally...");

                self.exec_local().context("Failed in exec_local")
            }
            Target::Remote => {
                println!("remotely...");

                self.exec_remote().context("Failed in exec_remote")
            }
        }
    }

    /// Execute rrdtool locally
    fn exec_local(&self) -> Result<()> {
        let commands = self.build_rrdtool_args();

        for args in commands {
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

            let output = Command::new("scp")
                .args(args)
                .output()
                .context("Failed to execute SSH")?;

            if output.status.success() == false {
                Rrdtool::print_process_command_output(output);

                anyhow::bail!("Failed to scp result image back to host: scp {:?}", args)
            }
            index += 1;
        }

        Ok(())
    }

    /// Build vector of rrdtool arguments based on data in self
    fn build_rrdtool_args(&self) -> Vec<Vec<String>> {
        let mut commands = Vec::new();

        let no_of_output_files = self.graph_args.len();

        for i in 0..no_of_output_files {
            let index = i as usize;
            commands.push(Vec::new());

            commands[index].push(String::from(self.subcommand.as_str()));

            let output_filename = self.get_output_filename(index);

            match self.target {
                Target::Local => commands[index].push(String::from(output_filename.as_str())),
                Target::Remote => {
                    commands[index].push(String::from(self.remote_filename.as_ref().unwrap()))
                }
            }

            for common_arg in &self.common_args {
                commands[index].push(String::from(common_arg));
            }

            for graph_arg in &self.graph_args[index] {
                commands[index].push(String::from(graph_arg));
            }
        }

        commands
    }

    /// Build output filename based on current index and number of expected output files
    fn get_output_filename(&self, index: usize) -> String {
        match self.graph_args.len() {
            1 => String::from(self.output_filename.as_str()),
            _ => {
                let mut output_filename = String::from(self.output_filename.as_str());
                let appendix = String::from("_") + (index + 1).to_string().as_str();

                output_filename.insert_str(output_filename.rfind(".").unwrap(), appendix.as_str());

                output_filename
            }
        }
    }

    /// Add process to the graph
    fn with_process_rss<'a>(
        &mut self,
        input_dir: PathBuf,
        process: String,
        color: String,
        graph_args_no: usize,
    ) -> &Self {
        let path = input_dir
            .join(String::from("processes-") + &process)
            .join("ps_rss.rrd");

        let process_first_word = process.split_whitespace().next().unwrap();

        if self.graph_args.len() <= graph_args_no {
            self.graph_args.push(Vec::new())
        }

        self.graph_args[graph_args_no].push(
            String::from("DEF:")
                + process_first_word
                + "="
                + match self.target {
                    Target::Local => "",
                    Target::Remote => "\"",
                }
                + path.as_os_str().to_str().unwrap()
                + match self.target {
                    Target::Local => "",
                    Target::Remote => "\"",
                }
                + ":value:AVERAGE",
        );

        self.graph_args[graph_args_no]
            .push(String::from("LINE3:") + process_first_word + &color + ":\"" + &process + "\"");

        self
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

                Ok((
                    target,
                    String::from(captures.get(3).unwrap().as_str()),
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

    /// Parse collectd results directory to get names of analysed processes
    fn get_processes_names_from_directory<'a>(&self) -> Result<Vec<String>> {
        match self.target {
            Target::Local => self.get_processes_names_from_local_directory(),
            Target::Remote => self.get_processes_names_from_remote_directory(),
        }
    }

    /// Get processes from local source
    fn get_processes_names_from_local_directory<'a>(&self) -> Result<Vec<String>> {
        let paths = read_dir(&self.input_dir)
            .context(format!("Failed to read directory: {}", self.input_dir))?;

        let processes = paths
            .filter_map(|path| {
                path.ok().and_then(|path| {
                    path.path().file_name().and_then(|name| {
                        name.to_str()
                            .and_then(|s| s.strip_prefix("processes-"))
                            .map(|s| String::from(s))
                    })
                })
            })
            .collect::<Vec<String>>();

        Ok(processes)
    }

    /// Get processes names from remote directory via SSH and ls commands
    fn get_processes_names_from_remote_directory<'a>(&self) -> Result<Vec<String>> {
        let network_address =
            String::from(self.username.as_ref().unwrap()) + "@" + &self.hostname.as_ref().unwrap();

        let output = Command::new("ssh")
            .args(&[
                &network_address,
                &String::from("ls"),
                &String::from(self.input_dir.as_str()),
            ])
            .output()
            .context("Failed to execute SSH")?;

        if !output.status.success() {
            Rrdtool::print_process_command_output(output);

            anyhow::bail!(
                "Failed to list remote directories in {}:{}!",
                network_address,
                self.input_dir.as_str()
            );
        }

        let output = String::from_utf8_lossy(&output.stdout);

        let processes = output
            .lines()
            .filter_map(|path| path.strip_prefix("processes-"))
            .map(|s| String::from(s))
            .collect::<Vec<String>>();

        Ok(processes)
    }

    /// If processes_to_draw is Some, returns only the processes in both vectors
    fn filter_processes(
        processes: Vec<String>,
        processes_to_draw: Option<Vec<String>>,
    ) -> Result<Vec<String>> {
        match processes_to_draw {
            None => Ok(processes),
            Some(processes_to_draw) => Ok(processes
                .into_iter()
                .filter_map(|process| match processes_to_draw.contains(&process) {
                    true => Some(process),
                    false => None,
                })
                .collect::<Vec<String>>()),
        }
    }

    fn print_process_command_output(output: std::process::Output) {
        eprintln!("status: {}", output.status);
        eprint!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs::{create_dir, remove_dir};
    use std::path::Path;
    use tempfile::TempDir;

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
        assert_eq!(0, rrd.graph_args.len());
        Ok(())
    }

    #[test]
    pub fn rrdtool_simple_exec() -> Result<()> {
        Rrdtool::new(Path::new("/some/local"))
            .with_subcommand(String::from("graph"))?
            .exec()
            .context("Failed to exec rrdtool")?;
        Ok(())
    }

    #[test]
    pub fn rrdtool_with_process_rss() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_process_rss(
            PathBuf::from("/some/path"),
            String::from("firefox"),
            String::from("#00ff00"),
            0,
        );

        assert_eq!(2, rrd.common_args.len() + rrd.graph_args[0].len());
        assert_eq!(
            "DEF:firefox=/some/path/processes-firefox/ps_rss.rrd:value:AVERAGE",
            rrd.graph_args[0][0]
        );
        assert_eq!("LINE3:firefox#00ff00:\"firefox\"", rrd.graph_args[0][1]);

        Ok(())
    }

    #[test]
    pub fn rrdtool_with_process_rss_process_name_with_space() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_process_rss(
            PathBuf::from("/some/path"),
            String::from("rust language server"),
            String::from("#00ff00"),
            0,
        );

        assert_eq!(2, rrd.common_args.len() + rrd.graph_args[0].len());
        assert_eq!(
            "DEF:rust=/some/path/processes-rust language server/ps_rss.rrd:value:AVERAGE",
            rrd.graph_args[0][0]
        );
        assert_eq!(
            "LINE3:rust#00ff00:\"rust language server\"",
            rrd.graph_args[0][1]
        );

        Ok(())
    }

    #[test]
    pub fn rrdtool_with_processes_rss_more_than_max_processes() -> Result<()> {
        let paths = vec![
            Path::new("/tmp/processes-firefox"),
            Path::new("/tmp/processes-chrome"),
            Path::new("/tmp/processes-dolphin"),
            Path::new("/tmp/processes-rust language server"),
            Path::new("/tmp/processes-vscode"),
        ];

        for path in &paths {
            if !path.exists() {
                create_dir(path)?;
            }
        }

        let mut rrd = Rrdtool::new(Path::new("/tmp"));
        rrd.with_max_processes(Some(2))?;

        rrd.with_processes_rss(None)?;

        for path in paths {
            if path.exists() {
                remove_dir(path)?;
            }
        }

        assert_eq!(3, rrd.graph_args.len());

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_directory_local() -> Result<()> {
        let paths = vec![
            Path::new("/tmp/processes-firefox"),
            Path::new("/tmp/processes-chrome"),
            Path::new("/tmp/processes-dolphin"),
            Path::new("/tmp/processes-rust language server"),
        ];

        for path in &paths {
            if !path.exists() {
                create_dir(path)?;
            }
        }

        let rrd = Rrdtool::new(Path::new("/tmp"));

        let mut processes = rrd.get_processes_names_from_directory()?;

        processes.sort();
        assert_eq!(4, processes.len());
        assert_eq!("chrome", processes[0]);
        assert_eq!("dolphin", processes[1]);
        assert_eq!("firefox", processes[2]);
        assert_eq!("rust language server", processes[3]);

        for path in &paths {
            if path.exists() {
                remove_dir(path)?;
            }
        }

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_remote_directory_network_hostname() -> Result<()> {
        let processes = vec!["chrome", "dolphin", "firefox"];
        let temp = TempDir::new().unwrap();
        let origin_path =
            String::from(whoami::username() + "@localhost:" + temp.path().to_str().unwrap());
        let origin_path = Path::new(&origin_path);

        for process in &processes {
            create_dir(Path::new(temp.path()).join(String::from("processes-") + process))?;
        }

        let rrd = Rrdtool::new(origin_path);

        let mut found_processes = rrd.get_processes_names_from_directory()?;

        found_processes.sort();
        assert_eq!(3, found_processes.len());
        assert_eq!("chrome", found_processes[0]);
        assert_eq!("dolphin", found_processes[1]);
        assert_eq!("firefox", found_processes[2]);

        for process in processes {
            remove_dir(Path::new(temp.path()).join(String::from("processes-") + process))?;
        }

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
    pub fn rrdtool_filter_processes_none() -> Result<()> {
        let processes = vec![
            String::from("firefox"),
            String::from("chrome"),
            String::from("dolphin"),
        ];
        let filtered = Rrdtool::filter_processes(processes.to_vec(), None)?;
        assert_eq!(processes, filtered);

        Ok(())
    }

    #[test]
    pub fn rrdtool_filter_processes_some() -> Result<()> {
        let processes = vec![
            String::from("firefox"),
            String::from("chrome"),
            String::from("dolphin"),
            String::from("notepad"),
        ];

        let filter = vec![
            String::from("dolphin"),
            String::from("firefox"),
            String::from("notes"),
        ];

        let mut filtered = Rrdtool::filter_processes(processes.to_vec(), Some(filter.to_vec()))?;
        filtered.sort();

        assert_eq!(
            vec![String::from("dolphin"), String::from("firefox")],
            filtered
        );

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_output_filename_single_file() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_output_file(String::from("some_file.png"))?;
        rrd.graph_args.push(Vec::new());

        let filename = rrd.get_output_filename(0);

        assert_eq!("some_file.png", filename);

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_output_filename_multiple_files() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_output_file(String::from("some other file.png"))?;
        rrd.graph_args.push(Vec::new());
        rrd.graph_args.push(Vec::new());
        rrd.graph_args.push(Vec::new());

        assert_eq!("some other file_1.png", rrd.get_output_filename(0));
        assert_eq!("some other file_2.png", rrd.get_output_filename(1));
        assert_eq!("some other file_3.png", rrd.get_output_filename(2));

        Ok(())
    }
}
