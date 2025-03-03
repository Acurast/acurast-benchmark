use core::fmt;
use std::rc::Rc;

use crate::{macros::*, utils::GB, CpuFeatures};

pub(crate) mod access;

pub struct Bench {
    features: Rc<CpuFeatures>,
    avail_storage: u64,
}

impl Bench {
    pub(crate) fn new(features: Rc<CpuFeatures>, avail_storage: u64) -> Self {
        Self { features, avail_storage }
    }

    pub fn avail_storage(&self) -> u64 {
        self.avail_storage
    }

    fn_bench!(access);

    pub(crate) fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report {
            avail_storage: self.avail_storage,
            access: self.access(config.access).map_err(Error::Access)?,
        })
    }
}

#[derive(Default)]
pub struct Config {
    pub access: access::Config,
}

pub struct Report {
    pub avail_storage: u64,
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "Storage")?;
        writeln!(
            f,
            "{i} available storage {:.2} GB",
            (self.avail_storage as f64) / GB as f64,
        )?;
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