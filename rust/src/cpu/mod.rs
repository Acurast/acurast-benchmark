use std::{fmt, rc::Rc};

use crate::{macros::*, CpuFeatures};

pub(crate) mod crypto;
pub(crate) mod math;
pub(crate) mod sort;

pub struct Bench {
    features: Rc<CpuFeatures>,
}

impl Bench {
    pub(crate) fn new(features: Rc<CpuFeatures>) -> Self {
        Self { features }
    }

    fn_bench!(crypto);
    fn_bench!(math);
    fn_bench!(sort);

    pub(crate) fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report { 
            crypto: self.crypto(config.crypto).map_err(Error::Crypto)?,
            math: self.math(config.math).map_err(Error::Math)?, 
            sort: self.sort(config.sort).map_err(Error::Sort)?,
        })
    }

    fn_bench_multithread!(crypto, crypto_multithread);
    fn_bench_multithread!(math, math_multithread);
    fn_bench_multithread!(sort, sort_multithread);

    pub(crate) fn all_multithread(&self, config: Config) -> Result<Report, Error> {
        Ok(Report { 
            crypto: self.crypto_multithread(config.crypto).map_err(Error::Crypto)?,
            math: self.math_multithread(config.math).map_err(Error::Math)?, 
            sort: self.sort_multithread(config.sort).map_err(Error::Sort)?,
        })
    }
}

#[derive(Default)]
pub struct Config {
    pub crypto: crypto::Config,
    pub math: math::Config,
    pub sort: sort::Config,
}

pub struct Report {
    pub crypto: crypto::Report,
    pub math: math::Report,
    pub sort: sort::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "CPU")?;
        writeln!(f, "{i} {}", self.crypto)?;
        writeln!(f, "{i} {}", self.math)?;
        write!(f, "{i} {}", self.sort)?;

        Ok(())
    }
}


#[derive(Debug)]
pub enum Error {
    Crypto(crypto::Error),
    Math(math::Error),
    Sort(sort::Error),
}