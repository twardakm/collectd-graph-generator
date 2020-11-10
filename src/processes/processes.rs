use super::rrdtool::{Plugin, Rrdtool, Target};

use anyhow::{Context, Result};
use log::{debug, trace};
use std::fs::read_dir;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug)]
pub struct ProcessesData {
    /// Maximum number of processes in one graph
    max_processes: usize,
    /// List of processes to draw, if None all processes are drawn
    processes_to_draw: Option<Vec<String>>,
}

impl ProcessesData {
    pub fn new(max_processes: usize, processes_to_draw: Option<Vec<String>>) -> ProcessesData {
        ProcessesData {
            max_processes: max_processes,
            processes_to_draw: processes_to_draw,
        }
    }
}

impl Rrdtool {
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

        trace!("Listed processes from remote directory: {:?}", processes);

        Ok(processes)
    }

    /// If processes_to_draw is Some, returns only the processes in both vectors
    fn filter_processes(
        processes: Vec<String>,
        processes_to_draw: &Option<Vec<String>>,
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

    /// Add process to the graph
    fn with_process_rss<'a>(
        &mut self,
        input_dir: PathBuf,
        process: String,
        color: String,
        graph_args_no: usize,
    ) -> &Self {
        trace!("Processing {}", process);

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
}

impl Plugin<&ProcessesData> for Rrdtool {
    /// Entry point for a plugin
    fn enter_plugin(&mut self, data: &ProcessesData) -> Result<&mut Self> {
        debug!("Processes plugin entry point");
        trace!("Processes plugin: {:?}", data);

        let processes = self.get_processes_names_from_directory();

        let processes = match processes {
            Ok(processes) => processes,
            Err(error) => anyhow::bail!(
                "Failed to read processes names from directory {}, error: {}",
                self.input_dir,
                error
            ),
        };

        if processes.len() == 0 {
            anyhow::bail!("Couldn't find any processes!");
        }

        trace!("Found processes: {:?}", processes);

        let processes = Rrdtool::filter_processes(processes, &data.processes_to_draw).unwrap();

        trace!("Processes after filtering: {:?}", processes);

        assert!(
            processes.len() < Rrdtool::COLORS.len(),
            "Too many processes! We are running out of colors to proceed."
        );

        let len = processes.len();
        let loops = math::round::ceil(len as f64 / data.max_processes as f64, 0) as u32;

        debug!("{} processes should be saved on {} graphs.", len, loops);

        for i in 0..loops {
            let mut color = 0;

            let lower = i as usize * data.max_processes;
            let upper = std::cmp::min((i as usize + 1) * data.max_processes, processes.len());

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
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use anyhow::Result;
    use std::fs::{create_dir, remove_dir};
    use std::path::Path;
    use tempfile::TempDir;

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

        rrd.enter_plugin(&ProcessesData {
            max_processes: 2,
            processes_to_draw: None,
        })?;

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
    pub fn rrdtool_filter_processes_none() -> Result<()> {
        let processes = vec![
            String::from("firefox"),
            String::from("chrome"),
            String::from("dolphin"),
        ];
        let filtered = Rrdtool::filter_processes(processes.to_vec(), &None)?;
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

        let mut filtered = Rrdtool::filter_processes(processes.to_vec(), &Some(filter.to_vec()))?;
        filtered.sort();

        assert_eq!(
            vec![String::from("dolphin"), String::from("firefox")],
            filtered
        );

        Ok(())
    }
}
