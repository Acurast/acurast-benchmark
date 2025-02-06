#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod arm;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod ffi;

mod cpu;
mod ram;
mod storage;

mod utils;

pub(crate) struct CpuFeatures {
    num_cores: usize,

    sve: bool,
    i8mm: bool,
}

pub struct Bench {
    total_ram: u64,
    avail_storage: u64,
    features: CpuFeatures,
}

impl Bench {
    pub(crate) fn with_features(total_ram: u64, avail_storage: u64, features: CpuFeatures) -> Self {
        Self {
            total_ram,
            avail_storage,
            features,
        }
    }

    pub fn cpu(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        let crypto_report = cpu::crypto::bench(&self.features, config.crypto)
            .map_err(|err| cpu::Error::Crypto(err))?;
        let math_report =
            cpu::math::bench(&self.features, config.math).map_err(|err| cpu::Error::Math(err))?;
        let sort_report =
            cpu::sort::bench(&self.features, config.sort).map_err(|err| cpu::Error::Sort(err))?;

        Ok(cpu::Report {
            crypto: crypto_report,
            math: math_report,
            sort: sort_report,
        })
    }

    pub fn cpu_multithread(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        let crypto_report = cpu::crypto::bench_multithread(&self.features, config.crypto)
            .map_err(|err| cpu::Error::Crypto(err))?;
        let math_report = cpu::math::bench_multithread(&self.features, config.math)
            .map_err(|err| cpu::Error::Math(err))?;
        let sort_report = cpu::sort::bench_multithread(&self.features, config.sort)
            .map_err(|err| cpu::Error::Sort(err))?;

        Ok(cpu::Report {
            crypto: crypto_report,
            math: math_report,
            sort: sort_report,
        })
    }

    pub fn ram(&self, config: ram::Config) -> Result<ram::Report, ram::Error> {
        let alloc_report = ram::alloc::bench(config.alloc).map_err(|err| ram::Error::Alloc(err))?;
        let access_report = ram::access::bench(&self.features, config.access)
            .map_err(|err| ram::Error::Access(err))?;

        Ok(ram::Report {
            total_mem: self.total_ram,
            alloc: alloc_report,
            access: access_report,
        })
    }

    pub fn storage(&self, config: storage::Config) -> Result<storage::Report, storage::Error> {
        let access_report = storage::access::bench(&self.features, config.access)
            .map_err(|err| storage::Error::Access(err))?;

        Ok(storage::Report {
            avail_storage: self.avail_storage,
            access: access_report,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::utils::GB;

    use super::*;

    #[test]
    fn test_cpu() {
        let bench = Bench::with_features(
            0,
            0,
            CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
        );
        let duration = Duration::from_secs(1);
        let result = bench.cpu(cpu::Config {
            crypto: cpu::crypto::Config {
                duration,
                data_len: 64,
                ..Default::default()
            },
            math: cpu::math::Config {
                duration,
                n: 10,
                ..Default::default()
            },
            sort: cpu::sort::Config {
                duration,
                item_len: 25,
                data_len: 100_000,
                ..Default::default()
            },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.crypto.tps > 0.);
        assert!(result.math.tps > 0.);
        assert!(result.sort.tps > 0.);

        println!("{result}");
    }

    #[test]
    fn test_cpu_multithread() {
        let bench = Bench::with_features(
            0,
            0,
            CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
        );
        let duration = Duration::from_secs(1);
        let result = bench.cpu_multithread(cpu::Config {
            crypto: cpu::crypto::Config {
                duration,
                data_len: 64,
                ..Default::default()
            },
            math: cpu::math::Config {
                duration,
                n: 10,
                ..Default::default()
            },
            sort: cpu::sort::Config {
                duration,
                item_len: 25,
                data_len: 100_000,
                ..Default::default()
            },
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
        let bench = Bench::with_features(
            16 * GB as u64,
            0,
            CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
        );
        let result = bench.ram(ram::Config {
            alloc: ram::alloc::Config {
                data_len: 64,
                iters: 5,
                ..Default::default()
            },
            access: ram::access::Config {
                data_len: 64,
                iters: 5,
                ..Default::default()
            },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.alloc.avg_t > Duration::ZERO);

        println!("{result}");
    }

    #[test]
    fn test_storage() {
        let bench = Bench::with_features(
            0,
            72 * GB as u64,
            CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
        );
        let result = bench.storage(storage::Config {
            access: storage::access::Config {
                data_len_mb: 1,
                iters: 1,
                ..Default::default()
            },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();

        println!("{result}");
    }
}
