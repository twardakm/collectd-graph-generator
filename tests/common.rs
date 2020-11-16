use std::path::PathBuf;
use tempfile::TempDir;

/// Initialize tests and returns handle to temporary directory
pub fn init() -> std::io::Result<TempDir> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("error"))
        .format_timestamp(None)
        .try_init();

    TempDir::new()
}

/// Returns path to collectd-graph-generator executable
pub fn get_cgg_exec_path() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_dir()?.join("target/debug/cgg"))
}
