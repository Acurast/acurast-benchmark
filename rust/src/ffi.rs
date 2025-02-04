use std::{ptr::null, time::Duration};

use crate::{cpu, arm::{Auxval, AuxvalMask}, Bench};

#[repr(C)]
pub struct TypedU64 {
    pub t: u8,
    pub v: u64,
}

#[no_mangle]
pub extern "C" fn new_bench(
    hwcap: u64,
    hwcap2: u64,
    sve_mask: TypedU64,
    i8mm_mask: TypedU64,
) -> *mut Bench {
    let bench = Bench::with_auxval(Auxval {
        hwcap,
        hwcap2,
        sve_mask: sve_mask.into(),
        i8mm_mask: i8mm_mask.into(),
    });

    Box::into_raw(Box::new(bench))
}

#[no_mangle]
pub extern "C" fn drop_bench(bench: *mut Bench) {
    unsafe { drop(Box::from_raw(bench)) };
}

#[repr(C)]
pub struct CpuConfig {
    duration: usize,
    enc_data_len: usize,
    math_data_len: usize,
    sort_data_len: usize,
}

#[repr(C)]
pub struct CpuReport {
    crypto_tps: f64,
    math_tps: f64,
    sort_tps: f64,

    err: *const u8,
    err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_cpu(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let report = bench.cpu(config.into());

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn bench_cpu_multithread(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let report = bench.cpu_multithread(config.into());

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_cpu_report(report: *const CpuReport) {
    unsafe {
        let report = Box::from_raw(report as *mut CpuReport);
        if !report.err.is_null() {
            let err = String::from_raw_parts(report.err as *mut u8, report.err_len, report.err_len);
            drop(err)
        }

        drop(report);
    }
}

impl From<TypedU64> for AuxvalMask {
    fn from(value: TypedU64) -> Self {
        match value.t {
            0 => Self::HWCAP(value.v),
            1 => Self::HWCAP2(value.v),
            _ => Self::HWCAP2(value.v)
        }
    }
}

impl From<CpuConfig> for cpu::Config {
    fn from(value: CpuConfig) -> Self {
        let duration = Duration::from_millis(value.duration as u64) / 3;

        Self {
            crypto: cpu::crypto::Config {
                duration,
                data_len: value.enc_data_len.try_into().unwrap(),
                ..Default::default()
            },
            math: cpu::math::Config {
                duration,
                n: value.math_data_len.try_into().unwrap(),
                ..Default::default()
            },
            sort: cpu::sort::Config {
                duration,
                data_len: value.sort_data_len.try_into().unwrap(),
                ..Default::default()
            }
        }
    }
}

impl From<Result<cpu::Report, cpu::Error>> for CpuReport {
    fn from(value: Result<cpu::Report, cpu::Error>) -> Self {
        match value {
            Ok(report) => Self {
                crypto_tps: report.crypto.tps,
                math_tps: report.math.tps,
                sort_tps: report.sort.tps,
                err: null(),
                err_len: 0,
            },
            Err(err) => {
                let err = format!("{err:?}");
                let report = Self { 
                    crypto_tps: 0.,
                    math_tps: 0.,
                    sort_tps: 0.,
                    err: err.as_ptr(),
                    err_len: err.len(),
                };

                std::mem::forget(err);

                report
            },
        }
    }
}