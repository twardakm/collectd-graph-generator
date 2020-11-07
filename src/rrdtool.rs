use anyhow::{Context, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Wrapper holding rrdtool command and parameters
pub struct Rrdtool {
    /// Local or Remote
    target: Target,
    /// Path to collectd data
    input_dir: String,
    /// Main rrdtool command, e.g. rrdtool
    command: String,
    /// Vector of parameters, passed later to system wide command
    args: Vec<String>,
    /// In case of SSH connection
    username: Option<String>,
    /// In case of SSH connection
    hostname: Option<String>,
    /// In case of SSH connection
    local_output: Option<String>,
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
            args: Vec::new(),
            username: username,
            hostname: hostname,
            local_output: None,
        }
    }

    /// Add subcommand to rrdtool, e.g. graph
    pub fn with_subcommand(&mut self, subcommand: String) -> &mut Self {
        self.args.push(subcommand);
        self
    }

    /// Add output file
    pub fn with_output_file<'a>(&mut self, output: String) -> &mut Self {
        match self.target {
            Target::Local => self.args.push(output),
            Target::Remote => {
                self.args.push(String::from("/tmp/cgg-out.png"));
                self.local_output = Some(output);
            }
        }
        self
    }

    /// Add width of output file
    pub fn with_width(&mut self, width: u32) -> &mut Self {
        self.args.push(String::from("-w"));
        self.args.push(width.to_string());
        self
    }

    /// Add height of output file
    pub fn with_height(&mut self, height: u32) -> &mut Self {
        self.args.push(String::from("-h"));
        self.args.push(height.to_string());
        self
    }

    /// Add start timestamp
    pub fn with_start(&mut self, start: u64) -> &mut Self {
        self.args.push(String::from("--start"));
        self.args.push(start.to_string());
        self
    }

    /// Add end timestamp
    pub fn with_end(&mut self, end: u64) -> &mut Self {
        self.args.push(String::from("--end"));
        self.args.push(end.to_string());
        self
    }

    /// Add RSS of all processes available in input_dir
    pub fn with_processes_rss<'a>(&mut self, processes_to_draw: Option<Vec<String>>) -> &mut Self {
        let processes = self.get_processes_names_from_directory();

        let processes = match processes {
            Ok(processes) => processes,
            Err(error) => {
                eprintln!(
                    "Failed to read processes names from directory {}, error: {}",
                    self.input_dir, error
                );
                return self;
            }
        };

        let processes = Rrdtool::filter_processes(processes, processes_to_draw).unwrap();

        assert!(
            processes.len() < Rrdtool::COLORS.len(),
            "Too many processes! We are running out of colors to proceed."
        );

        let mut i = 0;

        for process in processes {
            self.with_process_rss(
                PathBuf::from(self.input_dir.as_str()),
                process,
                String::from(Rrdtool::COLORS[i]),
            );
            i += 1;
        }

        self
    }

    /// Add custom argument to rrdtool
    pub fn with_custom_argument(&mut self, arg: String) -> &mut Self {
        self.args.push(arg);
        self
    }

    /// Execute command
    pub fn exec(&mut self) -> Result<Output> {
        print!("Executing {} ", &self.command);

        let output = match self.target {
            Target::Local => {
                println!("locally...");
                Command::new(&self.command)
                    .args(&self.args)
                    .output()
                    .context(format!(
                        "Failed to execute rrdtool: {}, args: {:?}",
                        self.command, self.args
                    ))?
            }
            Target::Remote => {
                println!("remotely...");
                self.args.insert(
                    0,
                    String::from(self.username.as_ref().unwrap().as_str())
                        + "@"
                        + self.hostname.as_ref().unwrap(),
                );

                self.args.insert(1, String::from(self.command.as_str()));

                let output = Command::new("ssh").args(&self.args).output()?;

                Command::new("scp")
                    .args(&["/tmp/cgg-out.png", &self.local_output.as_ref().unwrap()])
                    .output()
                    .context("Failed to execute SSH")?;

                match output.status.success() {
                    true => Command::new("scp")
                        .args(&["/tmp/cgg-out.png", &self.local_output.as_ref().unwrap()])
                        .output()
                        .context("Failed to scp result image back to host")?,
                    false => output,
                }
            }
        };

        match output.status.success() {
            true => Ok(output),
            false => {
                Rrdtool::print_process_command_output(output);

                anyhow::bail!("Failed to execute command!");
            }
        }
    }

    /// Add process to the graph
    fn with_process_rss<'a>(
        &mut self,
        input_dir: PathBuf,
        process: String,
        color: String,
    ) -> &Self {
        let path = input_dir
            .join(String::from("processes-") + &process)
            .join("ps_rss.rrd");

        let process_first_word = process.split_whitespace().next().unwrap();

        self.args.push(
            String::from("DEF:")
                + process_first_word
                + "=\""
                + path.as_os_str().to_str().unwrap()
                + "\":value:AVERAGE",
        );

        self.args
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
        let output = Command::new("ssh")
            .args(&[
                String::from(self.username.as_ref().unwrap())
                    + "@"
                    + &self.hostname.as_ref().unwrap(),
                String::from("ls"),
                String::from(self.input_dir.as_str()),
            ])
            .output()
            .context("Failed to execute SSH")?;

        if !output.status.success() {
            Rrdtool::print_process_command_output(output);

            anyhow::bail!(
                "Failed to list remote directories in {}!",
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

        rrd.with_output_file(String::from("out.png"))
            .with_subcommand(String::from("graph"))
            .with_start(123456)
            .with_end(1234567);

        assert_eq!("rrdtool", rrd.command);
        assert_eq!(6, rrd.args.len());
        Ok(())
    }

    #[test]
    pub fn rrdtool_simple_exec() -> Result<()> {
        Rrdtool::new(Path::new("/some/local"))
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
        );

        assert_eq!(2, rrd.args.len());
        assert_eq!(
            "DEF:firefox=\"/some/path/processes-firefox/ps_rss.rrd\":value:AVERAGE",
            rrd.args[0]
        );
        assert_eq!("LINE3:firefox#00ff00:\"firefox\"", rrd.args[1]);

        Ok(())
    }

    #[test]
    pub fn rrdtool_with_process_rss_process_name_with_space() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/path"));

        rrd.with_process_rss(
            PathBuf::from("/some/path"),
            String::from("rust language server"),
            String::from("#00ff00"),
        );

        assert_eq!(2, rrd.args.len());
        assert_eq!(
            "DEF:rust=\"/some/path/processes-rust language server/ps_rss.rrd\":value:AVERAGE",
            rrd.args[0]
        );
        assert_eq!("LINE3:rust#00ff00:\"rust language server\"", rrd.args[1]);

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_directory_local() -> Result<()> {
        let firefox = Path::new("/tmp/processes-firefox");
        let chrome = Path::new("/tmp/processes-chrome");
        let dolphin = Path::new("/tmp/processes-dolphin");
        let rust = Path::new("/tmp/processes-rust language server");

        if !firefox.exists() {
            create_dir(firefox)?;
        }
        if !chrome.exists() {
            create_dir(chrome)?;
        }
        if !dolphin.exists() {
            create_dir(dolphin)?;
        }
        if !rust.exists() {
            create_dir(rust)?;
        }

        let rrd = Rrdtool::new(Path::new("/tmp"));

        let mut processes = rrd.get_processes_names_from_directory()?;

        processes.sort();
        assert_eq!(4, processes.len());
        assert_eq!("chrome", processes[0]);
        assert_eq!("dolphin", processes[1]);
        assert_eq!("firefox", processes[2]);
        assert_eq!("rust language server", processes[3]);

        remove_dir(firefox)?;
        remove_dir(chrome)?;
        remove_dir(dolphin)?;
        remove_dir(rust)?;

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
        rrd.with_output_file(String::from("out.png"));

        assert_eq!("out.png", rrd.args[0]);
        Ok(())
    }

    #[test]
    pub fn rrdtool_with_output_file_remote() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("marcin@10.0.0.1:/some/remote/path"));
        rrd.with_output_file(String::from("out.png"));

        assert_eq!("/tmp/cgg-out.png", rrd.args[0]);
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
}
