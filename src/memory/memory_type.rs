use super::super::config;
use anyhow::Result;
use std::str::FromStr;
use std::string::ToString;

/// Collectd collects multiple types of memory used by operating system
/// This enum allows to choose which one should be drawn on a graph
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MemoryType {
    Buffered,
    Cached,
    Free,
    SlabRecl,
    SlabUnrecl,
    Used,
}

impl MemoryType {
    /// Returns filename used to store data for particular memory type
    ///
    /// # Examples
    ///
    /// ```
    /// use cgg::memory::memory_type::MemoryType;
    ///
    /// let filename = MemoryType::SlabRecl.to_filename();
    ///
    /// assert_eq!("memory-slab_recl.rrd", filename);
    /// ```
    ///
    pub fn to_filename(&self) -> &str {
        match self {
            MemoryType::Buffered => "memory-buffered.rrd",
            MemoryType::Cached => "memory-cached.rrd",
            MemoryType::Free => "memory-free.rrd",
            MemoryType::SlabRecl => "memory-slab_recl.rrd",
            MemoryType::SlabUnrecl => "memory-slab_unrecl.rrd",
            MemoryType::Used => "memory-used.rrd",
        }
    }
}

/// Returns [`MemoryType`] from str, which allows to convert command line arguments
/// to appropriate struct
impl FromStr for MemoryType {
    type Err = ();

    fn from_str(input: &str) -> Result<MemoryType, Self::Err> {
        match input {
            "buffered" => Ok(MemoryType::Buffered),
            "cached" => Ok(MemoryType::Cached),
            "free" => Ok(MemoryType::Free),
            "slab_recl" => Ok(MemoryType::SlabRecl),
            "slab_unrecl" => Ok(MemoryType::SlabUnrecl),
            "used" => Ok(MemoryType::Used),
            _ => Err(()),
        }
    }
}

/// Converts [`MemoryType`] to descriptive string which is used as a legend on a graphs
impl ToString for MemoryType {
    fn to_string(&self) -> String {
        String::from(match self {
            MemoryType::Buffered => "buffered",
            MemoryType::Cached => "cached",
            MemoryType::Free => "free",
            MemoryType::SlabRecl => "slab_recl",
            MemoryType::SlabUnrecl => "slab_unrecl",
            MemoryType::Used => "used",
        })
    }
}

impl<'a> config::Config<'a> {
    /// Returs vector of [`MemoryType`] from command line arguments.
    /// User may want to draw only chosen memory types.
    ///
    /// # Arguments
    /// * `cli` - A reference to [`clap::ArgMatches`] to get data from user
    ///
    pub fn get_memory_types(cli: &'a clap::ArgMatches) -> Result<Vec<MemoryType>> {
        match cli.value_of("memory") {
            Some(value) => config::Config::get_vec_of_type_from_cli::<MemoryType>(value),
            None => anyhow::bail!("Didn't find memory in command line"),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn memory_type_string_conversion() -> Result<()> {
        assert!(MemoryType::Buffered == MemoryType::from_str("buffered").unwrap());
        assert!(MemoryType::Cached == MemoryType::from_str("cached").unwrap());
        assert!(MemoryType::Free == MemoryType::from_str("free").unwrap());
        assert!(MemoryType::SlabRecl == MemoryType::from_str("slab_recl").unwrap());
        assert!(MemoryType::SlabUnrecl == MemoryType::from_str("slab_unrecl").unwrap());
        assert!(MemoryType::Used == MemoryType::from_str("used").unwrap());

        assert!(MemoryType::from_str("some other").is_err());
        Ok(())
    }

    #[test]
    fn memory_type_file_names() -> Result<()> {
        assert!(&MemoryType::Buffered
            .to_filename()
            .contains(&MemoryType::Buffered.to_string()));

        assert!(&MemoryType::Cached
            .to_filename()
            .contains(&MemoryType::Cached.to_string()));

        assert!(&MemoryType::Free
            .to_filename()
            .contains(&MemoryType::Free.to_string()));

        assert!(&MemoryType::SlabRecl
            .to_filename()
            .contains(&MemoryType::SlabRecl.to_string()));

        assert!(&MemoryType::SlabUnrecl
            .to_filename()
            .contains(&MemoryType::SlabUnrecl.to_string()));

        assert!(&MemoryType::Used
            .to_filename()
            .contains(&MemoryType::Used.to_string()));

        Ok(())
    }
}
