use std::{fmt, hint::black_box, time::Duration};

use rand::distributions::DistString;

use crate::{utils::{Expirable, GetValue, Timeout}, CpuFeatures};

pub(crate) fn bench(_features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.timeout.duration);

    'main: while !context.timeout.reached() {
        for i in 0..context.data.len() {
            if context.timeout.reached() {
                break 'main;
            }

            context.data[i] = rand::distributions::Alphanumeric.sample_string(&mut context.rng, context.item_len);
        }

        let ops = black_box(
            merge::run_test(
                &mut context.data[..],
                &mut context.temp[..],
                Some(&context.timeout),
            )
        );
        if ops.is_ok() && !context.data.is_sorted() {
            return Err(Error::Unsorted(context.data))
        }

        report_builder.add(ops);
    }

    Ok(report_builder.build())
}

pub(crate) fn bench_multithread(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let threadpool = rayon::ThreadPoolBuilder::new().num_threads(features.num_cores).build().unwrap();
    let mut result_builder = ReportBuilder::new(context.timeout.duration);

    'main: while !context.timeout.reached() {
        for i in 0..context.data.len() {
            if context.timeout.reached() {
                break 'main;
            }

            context.data[i] = rand::distributions::Alphanumeric.sample_string(&mut context.rng, context.item_len);
        }

        let ops = black_box(
            merge::run_test_multithread(
                &threadpool,
                &mut context.data[..],
                &mut context.temp[..],
                Some(&context.timeout),
            )
        );
        if ops.is_ok() && !context.data.is_sorted() {
            return Err(Error::Unsorted(context.data))
        }

        result_builder.add(ops);
    }

    Ok(result_builder.build())
}

mod merge {
    use super::*;

    pub(super) fn run_test<T>(data: &mut [T], temp: &mut [T], timeout: Option<&Timeout>) -> Result<u64, u64> 
    where
        T: Clone + PartialOrd + Send,
    {
        sort(data, temp, timeout, None)
    }

    pub(super) fn run_test_multithread<T>(
        threadpool: &rayon::ThreadPool,
        data: &mut [T],
        temp: &mut [T],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64> 
    where
        T: Clone + PartialOrd + Send,
    {
        threadpool.install(|| {
            sort(data, temp, timeout, Some(&threadpool))
        })
    }

    fn sort<T>(
        data: &mut [T],
        temp: &mut [T],
        timeout: Option<&Timeout>,
        threadpool: Option<&rayon::ThreadPool>,
    ) -> Result<u64, u64>
    where
        T: Clone + PartialOrd + Send,
    {
        let mut ops = 0;
        timeout.reached_with_err(ops)?;

        if data.len() == 1 {
            ops += 1;
            return Ok(ops);
        }
        
        let mid = data.len() / 2;
        let (data_left, data_right) = data.split_at_mut(mid);
        let (temp_left, temp_right) = temp.split_at_mut(mid);

        if let Some(_) = threadpool {
            let (left_ops, right_ops) = rayon::join(
                || sort(data_left, temp_left, timeout, threadpool),
                || sort(data_right, temp_right, timeout, threadpool),
            );

            ops += left_ops.map_err(|o| ops + o)?;
            ops += right_ops.map_err(|o| ops + o)?;
        } else {
            ops += sort(data_left, temp_left, timeout, threadpool).map_err(|o| ops + o)?;
            ops += sort(data_right, temp_right, timeout, threadpool).map_err(|o| ops + o)?;
        }
        
        for i in 0..data_left.len() {
            timeout.reached_with_err(ops)?;

            temp_left[i] = data_left[i].clone();
        }

        for i in 0..data_right.len() {
            timeout.reached_with_err(ops)?;

            temp_right[i] = data_right[i].clone();
        }

        ops += merge(
            data,
            temp_left,
            temp_right,
            timeout,
        ).map_err(|o| ops + o)?;

        Ok(ops)
    }

    fn merge<T>(
        data: &mut [T],
        left: &[T],
        right: &[T],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64>
    where
        T: Clone + PartialOrd,
    {
        let mut d = 0;
        let mut l = 0;
        let mut r = 0;

        while l < left.len() && r < right.len() {
            timeout.reached_with_err(d as u64)?;

            if left[l] <= right[r] {
                data[d] = left[l].clone();
                l += 1;
            } else {
                data[d] = right[r].clone();
                r += 1;
            }
            d += 1;
        }

        while l < left.len() {
            timeout.reached_with_err(d as u64)?;

            data[d] = left[l].clone();
            l += 1;
            d += 1;
        }

        while r < right.len() {
            timeout.reached_with_err(d as u64)?;

            data[d] = right[r].clone();
            r += 1;
            d += 1;
        }

        Ok(d as u64)
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,

    pub duration: Duration,

    pub item_len: usize,
    pub data_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            rng: Box::new(rand::thread_rng()),
            duration: Duration::from_secs(10),
            item_len: 25,
            data_len: 100_000,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Unsorted(Vec<String>),
}

pub struct Report {
    pub duration: Duration,
    pub ops: u64,
    pub tps: f64,
}

impl Report {
    fn new(builder: ReportBuilder) -> Self {
        Self {
            duration: builder.duration,
            ops: builder.ops,
            tps: builder.ops as f64 / builder.duration.as_secs_f64(),
        }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sort {} ops/s", self.tps.floor())
    }
}

struct ReportBuilder {
    duration: Duration,
    ops: u64,
}

impl ReportBuilder {
    fn new(duration: Duration) -> Self {
        Self { duration, ops: 0 }
    }

    fn add(&mut self, result: Result<u64, u64>) {
        self.ops += result.value();
    }

    fn build(self) -> Report {
        Report::new(self)
    }
}

struct Context {
    rng: Box<dyn rand::RngCore>,

    item_len: usize,
    data: Vec<String>,
    temp: Vec<String>,

    timeout: Timeout,
}

impl Context {
    fn new(config: Config) -> Self {
        let mut data = Vec::with_capacity(config.data_len);
        unsafe { data.set_len(config.data_len) };

        let mut temp = Vec::with_capacity(config.data_len);
        unsafe { temp.set_len(config.data_len) };

        let timeout = Timeout::new(config.duration);

        Self {
            rng: config.rng,
            item_len: config.item_len,
            data,
            temp,
            timeout,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_bench() {
        let duration = Duration::from_millis(1000);
        let start = Instant::now();
        let result = bench(
            &CpuFeatures { num_cores: 1, sve: false, i8mm: false },
            Config {
                duration,
                data_len: 10000,
                ..Default::default()
            },
        );
        let elapsed = start.elapsed();

        assert_eq!(true, result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.ops > 0);
        assert!(result.tps > 0f64);
        assert!(elapsed >= duration && elapsed <= duration + Duration::from_millis(100));

        println!("{result}");
    }

    #[test]
    fn test_bench_multithread() {
        let duration = Duration::from_millis(1000);
        let start = Instant::now();
        let result = bench_multithread(
            &CpuFeatures { num_cores: 8, sve: false, i8mm: false },
            Config {
                duration,
                data_len: 10000,
                ..Default::default()
            },
        );
        let elapsed = start.elapsed();

        assert_eq!(true, result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.ops > 0);
        assert!(result.tps > 0f64);
        assert!(elapsed >= duration && elapsed <= duration + Duration::from_millis(100));

        println!("{result}");
    }

    #[test]
    fn test_merge() {
        let mut data = [19, 72, 4, 86, 44, 7, 100, 79, 100, 99, 27, 12, 81, 46, 32];
        let mut temp = [0i32; 15];
        let expected = [4, 7, 12, 19, 27, 32, 44, 46, 72, 79, 81, 86, 99, 100, 100];

        let result = merge::run_test(&mut data, &mut temp, None);

        assert_eq!(true, result.is_ok(), "expected success");
        assert_eq!(expected, data);
    }

    #[test]
    fn test_merge_multithread() {
        let mut data = [19, 72, 4, 86, 44, 7, 100, 79, 100, 99, 27, 12, 81, 46, 32];
        let mut temp = [0i32; 15];
        let expected = [4, 7, 12, 19, 27, 32, 44, 46, 72, 79, 81, 86, 99, 100, 100];

        let threadpool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        let result = merge::run_test_multithread(&threadpool, &mut data, &mut temp, None);

        assert_eq!(true, result.is_ok(), "expected success");
        assert_eq!(expected, data);
    }
}