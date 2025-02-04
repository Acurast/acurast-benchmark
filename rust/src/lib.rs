use cpu::{crypto, math, sort};
use ram::{access, alloc};

#[cfg(any(target_arch= "arm", target_arch="aarch64"))]
pub mod arm;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod ffi;

mod cpu;
mod ram;

mod utils;

pub(crate) struct CpuFeatures {
    num_cores: usize,
    
    sve: bool,
    i8mm: bool,
}

pub struct Bench {
    total_ram: u64,
    features: CpuFeatures,
}

impl Bench {
    pub(crate) fn with_features(total_ram: u64, features: CpuFeatures) -> Self {
        Self { total_ram, features }
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

    pub fn ram(&self, config: ram::Config) -> Result<ram::Report, ram::Error> {
        let alloc_report = alloc::bench(config.alloc).map_err(|err| ram::Error::Alloc(err))?;
        let access_report = access::bench(&self.features, config.access).map_err(|err| ram::Error::Access(err))?;

        Ok(ram::Report { total_mem: self.total_ram, alloc: alloc_report, access: access_report })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_cpu() {
        let bench = Bench::with_features(0, CpuFeatures { num_cores: 1, sve: false, i8mm: false });
        let duration = Duration::from_secs(1);
        let result = bench.cpu(cpu::Config { 
            crypto: crypto::Config { duration, data_len: 64, ..Default::default() },
            math: math::Config { duration, n: 10, ..Default::default() },
            sort: sort::Config { duration, item_len: 5, data_len: 10, ..Default::default() },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.crypto.tps > 0.);
        assert!(result.math.tps > 0.);
        assert!(result.sort.tps > 0.);

        println!("{result}");
    }

    #[test]
    fn test_ram() {
        let bench = Bench::with_features(16 * 1024 * 1024 * 1024, CpuFeatures { num_cores: 1, sve: false, i8mm: false });
        let result = bench.ram(ram::Config {
            alloc: alloc::Config { data_len: 64, iters: 5, ..Default::default() },
            access: access::Config { data_len: 64, iters: 5, ..Default::default() },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.alloc.avg_t > Duration::ZERO);

        println!("{result}");
    }
}
