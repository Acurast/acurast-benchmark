use std::{fmt, hint::black_box, time::Duration};

use rand::Rng;

use crate::{utils::{closest_pow, is_pow, Expirable, GetValue, Timeout}, CpuFeatures};

pub(crate) fn bench(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.timeout.duration);
    
    'main: while !context.timeout.reached() {
        context.rng.fill(&mut context.matrix_a_i8[..]);
        context.rng.fill(&mut context.matrix_b_i8[..]);
        context.matrix_r_i32 = vec![0; context.matrix_r_i32.len()];

        let ops = if features.i8mm && features.sve {
            black_box(
                matrix::run_test_simd(
                    &context.matrix_a_i8[..], 
                    &context.matrix_b_i8[..], 
                    &mut context.matrix_r_i32[..],
                    context.n,
                    Some(&context.timeout),
                )
            )
        } else {
            black_box(
                matrix::run_test(
                    &context.matrix_a_i8.chunks(context.n).collect::<Vec<_>>()[..],
                    &context.matrix_b_i8.chunks(context.n).collect::<Vec<_>>()[..],
                    &mut context.matrix_r_i32.chunks_mut(context.n).collect::<Vec<_>>()[..],
                    Some(&context.timeout),
                )
            )
        };

        report_builder.add(ops);

        if ops.is_ok() {
            for i in 0..context.n {
                if context.timeout.reached() {
                    break 'main;
                }

                if context.matrix_r_i32[i * context.n] /* first in row */ == 0 && context.matrix_r_i32[(i + 1) * context.n - 1] /* last in row */ == 0 {
                    return Err(Error::Empty);
                }
            }
        }
    }

    Ok(report_builder.build())
}

pub(crate) fn bench_multithread(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let threadpool = rayon::ThreadPoolBuilder::new().num_threads(features.num_cores).build().unwrap();
    let mut result_builder = ReportBuilder::new(context.timeout.duration);
    
    'main: while !context.timeout.reached() {
        context.rng.fill(&mut context.matrix_a_f32[..]);
        context.rng.fill(&mut context.matrix_b_f32[..]);
        context.matrix_r_f32 = vec![0.; context.matrix_r_f32.len()];

        let ops = black_box(
            matrix::run_test_multithread(
                &threadpool,
                &context.matrix_a_f32.chunks(context.n).collect::<Vec<_>>()[..],
                &context.matrix_b_f32.chunks(context.n).collect::<Vec<_>>()[..],
                &mut context.matrix_r_f32.chunks_mut(context.n).collect::<Vec<_>>()[..],
                Some(&context.timeout),
            )
        );

        result_builder.add(ops);

        if ops.is_ok() {
            for i in 0..context.n {
                if context.timeout.reached() {
                    break 'main;
                }

                if context.matrix_r_f32[i * context.n] /* first in row */ == 0. && context.matrix_r_f32[(i + 1) * context.n - 1] /* last in row */ == 0. {
                    return Err(Error::Empty);
                }
            }
        }
    }

    Ok(result_builder.build())
}

extern "C" {
    fn matrix_mul_i8mm(
        matrix_a: *const i8,
        b: *const i8,
        r: *mut i32,
        n: usize,
        timeout_timestamp: usize,
    ) -> i64;
}

mod matrix {
    use std::{ops::{Add, Mul}, time::{Instant, SystemTime, UNIX_EPOCH}};

    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    use crate::utils::GetValue;

    use super::*;

    macro_rules! split {
        ($matrix: expr) => {{
            let mid = $matrix.len() / 2;

            let (e1, e2) = $matrix.split_at(mid);
            let (e11, e12) = e1.iter().map(|row| row.split_at(mid)).collect::<(Vec<_>, Vec<_>)>();
            let (e21, e22) = e2.iter().map(|row| row.split_at(mid)).collect::<(Vec<_>, Vec<_>)>();

            (e11, e12, e21, e22)
        }};
        (mut $matrix: expr) => {{
            let mid = $matrix.len() / 2;

            let (e1, e2) = $matrix.split_at_mut(mid);
            let (e11, e12) = e1.iter_mut().map(|row| row.split_at_mut(mid)).collect::<(Vec<_>, Vec<_>)>();
            let (e21, e22) = e2.iter_mut().map(|row| row.split_at_mut(mid)).collect::<(Vec<_>, Vec<_>)>();

            (e11, e12, e21, e22)
        }};
    }

    pub(super) fn run_test<T, R>(
        matrix_a: &[&[T]],
        matrix_b: &[&[T]],
        matrix_r: &mut [&mut [R]],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64>
    where 
        T: Into<R> + Copy + Send + Sync,
        R: Add<Output = R> + Mul<Output = R> + Copy + Send + Sync,
    {
        mul(matrix_a, matrix_b, matrix_r, timeout, None)
    }

    pub(super) fn run_test_simd(
        matrix_a: &[i8],
        matrix_b: &[i8],
        matrix_r: &mut [i32],
        n: usize,
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64> {
        let timeout = match timeout {
            Some(timeout) => {
                let timestamp = SystemTime::now() + (timeout.start - Instant::now()) + timeout.duration;
                if let Ok(timestamp) = timestamp.duration_since(UNIX_EPOCH) {
                    u128::min(usize::MAX as u128, timestamp.as_millis()) as usize
                } else {
                    0
                }
            },
            None => 0,
        };

        let ops = unsafe { matrix_mul_i8mm(matrix_a.as_ptr(), matrix_b.as_ptr(), matrix_r.as_mut_ptr(), n, timeout) };
        if ops > 0 {
            Ok(ops as u64)
        } else {
            Err(ops as u64)
        }
    }

    pub(super) fn run_test_multithread<T, R>(
        threadpool: &rayon::ThreadPool,
        matrix_a: &[&[T]],
        matrix_b: &[&[T]],
        matrix_r: &mut [&mut [R]],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64>
    where 
        T: Into<R> + Copy + Send + Sync,
        R: Add<Output = R> + Mul<Output = R> + Copy + Send + Sync,
    {
        threadpool.install(|| {
            mul(matrix_a, matrix_b, matrix_r, timeout, Some(&threadpool))
        })
    }

    fn mul<T, R>(
        matrix_a: &[&[T]],
        matrix_b: &[&[T]],
        matrix_r: &mut [&mut [R]],
        timeout: Option<&Timeout>,
        threadpool: Option<&rayon::ThreadPool>,
    ) -> Result<u64, u64>
    where 
        T: Into<R> + Copy + Send + Sync,
        R: Add<Output = R> + Mul<Output = R> + Copy + Send + Sync,
    {
        let mut ops = 0;
        timeout.reached_with_err(ops)?;

        if matrix_a.len() == 1 && matrix_b.len() == 1 {
            matrix_r[0][0] = matrix_r[0][0] + matrix_a[0][0].into() * matrix_b[0][0].into();
            ops += 1;
            return Ok(ops);
        }

        let (a11, a12, a21, a22) = split!(matrix_a);
        let (b11, b12, b21, b22) = split!(matrix_b);
        let (mut r11, mut r12, mut r21, mut r22) = split!(mut matrix_r);

        let tuples = [
            (&mut r11, &a11, &b11, &a12, &b21),
            (&mut r12, &a11, &b12, &a12, &b22),
            (&mut r21, &a21, &b11, &a22, &b21),
            (&mut r22, &a21, &b12, &a22, &b22),
        ];

        if let Some(_) = threadpool {
            ops += tuples.into_par_iter().map(|(r, a1, b1, a2, b2)| {
                if timeout.reached() {
                    return Err(0);
                }

                let ops1 = mul(a1, b1, r, timeout, threadpool)?;
                let ops2 = mul(a2, b2, r, timeout, threadpool).map_err(|o| ops1 + o)?;

                Ok(ops1 + ops2)
            }).fold(|| Ok(0), |acc, next| {
                match (acc, next) {
                    (Ok(acc), Ok(next)) => Ok(acc + next),
                    _ => Err(acc.value() + next.value()),
                }
            }).sum::<Result<u64, u64>>().map_err(|o| ops + o)?;
        } else {
            for (r, a1, b1, a2, b2) in tuples {
                timeout.reached_with_err(ops)?;
                
                ops += mul(a1, b1, r, timeout, threadpool).map_err(|o| ops + o)?;
                ops += mul(a2, b2, r, timeout, threadpool).map_err(|o| ops + o)?;
            }
        }

        Ok(ops)
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,

    pub duration: Duration,

    pub n: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            rng: Box::new(rand::thread_rng()),
            duration: Duration::from_secs(10),
            n: 4096,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Empty,
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
        write!(f, "mul {} ops/s", self.tps.floor())
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

    n: usize,
    matrix_a_i8: Vec<i8>,
    matrix_b_i8: Vec<i8>,
    matrix_r_i32: Vec<i32>,

    matrix_a_f32: Vec<f32>,
    matrix_b_f32: Vec<f32>,
    matrix_r_f32: Vec<f32>,

    timeout: Timeout,
}

impl Context {
    fn new(config: Config) -> Self {
        let n = if is_pow(config.n, 2) {
            config.n
        } else {
            closest_pow(config.n, 2)
        };

        let mut matrix_a_i8 = Vec::with_capacity(n * n);
        unsafe { matrix_a_i8.set_len(n * n) };
        
        let mut matrix_b_i8 = Vec::with_capacity(n * n);
        unsafe { matrix_b_i8.set_len(n * n) };

        let mut matrix_r_i32 = Vec::with_capacity(n * n);
        unsafe { matrix_r_i32.set_len(n * n) };

        let mut matrix_a_f32 = Vec::with_capacity(n * n);
        unsafe { matrix_a_f32.set_len(n * n) };
        
        let mut matrix_b_f32 = Vec::with_capacity(n * n);
        unsafe { matrix_b_f32.set_len(n * n) };

        let mut matrix_r_f32 = Vec::with_capacity(n * n);
        unsafe { matrix_r_f32.set_len(n * n) };

        let timeout = Timeout::new(config.duration);

        Self {
            rng: config.rng,
            n,
            matrix_a_i8,
            matrix_b_i8,
            matrix_r_i32,
            matrix_a_f32,
            matrix_b_f32,
            matrix_r_f32,
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
        let duration = Duration::from_millis(10000);
        let start = Instant::now();
        let result = bench(
            &CpuFeatures { num_cores: 1, sve: false, i8mm: false },
            Config {
                duration,
                n: 200,
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
        let duration = Duration::from_millis(10000);
        let start = Instant::now();
        let result = bench_multithread(
            &CpuFeatures { num_cores: 8, sve: false, i8mm: false },
            Config {
                duration,
                n: 200,
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
    fn test_matrix() {
        let matrix_a = [
            &[80, 43, 16, 5],
            &[70, 41, 38, 62],
            &[31, 19, 97, 39],
            &[66, 6, 40, 28],
        ].map(|row| &row[..]);

        let matrix_b = [
            &[24, 12, 24, 29],
            &[83, 59, 32, 44],
            &[97, 38, 67, 13],
            &[98, 64, 68, 29],
        ].map(|row| &row[..]);

        let matrix_r_expected = [
            [7531, 4425, 4708, 4565],
            [14845, 8671, 9754, 6126],
            [15552, 7675, 10503, 4127],
            [8706, 4458, 6360, 3510],
        ];

        let mut matrix_r = [[0i32; 4]; 4];
        let mut matrix_r_slice: Vec<&mut [i32]> = matrix_r.iter_mut().map(|row| &mut row[..]).collect();

        let result = matrix::run_test(&matrix_a, &matrix_b, &mut matrix_r_slice, None);

        assert_eq!(true, result.is_ok(), "expected success");
        assert_eq!(matrix_r_expected, matrix_r);
    }

    #[test]
    fn test_matrix_multithread() {
        let matrix_a = [
            &[80, 43, 16, 5],
            &[70, 41, 38, 62],
            &[31, 19, 97, 39],
            &[66, 6, 40, 28],
        ].map(|row| &row[..]);

        let matrix_b = [
            &[24, 12, 24, 29],
            &[83, 59, 32, 44],
            &[97, 38, 67, 13],
            &[98, 64, 68, 29],
        ].map(|row| &row[..]);

        let matrix_r_expected = [
            [7531, 4425, 4708, 4565],
            [14845, 8671, 9754, 6126],
            [15552, 7675, 10503, 4127],
            [8706, 4458, 6360, 3510],
        ];

        let mut matrix_r = [[0i32; 4]; 4];
        let mut matrix_r_slice: Vec<&mut [i32]> = matrix_r.iter_mut().map(|row| &mut row[..]).collect();

        let threadpool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        let result = matrix::run_test_multithread(&threadpool, &matrix_a, &matrix_b, &mut matrix_r_slice, None);

        assert_eq!(true, result.is_ok(), "expected success");
        assert_eq!(matrix_r_expected, matrix_r);
    }

    #[no_mangle]
    extern "C" fn matrix_mul_i8mm(
        _matrix_a: *const i8,
        _b: *const i8,
        _r: *mut i32,
        _n: usize,
        _timeout_timestamp: usize,
    ) -> i64 {
        0
    }
}