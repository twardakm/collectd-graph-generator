use super::{processes::processes, rrdtool};
use anyhow::{anyhow, Context};
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

/// Struct with all available options
pub struct Config<'a> {
    /// Common settings
    /// ---------------
    ///
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
    /// ---------------
    /// Plugins
    /// ---------------
    pub plugins_config: PluginsConfig,
}

pub struct PluginsConfig {
    /// Vector of enums to choose which plugins should be executed
    pub plugins: Vec<rrdtool::Plugins>,
    /// Processes plugin
    pub processes: Option<processes::ProcessesData>,
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

        let (start, end) = match cli.value_of("timespan") {
            Some(timespan) => Config::parse_timespan(String::from(timespan))
                .context(format!("Cannot parse timespan {}", timespan))?,
            None => (
                cli.value_of("start")
                    .context("Missing --start parameter")?
                    .parse::<u64>()
                    .context("Cannot parse start argument")?,
                cli.value_of("end")
                    .context("Missing --end parameter")?
                    .parse::<u64>()
                    .context("Cannot parse start argument")?,
            ),
        };

        let plugins = match cli.value_of("plugins") {
            Some(plugins) => {
                let mut vec = Vec::new();

                for plugin in plugins.split(",").collect::<Vec<&str>>() {
                    let plugin = match rrdtool::Plugins::from_str(plugin) {
                        Ok(plugin) => plugin,
                        Err(_) => anyhow::bail!("Failed to parse plugin: {:?}", plugins),
                    };

                    vec.push(plugin);
                }

                vec
            }
            None => unreachable!(),
        };

        let processes_to_draw = match cli.value_of("processes") {
            Some(processes) => Some(
                Config::parse_processes(String::from(processes))
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
            None => Some(rrdtool::Rrdtool::COLORS.len()),
        };

        let processes = match plugins.contains(&rrdtool::Plugins::Processes) {
            true => Some(processes::ProcessesData::new(
                max_processes.unwrap(),
                processes_to_draw,
            )),
            false => None,
        };

        let plugins_config = PluginsConfig {
            plugins: plugins,
            processes: processes,
        };

        Ok(Config {
            input_dir: Path::new(input),
            output_filename: output,
            width: width,
            height: height,
            start: start,
            end: end,
            plugins_config: plugins_config,
        })
    }

    /// Parsing descriptive timespan to UNIX timestamp, e.g.:
    /// - last 5 minutes
    /// - last 20 hours
    /// - last hour
    /// - last minute
    /// - last 30 seconds
    /// - last day
    fn parse_timespan(mut timespan: String) -> anyhow::Result<(u64, u64)> {
        if !timespan.is_ascii() {
            return Err(anyhow!(format!(
                "Timespan contains non ASCII characters: {}",
                timespan
            )));
        }

        timespan.make_ascii_lowercase();

        match timespan.starts_with("last ") {
            true => {
                let words: Vec<&str> = timespan.split(" ").collect();

                if words.len() < 2 {
                    return Err(anyhow!(format!(
                        "Find only one word in timespan: {}",
                        timespan
                    )));
                }

                // String may or may not contain number in second word, e.g. last 5 minutes or last minute
                let mut index = 1;
                let number = match u64::from_str(words[index]) {
                    Ok(number) => {
                        index = index + 1;
                        number
                    }
                    Err(_) => 1,
                };

                let multiplier = match words[index] {
                    "second" | "seconds" => 1,
                    "minute" | "minutes" => 60,
                    "hour" | "hours" => 3600,
                    "day" | "days" => 86400,
                    "week" | "weeks" => 604800,
                    "month" | "months" => 2592000,
                    "year" | "years" => 31536000,
                    _ => {
                        return Err(anyhow!(format!(
                            "Didn't recognize time unit in timespan: {}",
                            timespan
                        )))
                    }
                };

                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                Ok((now - (number * multiplier), now))
            }
            false => {
                return Err(anyhow!(format!(
                    "Unrecognized string in timespan: {}",
                    timespan
                )));
            }
        }
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
    use super::*;
    use anyhow::Result;
    use std::time::SystemTime;

    #[test]
    pub fn parse_timespan_error() -> Result<()> {
        let res = Config::parse_timespan(String::from("lasts 5 minutes"));
        assert!(res.is_err());

        Ok(())
    }

    #[test]
    pub fn parse_timespan_ok_last_5_minutes() -> Result<()> {
        let (start, end) = Config::parse_timespan(String::from("last 5 minutes")).unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(301 >= (now - start));
        assert_eq!(300, end - start);

        Ok(())
    }

    #[test]
    pub fn parse_timespan_ok_last_week() -> Result<()> {
        let (start, end) = Config::parse_timespan(String::from("last week")).unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(604801 >= (now - start));
        assert_eq!(604800, end - start);

        Ok(())
    }

    #[test]
    pub fn parse_timespan_ok_last_10_days() -> Result<()> {
        let (start, end) = Config::parse_timespan(String::from("last 10 days")).unwrap();

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(864001 >= (now - start));
        assert_eq!(864000, end - start);

        Ok(())
    }

    #[test]
    pub fn parse_processes_3_processes() -> Result<()> {
        let mut processes = Config::parse_processes(String::from("firefox,chrome,dolphin"))?;

        processes.sort();
        assert_eq!(vec!("chrome", "dolphin", "firefox"), processes);

        Ok(())
    }
}
