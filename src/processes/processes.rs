use super::processes_data::ProcessesData;
use super::processes_names;
use super::rrdtool::rrdtool::{Plugin, Rrdtool};

use anyhow::Result;
use log::{debug, trace};
use std::path::PathBuf;

impl Rrdtool {
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

        let processes =
            processes_names::get(self.target, &self.input_dir, &self.username, &self.hostname);

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

        let processes = filter_processes(processes, &data.processes_to_draw).unwrap();

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
    pub fn rrdtool_filter_processes_none() -> Result<()> {
        let processes = vec![
            String::from("firefox"),
            String::from("chrome"),
            String::from("dolphin"),
        ];
        let filtered = filter_processes(processes.to_vec(), &None)?;
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

        let mut filtered = filter_processes(processes.to_vec(), &Some(filter.to_vec()))?;
        filtered.sort();

        assert_eq!(
            vec![String::from("dolphin"), String::from("firefox")],
            filtered
        );

        Ok(())
    }
}
