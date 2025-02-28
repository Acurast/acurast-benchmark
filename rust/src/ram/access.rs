use std::{
    fmt,
    hint::black_box,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;

use crate::{utils::{Avg, KB}, CpuFeatures};

pub(crate) fn bench(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.seq_iters, context.rand_iters, context.concurr_iters);

    let mut start: Instant;
    for _ in 0..context.seq_iters {
        context.reset_data(context.seq_data_len);

        start = Instant::now();
        black_box(sequential::run_test(context.seq_data())?);
        report_builder.add_seq(start.elapsed());
    }

    let indices = (0..context.rand_data().len()).collect::<Vec<_>>();
    let mut write_indices;
    let mut read_indices;
    for _ in 0..context.rand_iters {
        context.reset_data(context.rand_data_len);

        write_indices = indices.clone();
        write_indices.shuffle(&mut context.rng);

        read_indices = indices.clone();
        read_indices.shuffle(&mut context.rng);

        start = Instant::now();
        black_box(random::run_test(
            context.rand_data(),
            &write_indices,
            &read_indices,
        )?);
        report_builder.add_rand(start.elapsed());
    }

    let chunk_size = context.concurr_data().len().div_ceil(features.num_cores);
    for _ in 0..context.concurr_iters {
        context.reset_data(context.concurr_data_len);
        let chunks = context.concurr_data().chunks_mut(chunk_size).collect::<Vec<_>>();

        start = Instant::now();
        black_box(concurrent::run_test(chunks)?);
        report_builder.add_concurr(start.elapsed());
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

    pub(super) fn run_test(
        data: &mut [u8],
        write_indices: &Vec<usize>,
        read_indices: &Vec<usize>,
    ) -> Result<(), Error> {
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
        chunks
            .into_par_iter()
            .map(|data| {
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
            })
            .reduce(
                || Ok(()),
                |acc, next| match (acc, next) {
                    (Ok(_), Ok(_)) => Ok(()),
                    (Ok(_), Err(err)) => Err(err),
                    (Err(err), _) => Err(err),
                },
            )?;

        Ok(())
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,

    pub seq_iters: usize,
    pub seq_data_len: usize,

    pub rand_iters: usize,
    pub rand_data_len: usize,

    pub concurr_iters: usize,
    pub concurr_data_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        let iters = 100;
        let data_len = 64 * KB;

        Self {
            rng: Box::new(rand::thread_rng()),

            seq_iters: iters,
            seq_data_len: data_len,

            rand_iters: iters,
            rand_data_len: data_len,

            concurr_iters: iters,
            concurr_data_len: data_len,
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
    pub concurr_avg_t: Duration,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "sequential access ... {:.6} s",
            self.seq_avg_t.as_secs_f64()
        )?;
        writeln!(
            f,
            "random access ... {:.6} s",
            self.rand_avg_t.as_secs_f64()
        )?;
        write!(
            f,
            "concurrent access ... {:.6} s",
            self.concurr_avg_t.as_secs_f64()
        )?;

        Ok(())
    }
}

struct ReportBuilder {
    seq_ts: Vec<Duration>,
    rand_ts: Vec<Duration>,
    concurr_ts: Vec<Duration>,
}

impl ReportBuilder {
    fn new(seq_iters: usize, rand_iters: usize, concurr_iters: usize) -> Self {
        Self {
            seq_ts: Vec::with_capacity(seq_iters),
            rand_ts: Vec::with_capacity(rand_iters),
            concurr_ts: Vec::with_capacity(concurr_iters),
        }
    }

    fn add_seq(&mut self, time: Duration) {
        self.seq_ts.push(time);
    }

    fn add_rand(&mut self, time: Duration) {
        self.rand_ts.push(time);
    }

    fn add_concurr(&mut self, time: Duration) {
        self.concurr_ts.push(time);
    }

    fn build(self) -> Report {
        Report {
            seq_avg_t: self.seq_ts.avg(),
            rand_avg_t: self.rand_ts.avg(),
            concurr_avg_t: self.concurr_ts.avg(),
        }
    }
}

struct Context {
    rng: Box<dyn rand::RngCore>,

    data: Vec<u8>,

    seq_iters: usize,
    seq_data_len: usize,

    rand_iters: usize,
    rand_data_len: usize,

    concurr_iters: usize,
    concurr_data_len: usize,
}

impl Context {
    fn new(config: Config) -> Self {
        let data = vec![0u8; config.seq_data_len];

        Self {
            rng: config.rng,
            data,

            seq_iters: config.seq_iters,
            seq_data_len: config.seq_data_len,

            rand_iters: config.rand_iters,
            rand_data_len: config.rand_data_len,

            concurr_iters: config.concurr_iters,
            concurr_data_len: config.concurr_data_len
        }
    }

    fn seq_data(&mut self) -> &mut [u8] {
        &mut self.data[..self.seq_data_len]
    }

    fn rand_data(&mut self) -> &mut [u8] {
        &mut self.data[..self.rand_data_len]
    }

    fn concurr_data(&mut self) -> &mut [u8] {
        &mut self.data[..self.concurr_data_len]
    }

    fn reset_data(&mut self, size: usize) {
        self.data.clear();
        self.data.resize(size, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench() {
        let iters = 5;
        let data_len = 64;

        let result = bench(
            &CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
            Config {
                seq_iters: iters,
                seq_data_len: data_len,
                rand_iters: iters,
                rand_data_len: data_len,
                concurr_iters: iters,
                concurr_data_len: data_len,
                ..Default::default()
            },
        );

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.seq_avg_t > Duration::ZERO);
        assert!(result.rand_avg_t > Duration::ZERO);
        assert!(result.concurr_avg_t > Duration::ZERO);

        println!("{result}");
    }
}
