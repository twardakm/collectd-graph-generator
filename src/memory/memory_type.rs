use super::super::config;
use anyhow::Result;
use std::str::FromStr;
use std::string::ToString;

/// Type of system memory to draw on graph
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
    pub fn get_memory_types(cli: &'a clap::ArgMatches) -> Result<Vec<MemoryType>> {
        config::Config::get_vec_of_type_from_cli::<MemoryType>(cli.value_of("memory").unwrap())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn memory_type_from_str() -> Result<()> {
        assert!(MemoryType::Cached == MemoryType::from_str("cached").unwrap());
        assert!(MemoryType::Free == MemoryType::from_str("free").unwrap());
        assert!(MemoryType::SlabRecl == MemoryType::from_str("slab_recl").unwrap());
        assert!(MemoryType::SlabUnrecl == MemoryType::from_str("slab_unrecl").unwrap());
        assert!(MemoryType::Used == MemoryType::from_str("used").unwrap());

        assert!(MemoryType::from_str("some other").is_err());
        Ok(())
    }
}
