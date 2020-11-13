use super::memory_data::MemoryData;
use super::memory_type::MemoryType;
use super::rrdtool::rrdtool::{Plugin, Rrdtool};

use std::path::Path;

use anyhow::{bail, Context, Result};
use log::{debug, trace};

impl Plugin<&MemoryData> for Rrdtool {
    fn enter_plugin(&mut self, data: &MemoryData) -> Result<&mut Self> {
        debug!("Memory plugin entry point");
        trace!("Memory plugin: {:?}", data);

        let memory_dir = Path::new(self.input_dir.as_str()).join("memory");

        verify_data_files_exist(&memory_dir, &data.memory_types)
            .context("Unable to find expected files")?;

        trace!("All expected files exist");

        self.graph_args.new_graph();

        for i in 0..data.memory_types.len() {
            self.graph_args.push(
                data.memory_types[i].to_string().as_str(),
                Rrdtool::COLORS[i],
                5,
                memory_dir
                    .join(data.memory_types[i].to_filename())
                    .to_str()
                    .unwrap(),
            );
        }

        trace!("Memory plugin exit");

        Ok(self)
    }
}

fn verify_data_files_exist<'a>(memory_dir: &'a Path, memory_types: &Vec<MemoryType>) -> Result<()> {
    match memory_types
        .iter()
        .map(|memory_type| memory_dir.join(memory_type.to_filename()).exists())
        .all(|element| element == true)
    {
        true => Ok(()),
        false => bail!(
            "Some file for memory measurements doesn't exist in {}",
            memory_dir.to_str().unwrap()
        ),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fs::{create_dir, File};
    use tempfile::TempDir;

    #[test]
    fn verify_data_files_exist() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let mem_path = temp.path().join("memory");
        if !mem_path.exists() {
            create_dir(&mem_path)?;
        }

        let _files = vec![
            File::create(mem_path.join("memory-cached.rrd"))?,
            File::create(mem_path.join("memory-free.rrd"))?,
            File::create(mem_path.join("memory-used.rrd"))?,
        ];

        let memory_types_ok = vec![MemoryType::Free, MemoryType::Cached, MemoryType::Used];
        let memory_types_nok = vec![MemoryType::Used, MemoryType::SlabRecl];

        let memory_types_ok = super::verify_data_files_exist(&mem_path, &memory_types_ok);
        let memory_types_nok = super::verify_data_files_exist(&mem_path, &memory_types_nok);

        assert!(memory_types_ok.is_ok());
        assert!(memory_types_nok.is_err());

        Ok(())
    }
}
