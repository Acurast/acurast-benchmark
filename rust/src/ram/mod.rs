use core::fmt;

use crate::{macros::*, CpuFeatures};

pub(crate) mod access;
pub(crate) mod alloc;

pub struct Bench {
    features: CpuFeatures,
}

impl Bench {
    pub(crate) fn new(features: CpuFeatures) -> Self {
        Self { features }
    }

    fn_bench!(access);
    fn_bench!(alloc);

    pub(crate) fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report {
            alloc: self.alloc(config.alloc).map_err(Error::Alloc)?,
            access: self.access(config.access).map_err(Error::Access)?,
        })
    }
}

#[derive(Default)]
pub struct Config {
    pub alloc: alloc::Config,
    pub access: access::Config,
}

pub struct Report {
    pub alloc: alloc::Report,
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "RAM")?;
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
