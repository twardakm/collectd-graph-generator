use super::super::config;
use super::memory_type::MemoryType;
use super::rrdtool::common::Plugins;
use anyhow::{Context, Result};

/// Data used by memory plugin
///
/// # Examples
///
/// ```
/// use cgg::memory::{memory_data::MemoryData, memory_type::MemoryType};
///
/// let memory_data = MemoryData::new(vec![MemoryType::Buffered, MemoryType::Free]);
/// ```
///
#[derive(Debug, Clone)]
pub struct MemoryData {
    /// Types of data to visualize on graph
    pub memory_types: Vec<MemoryType>,
}

impl MemoryData {
    pub fn new(memory_types: Vec<MemoryType>) -> MemoryData {
        MemoryData { memory_types }
    }
}

impl<'a> config::Config<'a> {
    /// Returns [`MemoryData`] structure with all data needed by memory plugin
    ///
    /// # Arguments
    /// * `cli` - A reference to [`clap::ArgMatches`] to get data from user
    /// * `plugins` - Vector of plugins already read from command line
    ///
    pub fn get_memory_data(
        cli: &'a clap::ArgMatches,
        plugins: &[Plugins],
    ) -> Result<Option<MemoryData>> {
        Ok(match plugins.contains(&Plugins::Memory) {
            true => Some(MemoryData::new(
                config::Config::get_memory_types(cli)
                    .context("Failed to get memory types to draw")?,
            )),
            false => None,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use super::super::super::config;
    use super::*;

    #[test]
    fn get_memory_data_nok() -> Result<()> {
        let cli = clap::ArgMatches::default();
        let plugins = vec![Plugins::Processes];

        let config = config::Config::get_memory_data(&cli, &plugins)?;

        let res = match config {
            Some(_) => Err(()),
            None => Ok(()),
        };

        assert_eq!(Ok(()), res);

        let plugins = vec![Plugins::Memory];

        let config = config::Config::get_memory_data(&cli, &plugins);

        assert!(config.is_err());

        Ok(())
    }
}
