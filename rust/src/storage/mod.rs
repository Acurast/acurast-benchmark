use core::fmt;

use crate::{macros::*, CpuFeatures};

pub(crate) mod access;

pub struct Bench {
    features: CpuFeatures,
}

impl Bench {
    pub(crate) fn new(features: CpuFeatures) -> Self {
        Self { features }
    }

    fn_bench!(access);

    pub(crate) fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report {
            access: self.access(config.access).map_err(Error::Access)?,
        })
    }
}

#[derive(Default)]
pub struct Config {
    pub access: access::Config,
}

pub struct Report {
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "Storage")?;
        write!(f, "{i} {}", self.access)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Access(access::Error),
}

impl Error {
    pub fn access(err: access::Error) -> Error {
        Self::Access(err)
    }
}