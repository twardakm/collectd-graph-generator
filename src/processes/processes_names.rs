use super::rrdtool::common::Target;
use super::rrdtool::remote;

use anyhow::{Context, Result};
use log::trace;

use std::fs::read_dir;

/// Parse collectd results directory to get names of analysed processes
///
/// # Arguments
/// * `target` - [`Target`] enum describing, whether local or remote directory is provided
/// * `input_dir` - path to local or remote directory
/// * `username` - username to login in case of remote directory
/// * `hostname` - hostname to use in case of remote directory
///
pub fn get<'a>(
    target: Target,
    input_dir: &'a str,
    username: &Option<String>,
    hostname: &Option<String>,
) -> Result<Vec<String>> {
    match target {
        Target::Local => get_from_local(input_dir),
        Target::Remote => get_from_remote(input_dir, username, hostname),
    }
}

/// Get processes names from local directory
fn get_from_local(input_dir: &str) -> Result<Vec<String>> {
    let paths = read_dir(input_dir).context(format!("Failed to read directory: {}", input_dir))?;

    let processes = paths
        .filter_map(|path| {
            path.ok().and_then(|path| {
                path.path().file_name().and_then(|name| {
                    name.to_str()
                        .and_then(|s| s.strip_prefix("processes-"))
                        .map(String::from)
                })
            })
        })
        .collect::<Vec<String>>();

    Ok(processes)
}

/// Get processes names from remote directory via SSH and ls commands
fn get_from_remote<'a>(
    input_dir: &'a str,
    username: &Option<String>,
    hostname: &Option<String>,
) -> Result<Vec<String>> {
    let paths = remote::ls(
        input_dir,
        username.as_ref().unwrap(),
        hostname.as_ref().unwrap(),
    )
    .context(format!("Failed to read remote directory {}", input_dir))?;

    let processes = paths
        .iter()
        .filter_map(|path| path.strip_prefix("processes-"))
        .map(String::from)
        .collect::<Vec<String>>();

    trace!("Listed processes from remote directory: {:?}", processes);

    Ok(processes)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    use anyhow::Result;
    use std::fs::{create_dir, remove_dir};
    use std::path::Path;
    use tempfile::TempDir;
    #[test]
    pub fn rrdtool_get_processes_names_from_directory_local() -> Result<()> {
        let temp = TempDir::new().unwrap();

        let paths = vec![
            temp.path().join("processes-firefox"),
            temp.path().join("processes-chrome"),
            temp.path().join("processes-dolphin"),
            temp.path().join("processes-rust language server"),
        ];

        for path in &paths {
            if !path.exists() {
                create_dir(path)?;
            }
        }

        let mut processes = super::get(Target::Local, temp.path().to_str().unwrap(), &None, &None)?;

        processes.sort();
        assert_eq!(4, processes.len());
        assert_eq!("chrome", processes[0]);
        assert_eq!("dolphin", processes[1]);
        assert_eq!("firefox", processes[2]);
        assert_eq!("rust language server", processes[3]);

        for path in &paths {
            if path.exists() {
                remove_dir(path)?;
            }
        }

        Ok(())
    }

    #[test]
    pub fn rrdtool_get_processes_names_from_remote_directory_network_hostname() -> Result<()> {
        let processes = vec!["chrome", "dolphin", "firefox"];
        let temp = TempDir::new().unwrap();

        for process in &processes {
            create_dir(Path::new(temp.path()).join(String::from("processes-") + process))?;
        }

        let mut found_processes = super::get(
            Target::Remote,
            temp.path().to_str().unwrap(),
            &Some(whoami::username()),
            &Some(String::from("localhost")),
        )?;

        found_processes.sort();
        assert_eq!(3, found_processes.len());
        assert_eq!("chrome", found_processes[0]);
        assert_eq!("dolphin", found_processes[1]);
        assert_eq!("firefox", found_processes[2]);

        for process in processes {
            remove_dir(Path::new(temp.path()).join(String::from("processes-") + process))?;
        }

        Ok(())
    }
}
