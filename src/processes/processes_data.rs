use super::super::config;
use super::rrdtool::common::{Plugins, Rrdtool};

use anyhow::{Context, Result};

/// Data used by processes plugin
///
/// # Examples
///
/// ```
/// use cgg::processes::processes_data::ProcessesData;
///
/// let processes_data =
///     ProcessesData::new(10, Some(vec![String::from("firefox"), String::from("chrome")]));
/// ```
///
#[derive(Debug, Clone)]
pub struct ProcessesData {
    /// Maximum number of processes in one graph
    pub max_processes: usize,
    /// List of processes to draw, if None all processes are drawn
    pub processes_to_draw: Option<Vec<String>>,
}

impl ProcessesData {
    pub fn new(max_processes: usize, processes_to_draw: Option<Vec<String>>) -> ProcessesData {
        ProcessesData {
            max_processes,
            processes_to_draw,
        }
    }
}

impl<'a> config::Config<'a> {
    /// Returns [`ProcessesData`] structure with all data needed by processes plugin
    ///
    /// # Arguments
    /// * `cli` - A reference to [`clap::ArgMatches`] to get data from user
    /// * `plugins` - Vector of plugins already read from command line
    ///
    pub fn get_processes_data(
        cli: &'a clap::ArgMatches,
        plugins: &[Plugins],
    ) -> Result<Option<ProcessesData>> {
        let processes_to_draw = match cli.value_of("processes") {
            Some(processes) => Some(
                parse_processes(String::from(processes))
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
            false => unreachable!(),
        })
    }
}

/// Return vector of processes to draw graph for from CLI provided list
fn parse_processes(processes: String) -> anyhow::Result<Vec<String>> {
    Ok(processes
        .split(',')
        .map(String::from)
        .collect::<Vec<String>>())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn parse_processes_1_process() -> Result<()> {
        let mut processes = super::parse_processes(String::from("firefox"))?;

        processes.sort();
        assert_eq!(vec!("firefox"), processes);

        Ok(())
    }

    #[test]
    pub fn parse_processes_3_processes() -> Result<()> {
        let mut processes = super::parse_processes(String::from("firefox,chrome,dolphin"))?;

        processes.sort();
        assert_eq!(vec!("chrome", "dolphin", "firefox"), processes);

        Ok(())
    }
}
