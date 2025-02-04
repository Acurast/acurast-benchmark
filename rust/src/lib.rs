use cpu::{crypto, math, sort};

#[cfg(any(target_arch= "arm", target_arch="aarch64"))]
pub mod arm;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod ffi;

mod cpu;

mod utils;

pub(crate) struct CpuFeatures {
    num_cores: usize,
    
    sve: bool,
    i8mm: bool,
}

pub struct Bench {
    features: CpuFeatures,
}

impl Bench {
    pub(crate) fn with_features(features: CpuFeatures) -> Self {
        Self { features }
    }

    pub fn cpu(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        let crypto_report = crypto::bench(&self.features, config.crypto).map_err(|err| cpu::Error::Crypto(err))?;
        let math_report = math::bench(&self.features, config.math).map_err(|err| cpu::Error::Math(err))?;
        let sort_report = sort::bench(&self.features, config.sort).map_err(|err| cpu::Error::Sort(err))?;

        Ok(cpu::Report { crypto: crypto_report, math: math_report, sort: sort_report })
    }

    pub fn cpu_multithread(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        let crypto_report = crypto::bench_multithread(&self.features, config.crypto).map_err(|err| cpu::Error::Crypto(err))?;
        let math_report = math::bench_multithread(&self.features, config.math).map_err(|err| cpu::Error::Math(err))?;
        let sort_report = sort::bench_multithread(&self.features, config.sort).map_err(|err| cpu::Error::Sort(err))?;

        Ok(cpu::Report { crypto: crypto_report, math: math_report, sort: sort_report })
    }
}
