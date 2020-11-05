use anyhow::{Context, Result};
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

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
    /// In case of network connection this is a handle to temporary
    /// directory holding rrd data
    temp_directory: Option<TempDir>,
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
        "#ff0022", "#e5ff80", "#f2ba79", "#33cc47", "#bf968f", "#69a653", "#235b8c", "#7f3300",
        "#446600", "#14004d", "#244020", "#ffbfc8", "#eeff00", "#f2aa79", "#b4cc99", "#bf6060",
        "#6ea600", "#00708c", "#7f4620", "#5f6600", "#001f4d", "#304010", "#ff80c4", "#ffee00",
        "#f29979", "#b8cc66", "#b22d3e", "#95a653", "#69858c", "#7f5940", "#665c33", "#00294d",
        "#403e20", "#ff00aa", "#fff780", "#f2beb6", "#cc6d00", "#b38692", "#a69d7c", "#698c8a",
        "#7f3920", "#664d1a", "#00334d", "#403520", "#ff00cc", "#ffcc00", "#f27979", "#cc4733",
        "#b22d62", "#a65800", "#468c7e", "#7f1100", "#661b00", "#003d4d", "#403830", "#ff00ee",
        "#ffbf40", "#f23d3d", "#cc0000", "#b386b0", "#a64200", "#698c6e", "#733949", "#664d4d",
        "#004d3d", "#402820", "#ff80f6", "#ffaa00", "#003de6", "#bf6079", "#982db3", "#a62c00",
        "#468c4f", "#73002e", "#660000", "#004d29", "#401d10", "#cc00ff", "#ffeabf", "#73b0e6",
        "#bf004d", "#742db3", "#a66953", "#818c69", "#731d4b", "#59000c", "#004d0a", "#401010",
        "#d580ff", "#ff8800", "#00d6e6", "#bf6093", "#3e2db3", "#a60000", "#8c8169", "#731d62",
        "#59161f", "#3d4d00", "#402020", "#8800ff", "#ff8c40", "#00e6d6", "#bf8fa9", "#2d50b3",
        "#994d57", "#8c7769", "#001f73", "#590024", "#4c4700", "#330014", "#a640ff", "#ffd9bf",
        "#00e6b8", "#b960bf", "#2d62b3", "#992645", "#8c6e69", "#39736f", "#59434c", "#4c3300",
        "#330022", "#a280ff", "#ffd0bf", "#00e699", "#9360bf", "#aab32d", "#994d75", "#8c3123",
        "#734b1d", "#59003c", "#4c2900", "#331a2b", "#0022ff", "#ff2200", "#00e67a", "#a38fbf",
        "#b2982d", "#994d8a", "#800077", "#734139", "#551659", "#4c1f00", "#300033", "#8091ff",
        "#ff9180", "#00e61f", "#6c60bf", "#b2a159", "#8a4d99", "#7d6080", "#731d1d", "#554359",
        "#4c3626", "#000733", "#80a2ff", "#f23d55", "#7ee639", "#606cbf", "#b2862d", "#269973",
        "#550080", "#66334e", "#3a1659", "#4c0a00", "#1a1d33", "#bfd0ff", "#f27989", "#ace639",
        "#8f96bf", "#b2742d", "#99804d", "#624080", "#633366", "#434359", "#403034", "#001433",
        "#bfe1ff", "#f23d6d", "#e5bf73", "#8fa3bf", "#b28959", "#99574d", "#000080", "#360066",
        "#435259", "#402028", "#002233", "#40bfff", "#f23d85", "#e59539", "#8fafbf", "#b2622d",
        "#8c0013", "#406280", "#413366", "#2d5956", "#401023", "#002933", "#00ccff", "#f2b6ce",
        "#e55c00", "#609fbf", "#b27d59", "#8c696e", "#007780", "#333a66", "#2d593e", "#401030",
        "#003329", "#80e6ff", "#f2b6e6", "#e56739", "#0080bf", "#b2502d", "#8c0025", "#408062",
        "#000e66", "#43594c", "#40303d", "#26332d", "#bff2ff", "#ceb6f2", "#e53d00", "#8fbcbf",
        "#b22d2d", "#8c004b", "#608071", "#334766", "#3e592d", "#3d1040", "#003300", "#bffffb",
        "#6d3df2", "#5700d9", "#60bf9f", "#a60085", "#8c697c", "#008022", "#002966", "#585943",
        "#282040", "#2b331a", "#80ffc3", "#b6b6f2", "#6cd9d2", "#8fbfa9", "#2c00a6", "#8c005e",
        "#448000", "#4d5766", "#595243", "#161040", "#303300", "#00ff66", "#3d6df2", "#d9ce36",
        "#60bf86", "#293aa6", "#69238c", "#628040", "#003666", "#59442d", "#323040", "#323326",
        "#bfffd9", "#3d85f2", "#d9b836", "#8fbf96", "#5374a6", "#77698c", "#668000", "#004466",
        "#593116", "#202d40", "#332900", "#7fffa1", "#3d9df2", "#cc0088", "#60bf60", "#297ca6",
        "#31238c", "#7b8040", "#335c66", "#594943", "#203540", "#331b00", "#90ff80", "#79caf2",
        "#cc00a3", "#bcbf8f", "#5395a6", "#00008c", "#7f7700", "#005266", "#59392d", "#103d40",
        "#33241a", "#88ff00", "#b6f2de", "#be00cc", "#bfb960", "#299da6", "#696e8c", "#7f6600",
        "#006652", "#591f16", "#303f40", "#330e00", "#c3ff80", "#bef2b6", "#8800cc", "#bf8000",
        "#29a69d", "#233f8c", "#7f5500", "#006629", "#592d2d", "#204039", "#332826", "#eaffbf",
        "#f2eeb6", "#3347cc", "#bfa98f", "#29a65b", "#234d8c", "#7f4400", "#33663a", "#4d264a",
        "#104029", "#330000", "#ccff00", "#f2da79", "#33adcc", "#bfa38f", "#3aa629", "#69778c",
        "#7f6240", "#53664d", "#33004d", "#304030",
    ];

    pub fn new<'a>(input_dir: &'a Path) -> Rrdtool {
        let (target, input_dir, username, hostname) = Rrdtool::parse_input_path(input_dir).unwrap();

        Rrdtool {
            target: target,
            input_dir: input_dir,
            command: String::from("rrdtool"),
            args: Vec::new(),
            temp_directory: None,
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
    pub fn with_all_processes_rss<'a>(&mut self) -> &mut Self {
        let directory = self.get_local_path();

        let directory = match directory {
            Ok(directory) => directory,
            Err(error) => {
                eprintln!(
                    "Failed to determine local or network path {}, error: {}",
                    self.input_dir, error
                );
                return self;
            }
        };

        let processes = Rrdtool::get_processes_names_from_directory(&directory);

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
                eprintln!("status: {}", output.status);
                eprint!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));

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

        self.args.push(
            String::from("DEF:")
                + &process
                + "="
                + path.as_os_str().to_str().unwrap()
                + ":value:AVERAGE",
        );

        self.args
            .push(String::from("LINE3:") + &process + &color + ":\"" + &process + "\"");

        self
    }

    /// Get path to local resources (if SSH scp it to local PC)
    fn get_local_path<'a>(&mut self) -> Result<PathBuf> {
        match self.target {
            // Local path
            Target::Local => Ok(PathBuf::from(self.input_dir.as_str())),

            // Assume network path
            Target::Remote => {
                self.temp_directory = Some(TempDir::new().unwrap());

                let status = Command::new("scp")
                    .arg("-r")
                    .arg("-q")
                    .arg(
                        String::from(self.username.as_ref().unwrap())
                            + "@"
                            + self.hostname.as_ref().unwrap().as_str()
                            + ":"
                            + self.input_dir.as_str()
                            + "/*",
                    )
                    .arg(self.temp_directory.as_ref().unwrap().path().to_path_buf())
                    .status()
                    .expect("Failed to execute scp");

                if !status.success() {
                    eprintln!("Error while executing scp");
                    return Ok(PathBuf::new());
                }

                Ok(self.temp_directory.as_ref().unwrap().path().to_path_buf())
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
    fn get_processes_names_from_directory<'a>(input_dir: &'a Path) -> Result<Vec<String>> {
        let paths = read_dir(input_dir).context(format!(
            "Failed to read directory: {}",
            input_dir.to_str().unwrap()
        ))?;

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
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs::{create_dir, remove_dir};
    use std::path::Path;

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
            "DEF:firefox=/some/path/processes-firefox/ps_rss.rrd:value:AVERAGE",
            rrd.args[0]
        );
        assert_eq!("LINE3:firefox#00ff00:\"firefox\"", rrd.args[1]);

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_directory() -> Result<()> {
        let firefox = Path::new("/tmp/processes-firefox");
        let chrome = Path::new("/tmp/processes-chrome");
        let dolphin = Path::new("/tmp/processes-dolphin");

        if !firefox.exists() {
            create_dir(firefox)?;
        }
        if !chrome.exists() {
            create_dir(chrome)?;
        }
        if !dolphin.exists() {
            create_dir(dolphin)?;
        }

        let mut processes = Rrdtool::get_processes_names_from_directory(Path::new("/tmp"))?;

        processes.sort();
        assert_eq!(3, processes.len());
        assert_eq!("chrome", processes[0]);
        assert_eq!("dolphin", processes[1]);
        assert_eq!("firefox", processes[2]);

        remove_dir(firefox)?;
        remove_dir(chrome)?;
        remove_dir(dolphin)?;

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_local_path_local() -> Result<()> {
        let mut rrd = Rrdtool::new(Path::new("/some/local/path"));
        let path = rrd.get_local_path()?;

        assert_eq!(Path::new("/some/local/path"), path);
        assert!(!rrd.temp_directory.is_some());

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_local_path_network_hostname() -> Result<()> {
        let processes = vec!["chrome", "dolphin", "firefox"];
        let temp = TempDir::new().unwrap();
        let origin_path =
            String::from(whoami::username() + "@localhost:" + temp.path().to_str().unwrap());
        let origin_path = Path::new(&origin_path);

        for process in processes {
            create_dir(Path::new(temp.path()).join(String::from("processes-") + process))?;
        }

        let mut rrd = Rrdtool::new(origin_path);

        let local_path = rrd.get_local_path()?;

        assert!(rrd.temp_directory.is_some());
        assert_ne!(origin_path, local_path);
        assert!(!local_path.to_str().unwrap().contains("localhost"));

        let paths = read_dir(local_path)?;
        assert_eq!(3, paths.count());

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
}
