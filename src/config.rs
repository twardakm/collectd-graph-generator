use anyhow::{anyhow, Context};
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

/// Struct with all available options
pub struct Config<'a> {
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

        Ok(Config {
            input_dir: Path::new(input),
            output_filename: output,
            width: width,
            height: height,
            start: start,
            end: end,
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
}
