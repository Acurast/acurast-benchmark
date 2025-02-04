use std::{fmt, hint::black_box, time::{Duration, Instant}};

use rand::seq::SliceRandom;

use crate::{utils::Avg, CpuFeatures};

pub(crate) fn bench(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.iters);

    let mut start: Instant;
    for _ in 0..context.iters {
        context.reset_data();

        start = Instant::now();
        black_box(sequential::run_test(&mut context.data)?);
        report_builder.add_seq(start.elapsed());
    }

    let indices = (0..context.data.len()).collect::<Vec<_>>();
    let mut write_indices;
    let mut read_indices;
    for _ in 0..context.iters {
        context.reset_data();

        write_indices = indices.clone();
        write_indices.shuffle(&mut context.rng);

        read_indices = indices.clone();
        read_indices.shuffle(&mut context.rng);

        start = Instant::now();
        black_box(random::run_test(&mut context.data, &write_indices, &read_indices)?);
        report_builder.add_rand(start.elapsed());
    }

    let chunk_size = context.data.len().div_ceil(features.num_cores);
    for _ in 0..context.iters {
        context.reset_data();
        let chunks = context.data.chunks_mut(chunk_size).collect::<Vec<_>>();
        
        start = Instant::now();
        black_box(concurrent::run_test(chunks)?);
        report_builder.add_con(start.elapsed());
    }

    Ok(report_builder.build())
}

mod sequential {
    use super::*;

    pub(super) fn run_test(data: &mut [u8]) -> Result<(), Error> {
        for i in 0..data.len() {
            data[i] = (i % 256) as u8;
        }
        for i in 0..data.len() {
            let v = data[i];
            let expected = (i % 256) as u8;
            if v != expected {
                return Err(Error::InvalidValue(expected, v));
            }
        }

        Ok(())
    }
}

mod random {
    use super::*;

    pub(super) fn run_test(data: &mut [u8], write_indices: &Vec<usize>, read_indices: &Vec<usize>) -> Result<(), Error> {
        for &i in write_indices {
            data[i] = (i % 256) as u8;
        }
        for &i in read_indices {
            let v = data[i];
            let expected = (i % 256) as u8;
            if v != expected {
                return Err(Error::InvalidValue(expected, v));
            }
        }

        Ok(())
    }
}

mod concurrent {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use super::*;

    pub(super) fn run_test(chunks: Vec<&mut [u8]>) -> Result<(), Error> {
        chunks.into_par_iter().map(|data| {
            for i in 0..data.len() {
                data[i] = (i % 256) as u8;
            }
            for i in 0..data.len() {
                let v = data[i];
                let expected = (i % 256) as u8;
                if v != expected {
                    return Err(Error::InvalidValue(expected, v));
                }
            }

            Ok(())
        }).reduce(|| Ok(()), |acc, next| {
            match (acc, next) {
                (Ok(_), Ok(_)) => Ok(()),
                (Ok(_), Err(err)) => Err(err),
                (Err(err), _) => Err(err),
            }
        })?;

        Ok(())
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,
    pub data_len: usize,
    pub iters: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            rng: Box::new(rand::thread_rng()),
            data_len: 64 * 1024,
            iters: 100,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidValue(u8, u8),
}

pub struct Report {
    pub seq_avg_t: Duration,
    pub rand_avg_t: Duration,
    pub con_avg_t: Duration,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "sequential access ... {:.6} s", self.seq_avg_t.as_secs_f64())?;
        writeln!(f, "random access ... {:.6} s", self.rand_avg_t.as_secs_f64())?;
        write!(f, "concurrent access ... {:.6} s", self.con_avg_t.as_secs_f64())?;

        Ok(())
    }
}

struct ReportBuilder {
    seq_ts: Vec<Duration>,
    rand_ts: Vec<Duration>,
    con_ts: Vec<Duration>,
}

impl ReportBuilder {
    fn new(iters: usize) -> Self {
        Self {
            seq_ts: Vec::with_capacity(iters),
            rand_ts: Vec::with_capacity(iters),
            con_ts: Vec::with_capacity(iters),
        }
    }

    fn add_seq(&mut self, time: Duration) {
        self.seq_ts.push(time);
    }

    fn add_rand(&mut self, time: Duration) {
        self.rand_ts.push(time);
    }

    fn add_con(&mut self, time: Duration) {
        self.con_ts.push(time);
    }

    fn build(self) -> Report {
        Report { 
            seq_avg_t: self.seq_ts.avg(),
            rand_avg_t: self.rand_ts.avg(),
            con_avg_t: self.con_ts.avg(),
        }
    }
}

struct Context {
    rng: Box<dyn rand::RngCore>,

    iters: usize,
    data: Vec<u8>,
}

impl Context { 
    fn new(config: Config) -> Self {
        let data = vec![0u8; config.data_len];

        Self {
            rng: config.rng,
            iters: config.iters,
            data,
        }
    }

    fn reset_data(&mut self) {
        let size = self.data.len();
        self.data.clear();
        self.data.resize(size, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench() {
        let result = bench(
            &CpuFeatures { num_cores: 8, sve: false, i8mm: false },
            Config {
                data_len: 1024,
                iters: 10,
                ..Default::default()
            },
        );

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.seq_avg_t > Duration::ZERO);
        assert!(result.rand_avg_t > Duration::ZERO);

        println!("{result}");
    }
}