use super::rrdtool;

use anyhow::{Context, Result};
use std::process::Command;

/// Get list of remote files
///
/// # Arguments
/// * `dir` - path of remote directory
/// * `username` - username to SSH login
/// * `hostname` - hostname of remote target
///
pub fn ls(dir: &str, username: &str, hostname: &str) -> Result<Vec<String>> {
    let network_address = String::from(username) + "@" + hostname;

    let output = Command::new("ssh")
        .args(&[&network_address, &String::from("ls"), &String::from(dir)])
        .output()
        .context("Failed to execute SSH")?;

    if !output.status.success() {
        rrdtool::print_process_command_output(output);

        anyhow::bail!(
            "Failed to list remote directories in {}:{}!",
            network_address,
            dir
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| String::from(s))
        .collect::<Vec<String>>())
}

#[cfg(test)]
pub mod tests {
    use anyhow::Result;
    use std::fs::{create_dir, File};
    use tempfile::TempDir;

    #[test]
    fn ls() -> Result<()> {
        let dir = TempDir::new().unwrap();

        let dirs = vec![
            dir.path().join("some_directory"),
            dir.path().join("some other directory"),
        ];

        for dir in dirs {
            if !dir.exists() {
                create_dir(dir)?;
            }
        }

        let _files = vec![
            File::create(dir.path().join("some_file.rrd"))?,
            File::create(dir.path().join("some other file.rrd"))?,
        ];

        let res = super::ls(
            dir.path().to_str().unwrap(),
            &whoami::username(),
            "localhost",
        );

        let res_nok = super::ls(dir.path().to_str().unwrap(), &whoami::username(), "local");

        assert!(res.is_ok());
        assert!(res_nok.is_err());

        let mut res = res.unwrap();
        res.sort();
        assert_eq!(4, res.len());

        assert_eq!("some other directory", res[0]);
        assert_eq!("some other file.rrd", res[1]);
        assert_eq!("some_directory", res[2]);
        assert_eq!("some_file.rrd", res[3]);

        Ok(())
    }
}
