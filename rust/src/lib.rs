use std::{fmt, rc::Rc};

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod arm;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod ffi;

mod cpu;
mod ram;
mod storage;

mod macros;
mod utils;

pub(crate) struct CpuFeatures {
    num_cores: usize,

    sve: bool,
    i8mm: bool,
}

pub struct Bench {
    features: Rc<CpuFeatures>,

    pub cpu: cpu::Bench,
    pub ram: ram::Bench,
    pub storage: storage::Bench,
}

impl Bench {
    pub(crate) fn with_features(total_ram: u64, avail_storage: u64, features: CpuFeatures) -> Self {
        let features = Rc::new(features);

        Self {
            features: features.clone(),
            cpu: cpu::Bench::new(features.clone()),
            ram: ram::Bench::new(features.clone(), total_ram),
            storage: storage::Bench::new(features, avail_storage),
        }
    }

    pub fn cpu(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        self.cpu.all(config)
    }

    pub fn cpu_multithread(&self, config: cpu::Config) -> Result<cpu::Report, cpu::Error> {
        self.cpu.all_multithread(config)
    }

    pub fn ram(&self, config: ram::Config) -> Result<ram::Report, ram::Error> {
        self.ram.all(config)
    }

    pub fn storage(&self, config: storage::Config) -> Result<storage::Report, storage::Error> {
        self.storage.all(config)
    }

    pub fn all(&self, config: Config) -> Result<Report, Error> {
        Ok(Report {
            cpu: self.cpu(config.cpu).map_err(Error::Cpu)?,
            cpu_multithread: self.cpu_multithread(config.cpu_multithread).map_err(Error::Cpu)?,
            ram: self.ram(config.ram).map_err(Error::Ram)?,
            storage: self.storage(config.storage).map_err(Error::Storage)?,
        })
    }
}

#[derive(Default)]
pub struct Config {
    cpu: cpu::Config,
    cpu_multithread: cpu::Config,
    ram: ram::Config,
    storage: storage::Config,
}

pub struct Report {
    cpu: cpu::Report,
    cpu_multithread: cpu::Report,
    ram: ram::Report,
    storage: storage::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.cpu)?;
        writeln!(f, "Multithreaded {}", self.cpu_multithread)?;
        writeln!(f, "{}", self.ram)?;
        writeln!(f, "{}", self.storage)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Cpu(cpu::Error),
    Ram(ram::Error),
    Storage(storage::Error),
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
            }.into(),
            math: cpu::math::Config {
                duration,
                n: 10,
                ..Default::default()
            }.into(),
            sort: cpu::sort::Config {
                duration,
                item_len: 25,
                data_len: 100_000,
                ..Default::default()
            }.into(),
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
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
                seq_data_len: 64,
                seq_iters: 5,
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
                seq_data_len_mb: 1,
                seq_iters: 1,
                ..Default::default()
            },
        });

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();

        println!("{result}");
    }
}
