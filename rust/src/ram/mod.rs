use core::fmt;

use crate::utils::MB;

pub(crate) mod access;
pub(crate) mod alloc;

#[derive(Default)]
pub struct Config {
    pub alloc: alloc::Config,
    pub access: access::Config,
}

pub struct Report {
    pub total_mem: u64,
    pub alloc: alloc::Report,
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "RAM")?;
        writeln!(f, "{i} total mem {} MB", self.total_mem / MB as u64)?;
        writeln!(f, "{i} {}", self.alloc)?;
        write!(f, "{i} {}", self.access)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Alloc(alloc::Error),
    Access(access::Error),
}
