use super::memory_data::MemoryData;
use super::memory_type::MemoryType;
use super::rrdtool::remote;
use super::rrdtool::rrdtool::{Plugin, Rrdtool, Target};

use std::path::Path;

use anyhow::{bail, Context, Result};
use log::{debug, trace};

impl Plugin<&MemoryData> for Rrdtool {
    fn enter_plugin(&mut self, data: &MemoryData) -> Result<&mut Self> {
        debug!("Memory plugin entry point");
        trace!("Memory plugin: {:?}", data);

        let memory_dir = Path::new(self.input_dir.as_str()).join("memory");

        verify_data_files_exist(
            self.target,
            &memory_dir,
            &data.memory_types,
            &self.username,
            &self.hostname,
        )
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

fn verify_data_files_exist<'a>(
    target: Target,
    memory_dir: &'a Path,
    memory_types: &Vec<MemoryType>,
    username: &Option<String>,
    hostname: &Option<String>,
) -> Result<()> {
    match target {
        Target::Local => verify_data_files_exist_local(memory_dir, memory_types),
        Target::Remote => verify_data_files_exist_remote(
            memory_dir,
            memory_types,
            &username.as_ref().unwrap(),
            &hostname.as_ref().unwrap(),
        ),
    }
}

fn verify_data_files_exist_remote<'a>(
    memory_dir: &'a Path,
    memory_types: &Vec<MemoryType>,
    username: &str,
    hostname: &str,
) -> Result<()> {
    let files = remote::ls(memory_dir.to_str().unwrap(), username, hostname).context(format!(
        "Failed to list remote files in: {}",
        memory_dir.to_str().unwrap()
    ))?;

    match memory_types
        .iter()
        .map(|memory_type| files.contains(&String::from(memory_type.to_filename())))
        .all(|element| element == true)
    {
        true => Ok(()),
        false => bail!(
            "Some foile for memory measurements doesn't exist in {}",
            memory_dir.to_str().unwrap()
        ),
    }
}

fn verify_data_files_exist_local<'a>(
    memory_dir: &'a Path,
    memory_types: &Vec<MemoryType>,
) -> Result<()> {
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
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_temp_memory_files(temp: &TempDir) -> Result<PathBuf> {
        let mem_path = temp.path().join("memory");
        if !mem_path.exists() {
            create_dir(&mem_path)?;
        }

        let _files = vec![
            File::create(mem_path.join("memory-cached.rrd"))?,
            File::create(mem_path.join("memory-free.rrd"))?,
            File::create(mem_path.join("memory-used.rrd"))?,
        ];

        Ok(mem_path)
    }

    #[test]
    fn verify_data_files_exist_local() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let mem_path = create_temp_memory_files(&temp)?;

        let memory_types_ok = vec![MemoryType::Free, MemoryType::Cached, MemoryType::Used];
        let memory_types_nok = vec![MemoryType::Used, MemoryType::SlabRecl];

        let memory_types_ok = super::verify_data_files_exist_local(&mem_path, &memory_types_ok);
        let memory_types_nok = super::verify_data_files_exist_local(&mem_path, &memory_types_nok);

        assert!(memory_types_ok.is_ok());
        assert!(memory_types_nok.is_err());

        Ok(())
    }

    #[test]
    fn verify_data_files_exist_remote() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let mem_path = create_temp_memory_files(&temp)?;

        let memory_types_ok = vec![MemoryType::Free, MemoryType::Cached, MemoryType::Used];
        let memory_types_nok = vec![MemoryType::Used, MemoryType::SlabRecl];

        let memory_types_ok = super::verify_data_files_exist_remote(
            &mem_path,
            &memory_types_ok,
            &whoami::username(),
            "localhost",
        );

        let memory_types_nok = super::verify_data_files_exist_remote(
            &mem_path,
            &memory_types_nok,
            &whoami::username(),
            "localhost",
        );

        assert!(memory_types_ok.is_ok());
        assert!(memory_types_nok.is_err());

        Ok(())
    }
}
