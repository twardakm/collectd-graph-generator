use super::super::config;
use super::rrdtool::remote;
use super::rrdtool::rrdtool::{Plugin, Plugins, Rrdtool, Target};

use anyhow::{Context, Result};
use log::{debug, trace};
use std::fs::read_dir;
use std::path::PathBuf;

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
        let paths = remote::ls(
            &self.input_dir,
            &self.username.as_ref().unwrap(),
            &self.hostname.as_ref().unwrap(),
        )
        .context(format!(
            "Failed to read remote directory {}",
            self.input_dir
        ))?;

        let processes = paths
            .iter()
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

        if self.graph_args.args.len() <= graph_args_no {
            self.graph_args.new_graph();
        }

        self.graph_args
            .push(process.as_str(), color.as_str(), 3, path.to_str().unwrap());

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

impl<'a> config::Config<'a> {
    pub fn get_processes_data(
        cli: &'a clap::ArgMatches,
        plugins: &Vec<Plugins>,
    ) -> Result<Option<ProcessesData>> {
        let processes_to_draw = match cli.value_of("processes") {
            Some(processes) => Some(
                config::Config::parse_processes(String::from(processes))
                    .context(format!("Cannot parse processes {}", processes))?,
            ),
            None => None,
        };

        let max_processes = match cli.value_of("max_processes") {
            Some(max_processes) => Some(
                max_processes
                    .parse::<usize>()
                    .context("Failed to parse max_processes argument")?,
            ),
            None => Some(Rrdtool::COLORS.len()),
        };

        Ok(match plugins.contains(&Plugins::Processes) {
            true => Some(ProcessesData::new(
                max_processes.unwrap(),
                processes_to_draw,
            )),
            false => None,
        })
    }

    /// Return vector of processes to draw graph for from CLI provided list
    fn parse_processes(processes: String) -> anyhow::Result<Vec<String>> {
        Ok(processes
            .split(",")
            .map(|s| String::from(s))
            .collect::<Vec<String>>())
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::super::config;
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

        assert_eq!(2, rrd.common_args.len() + rrd.graph_args.args[0].len());
        assert_eq!(
            "DEF:firefox=/some/path/processes-firefox/ps_rss.rrd:value:AVERAGE",
            rrd.graph_args.args[0][0]
        );
        assert_eq!(
            "LINE3:firefox#00ff00:\"firefox\"",
            rrd.graph_args.args[0][1]
        );

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

        assert_eq!(2, rrd.common_args.len() + rrd.graph_args.args[0].len());
        assert_eq!(
            "DEF:rust=/some/path/processes-rust language server/ps_rss.rrd:value:AVERAGE",
            rrd.graph_args.args[0][0]
        );
        assert_eq!(
            "LINE3:rust#00ff00:\"rust language server\"",
            rrd.graph_args.args[0][1]
        );

        Ok(())
    }

    #[test]
    pub fn rrdtool_with_processes_rss_more_than_max_processes() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let paths = vec![
            temp.path().join("processes-firefox"),
            temp.path().join("processes-chrome"),
            temp.path().join("processes-dolphin"),
            temp.path().join("processes-rust language server"),
            temp.path().join("processes-vscode"),
        ];

        for path in &paths {
            if !path.exists() {
                create_dir(path)?;
            }
        }

        let mut rrd = Rrdtool::new(temp.path());

        rrd.enter_plugin(&ProcessesData {
            max_processes: 2,
            processes_to_draw: None,
        })?;

        for path in paths {
            if path.exists() {
                remove_dir(path)?;
            }
        }

        assert_eq!(3, rrd.graph_args.args.len());

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_directory_local() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let paths = vec![
            temp.path().join("processes-firefox"),
            temp.path().join("processes-chrome"),
            temp.path().join("processes-dolphin"),
            temp.path().join("processes-rust language server"),
        ];

        for path in &paths {
            if !path.exists() {
                create_dir(path)?;
            }
        }

        let rrd = Rrdtool::new(temp.path());

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

    #[test]
    pub fn parse_processes_3_processes() -> Result<()> {
        let mut processes =
            config::Config::parse_processes(String::from("firefox,chrome,dolphin"))?;

        processes.sort();
        assert_eq!(vec!("chrome", "dolphin", "firefox"), processes);

        Ok(())
    }
}
