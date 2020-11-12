use super::memory_data::MemoryData;
use super::memory_type::MemoryType;
use super::rrdtool::{Plugin, Rrdtool};

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

        let mut graph_args = Vec::new();

        for i in 0..data.memory_types.len() {
            let mut args = get_graph_args_for_memory_type(i, &memory_dir, &data.memory_types[i])
                .context("Failed to generate graph arguments")?;
            graph_args.append(&mut args);
        }

        self.graph_args.push(Vec::new());
        let len = self.graph_args.len() - 1;
        self.graph_args[len].append(&mut graph_args);

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

fn get_graph_args_for_memory_type<'a>(
    iter: usize,
    memory_dir: &'a Path,
    memory_type: &MemoryType,
) -> Result<Vec<String>> {
    debug!("Generating arguments for {:?}", memory_type);

    let mut graph_args = Vec::new();

    graph_args.push(
        String::from("DEF:")
            + &memory_type.to_string()
            + "="
            + memory_dir.join(memory_type.to_filename()).to_str().unwrap()
            + ":value:AVERAGE",
    );

    graph_args.push(
        String::from("LINE5:")
            + &memory_type.to_string()
            + Rrdtool::COLORS[iter]
            + ":\""
            + &memory_type.to_string()
            + "\"",
    );

    trace!("Generated arguments: {:?}", graph_args);

    Ok(graph_args)
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

    #[test]
    fn get_graph_args_for_memory_type() -> Result<()> {
        let free = super::get_graph_args_for_memory_type(
            0,
            Path::new("/some/path/to/memory"),
            &MemoryType::Free,
        )?;
        let cached = super::get_graph_args_for_memory_type(
            1,
            Path::new("/some/path/to/memory"),
            &MemoryType::Cached,
        )?;

        assert_eq!(
            "DEF:free=/some/path/to/memory/memory-free.rrd:value:AVERAGE",
            free[0]
        );
        assert_eq!("LINE5:free#e6194b:\"free\"", free[1]);

        assert_eq!(
            "DEF:cached=/some/path/to/memory/memory-cached.rrd:value:AVERAGE",
            cached[0]
        );
        assert_eq!("LINE5:cached#3cb44b:\"cached\"", cached[1]);

        Ok(())
    }
}
