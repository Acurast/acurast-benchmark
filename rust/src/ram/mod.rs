use core::fmt;
use std::rc::Rc;

use crate::{macros::*, utils::MB, CpuFeatures};

pub(crate) mod access;
pub(crate) mod alloc;

pub(crate) struct Bench {
    features: Rc<CpuFeatures>,
    total_mem: u64,
}

impl Bench {
    pub(crate) fn new(features: Rc<CpuFeatures>, total_mem: u64) -> Self {
        Self { features, total_mem }
    }

    pub fn total_mem(&self) -> u64 {
        self.total_mem
    }

    fn_bench!(access);
    fn_bench!(alloc);

    pub(crate) fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report {
            total_mem: self.total_mem,
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
