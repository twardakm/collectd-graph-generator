use super::rrdtool;
use anyhow::{anyhow, Context};
use rrdtool::rrdtool::Plugins;
use std::any::Any;
use std::collections::HashMap;
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
    /// Map of plugins data
    pub data: HashMap<rrdtool::rrdtool::Plugins, Box<dyn Any + 'static>>,
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
                Config::get_vec_of_type_from_cli::<rrdtool::rrdtool::Plugins>(plugins).unwrap()
            }
            None => unreachable!(),
        };

        let mut plugins_config = PluginsConfig {
            data: HashMap::new(),
        };

        for plugin in plugins.iter() {
            match plugin {
                Plugins::Memory => plugins_config
                    .data
                    .insert(
                        *plugin,
                        Box::new(
                            Config::get_memory_data(cli, &plugins)
                                .context("Failed to get memory data")?,
                        ),
                    )
                    .context("Failed to insert memory data into map")?,
                Plugins::Processes => plugins_config
                    .data
                    .insert(
                        *plugin,
                        Box::new(
                            Config::get_processes_data(cli, &plugins)
                                .context("Failed to get processes data")?,
                        ),
                    )
                    .context("Failed to insert processes data into map")?,
            };
        }

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

    pub fn get_vec_of_type_from_cli<T>(args: &'a str) -> anyhow::Result<Vec<T>>
    where
        T: FromStr,
        <T as std::str::FromStr>::Err: std::fmt::Debug,
    {
        Ok(args
            .split(",")
            .collect::<Vec<&str>>()
            .iter()
            .map(|arg| T::from_str(arg).unwrap())
            .collect::<Vec<T>>())
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
    pub fn get_plugins_from_cli() -> Result<()> {
        let plugins =
            Config::get_vec_of_type_from_cli::<rrdtool::rrdtool::Plugins>("processes,memory")
                .unwrap();

        assert_eq!(2, plugins.len());

        assert!(plugins.contains(&rrdtool::rrdtool::Plugins::Processes));
        assert!(plugins.contains(&rrdtool::rrdtool::Plugins::Memory));

        Ok(())
    }
}
