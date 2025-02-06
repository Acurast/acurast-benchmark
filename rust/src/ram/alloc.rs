use std::{
    fmt,
    hint::black_box,
    time::{Duration, Instant},
};

use crate::utils::{Avg, MB};

pub(crate) fn bench(config: Config) -> Result<Report, Error> {
    let mut report_builder = ReportBuilder::new(config.iters);

    let mut start: Instant;
    for _ in 0..config.iters {
        start = Instant::now();
        black_box(run_test(config.data_len)?);
        report_builder.add(start.elapsed());
    }

    Ok(report_builder.build())
}

fn run_test(n: usize) -> Result<(), Error> {
    let mut data = Vec::with_capacity(n);
    data.resize(n, 0u8);
    if data.len() != n {
        return Err(Error::WrongLen(data.len()));
    }
    drop(data);

    Ok(())
}

pub struct Config {
    pub data_len: usize,
    pub iters: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_len: 64 * MB,
            iters: 100,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    WrongLen(usize),
}

pub struct Report {
    pub avg_t: Duration,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "alloc ... {:.6} s", self.avg_t.as_secs_f64())
    }
}

struct ReportBuilder {
    ts: Vec<Duration>,
}

impl ReportBuilder {
    fn new(iters: usize) -> Self {
        Self {
            ts: Vec::with_capacity(iters),
        }
    }

    fn add(&mut self, time: Duration) {
        self.ts.push(time);
    }

    fn build(self) -> Report {
        Report {
            avg_t: self.ts.avg(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench() {
        let result = bench(Config {
            data_len: 64,
            iters: 5,
            ..Default::default()
        });

        assert_eq!(true, result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.avg_t > Duration::ZERO);

        println!("{result}");
    }
}
