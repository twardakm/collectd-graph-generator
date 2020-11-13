use super::super::config;
use super::memory_type::MemoryType;
use super::rrdtool::rrdtool::Plugins;
use anyhow::{Context, Result};

#[derive(Debug)]
pub struct MemoryData {
    /// Types of data to visualize on graph
    pub memory_types: Vec<MemoryType>,
}

impl MemoryData {
    pub fn new(memory_types: Vec<MemoryType>) -> MemoryData {
        MemoryData {
            memory_types: memory_types,
        }
    }
}

impl<'a> config::Config<'a> {
    pub fn get_memory_data(
        cli: &'a clap::ArgMatches,
        plugins: &Vec<Plugins>,
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
