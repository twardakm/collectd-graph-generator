use super::rrdtool::{Plugin, Rrdtool};

use anyhow::Result;
use log::debug;

#[derive(Debug)]
pub struct MemoryData {
    /// Dummy placeholder for future reference
    dummy: String,
}

impl MemoryData {
    pub fn new(dummy: String) -> MemoryData {
        MemoryData { dummy: dummy }
    }
}

impl Plugin<MemoryData> for Rrdtool {
    fn enter_plugin(&mut self, data: MemoryData) -> Result<&mut Self> {
        debug!("TRAIT: {:?}", data);
        Ok(self)
    }
}
