use std::{path::PathBuf, ptr::null, slice, str, time::Duration};

use crate::{arm::{Auxval, AuxvalMask}, cpu, ram, storage, Bench};

#[repr(C)]
pub struct TypedU64 {
    pub t: u8,
    pub v: u64,
}

#[no_mangle]
pub extern "C" fn new_bench(
    total_ram: u64,
    avail_storage: u64,
    hwcap: u64,
    hwcap2: u64,
    sve_mask: TypedU64,
    i8mm_mask: TypedU64,
) -> *mut Bench {
    let bench = Bench::with_auxval(total_ram, avail_storage, Auxval {
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
        drop_string(report.err, report.err_len);

        drop(report);
    }
}

#[repr(C)]
pub struct RamConfig {
    alloc_data_len: usize,
    access_data_len: usize,
    iters: usize,
}

#[repr(C)]
pub struct RamReport {
    total_mem: u64,
    alloc_avg_t: f64,
    access_seq_avg_t: f64,
    access_rand_avg_t: f64,
    access_con_avg_t: f64,

    err: *const u8,
    err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_ram(bench: *mut Bench, config: RamConfig) -> *const RamReport {
    let bench = unsafe { &mut *bench };
    let report = bench.ram(config.into());

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_ram_report(report: *const RamReport) {
    unsafe {
        let report = Box::from_raw(report as *mut RamReport);
        drop_string(report.err, report.err_len);

        drop(report);
    }
}

#[repr(C)]
pub struct StorageConfig {
    dir: *const u8,
    dir_len: usize,
    access_data_len_mb: usize,
    iters: usize,
}

#[repr(C)]
pub struct StorageReport {
    avail_storage: u64,
    access_seq_avg_t: f64,
    access_rand_avg_t: f64,

    err: *const u8,
    err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_storage(bench: *mut Bench, config: StorageConfig) -> *const StorageReport {
    let bench = unsafe { &mut *bench };
    let report = bench.storage(config.into());

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_storage_report(report: *const StorageReport) {
    unsafe {
        let report = Box::from_raw(report as *mut StorageReport);
        drop_string(report.err, report.err_len);

        drop(report);
    }
}

unsafe fn drop_string(ptr: *const u8, len: usize) {
    if !ptr.is_null() {
        let str = String::from_raw_parts(ptr as *mut u8, len, len);
        drop(str)
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

impl From<RamConfig> for ram::Config {
    fn from(value: RamConfig) -> Self {
        Self {
            alloc: ram::alloc::Config {
                data_len: value.alloc_data_len,
                iters: value.iters,
                ..Default::default()
            },
            access: ram::access::Config {
                data_len: value.access_data_len,
                iters: value.iters,
                ..Default::default()
            },
        }
    }
}

impl From<Result<ram::Report, ram::Error>> for RamReport {
    fn from(value: Result<ram::Report, ram::Error>) -> Self {
        match value {
            Ok(report) => Self {
                total_mem: report.total_mem,
                alloc_avg_t: report.alloc.avg_t.as_secs_f64(),
                access_seq_avg_t: report.access.seq_avg_t.as_secs_f64(),
                access_rand_avg_t: report.access.rand_avg_t.as_secs_f64(),
                access_con_avg_t: report.access.con_avg_t.as_secs_f64(),
                err: null(),
                err_len: 0,
            },
            Err(err) => {
                let err = format!("{err:?}");
                let report = Self {
                    total_mem: 0,
                    alloc_avg_t: 0.,
                    access_seq_avg_t: 0.,
                    access_rand_avg_t: 0.,
                    access_con_avg_t: 0.,
                    err: err.as_ptr(),
                    err_len: err.len(),
                };

                std::mem::forget(err);

                report
            },
        }
    }
}

impl From<StorageConfig> for storage::Config {
    fn from(value: StorageConfig) -> Self {
        let dir = unsafe { str::from_utf8(slice::from_raw_parts(value.dir, value.dir_len)).unwrap() };

        Self {
            access: storage::access::Config {
                dir: PathBuf::from(dir),
                data_len_mb: value.access_data_len_mb,
                iters: value.iters,
                ..Default::default()
            },
        }
    }
}

impl From<Result<storage::Report, storage::Error>> for StorageReport {
    fn from(value: Result<storage::Report, storage::Error>) -> Self {
        match value {
            Ok(report) => Self {
                avail_storage: report.avail_storage,
                access_seq_avg_t: report.access.seq_avg_t.as_secs_f64(),
                access_rand_avg_t: report.access.rand_avg_t.as_secs_f64(),
                err: null(),
                err_len: 0,
            },
            Err(err) => {
                let err = format!("{err:?}");
                let report = Self {
                    avail_storage: 0,
                    access_seq_avg_t: 0.,
                    access_rand_avg_t: 0.,
                    err: err.as_ptr(),
                    err_len: err.len(),
                };

                std::mem::forget(err);

                report
            },
        }
    }
}