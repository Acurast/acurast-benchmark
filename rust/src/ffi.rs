use std::{path::PathBuf, ptr::null, slice, str, time::Duration};

use crate::{
    arm::{Auxval, AuxvalMask},
    cpu, ram, storage, Bench,
};

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
    let bench = Bench::with_auxval(
        total_ram,
        avail_storage,
        Auxval {
            hwcap,
            hwcap2,
            sve_mask: sve_mask.into(),
            i8mm_mask: i8mm_mask.into(),
        },
    );

    Box::into_raw(Box::new(bench))
}

#[no_mangle]
pub extern "C" fn drop_bench(bench: *mut Bench) {
    unsafe { drop(Box::from_raw(bench)) };
}

macro_rules! bench {
    ($bench:expr, $mod:ident, $typ:ident, $config:expr) => {{
        match (&$config).into() {
            Some(config) => Some($bench.$mod.$typ(config)),
            None => None,
        }
    }};
}

#[repr(C)]
pub struct CpuConfig {
    crypto_duration: usize,
    crypto_data_len: usize,
    
    math_duration: usize,
    math_data_len: usize,

    sort_duration: usize,
    sort_data_len: usize,
}

#[repr(C)]
pub struct CpuReport {
    crypto_tps: f64,
    crypto_err: *const u8,
    crypto_err_len: usize,

    math_tps: f64,
    math_err: *const u8,
    math_err_len: usize,

    sort_tps: f64,
    sort_err: *const u8,
    sort_err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_cpu(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let crypto_report = bench!(bench, cpu, crypto, config);
    let math_report = bench!(bench, cpu, math, config);
    let sort_report = bench!(bench, cpu, sort, config);

    let report = (crypto_report, math_report, sort_report);

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn bench_cpu_multithread(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let crypto_report = bench!(bench, cpu, crypto_multithread, config);
    let math_report = bench!(bench, cpu, math_multithread, config);
    let sort_report = bench!(bench, cpu, sort_multithread, config);

    let report = (crypto_report, math_report, sort_report);

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_cpu_report(report: *const CpuReport) {
    unsafe {
        let report = Box::from_raw(report as *mut CpuReport);
        drop_string(report.crypto_err, report.crypto_err_len);
        drop_string(report.math_err, report.math_err_len);
        drop_string(report.sort_err, report.sort_err_len);

        drop(report);
    }
}

#[repr(C)]
pub struct RamConfig {
    alloc_iters: usize,
    alloc_data_len: usize,
    
    access_seq_iters: usize,
    access_seq_data_len: usize,

    access_rand_iters: usize,
    access_rand_data_len: usize,

    access_concurr_iters: usize,
    access_concurr_data_len: usize,
}

#[repr(C)]
pub struct RamReport {
    total_mem: u64,

    alloc_avg_t: f64,
    alloc_err: *const u8,
    alloc_err_len: usize,
    
    access_seq_avg_t: f64,
    access_rand_avg_t: f64,
    access_concurr_avg_t: f64,
    access_err: *const u8,
    access_err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_ram(bench: *mut Bench, config: RamConfig) -> *const RamReport {
    let bench = unsafe { &mut *bench };
    let total_mem = bench.ram.total_mem();
    let alloc_report = bench!(bench, ram, alloc, config);
    let access_report = bench!(bench, ram, access, config);

    let report = (total_mem, alloc_report, access_report);

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_ram_report(report: *const RamReport) {
    unsafe {
        let report = Box::from_raw(report as *mut RamReport);
        drop_string(report.alloc_err, report.alloc_err_len);
        drop_string(report.access_err, report.access_err_len);

        drop(report);
    }
}

#[repr(C)]
pub struct StorageConfig {
    dir: *const u8,
    dir_len: usize,

    access_seq_iters: usize,
    access_seq_data_len_mb: usize,

    access_rand_iters: usize,
    access_rand_data_len_mb: usize,
}

#[repr(C)]
pub struct StorageReport {
    avail_storage: u64,

    access_seq_avg_t: f64,
    access_rand_avg_t: f64,
    access_err: *const u8,
    access_err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_storage(bench: *mut Bench, config: StorageConfig) -> *const StorageReport {
    let bench = unsafe { &mut *bench };
    let avail_storage = bench.storage.avail_storage();
    let access_report = bench!(bench, storage, access, config);

    let report = (avail_storage, access_report);

    Box::into_raw(Box::new(report.into()))
}

#[no_mangle]
pub extern "C" fn drop_storage_report(report: *const StorageReport) {
    unsafe {
        let report = Box::from_raw(report as *mut StorageReport);
        drop_string(report.access_err, report.access_err_len);

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
            _ => Self::HWCAP2(value.v),
        }
    }
}

macro_rules! unpack_report {
    ($rep:expr, $(($val:ident : $def:expr)),*) => {{
        match $rep {
            Some(Ok(report)) => ($(report.$val),*, null(), 0),
            Some(Err(err)) => {
                let err = format!("{err:?}");
                ($($def),*, err.as_ptr(), err.len())
            },
            None => ($($def),*, null(), 0),
        }   
    }};
    ($rep:expr) => {
        unpack_ram_report!($rep, avg_t)
    }
}


macro_rules! impl_from_cpu_config {
    ($typ:ident, $duration:ident, ($src_data_len:ident : $tar_data_len:ident)) => {
        impl From<&CpuConfig> for Option<cpu::$typ::Config> {
            fn from(value: &CpuConfig) -> Self {
                if value.$duration > 0 && value.$src_data_len > 0 {
                    Some(cpu::$typ::Config {
                        duration: Duration::from_millis(value.$duration as u64),
                        $tar_data_len: value.$src_data_len.try_into().unwrap(),
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        }
    };
    ($typ:ident, $duration:ident, $src_data_len:ident) => {
        impl_from_cpu_config!($typ, $duration, ($src_data_len : data_len));
    };
}

impl_from_cpu_config!(crypto, crypto_duration, crypto_data_len);
impl_from_cpu_config!(math, math_duration, (math_data_len: n));
impl_from_cpu_config!(sort, sort_duration, sort_data_len);

type CpuCombinedReports = (
    Option<Result<cpu::crypto::Report, cpu::crypto::Error>>,
    Option<Result<cpu::math::Report, cpu::math::Error>>,
    Option<Result<cpu::sort::Report, cpu::sort::Error>>,
);

macro_rules! unpack_cpu_report {
    ($rep:expr) => {
        unpack_report!($rep, (tps: 0.))
    };
}

impl From<CpuCombinedReports> for CpuReport {
    fn from(value: CpuCombinedReports) -> Self {
        let (crypto_tps, crypto_err, crypto_err_len) = unpack_cpu_report!(value.0);
        let (math_tps, math_err, math_err_len) = unpack_cpu_report!(value.1);
        let (sort_tps, sort_err, sort_err_len) = unpack_cpu_report!(value.2);

        Self {
            crypto_tps,
            crypto_err,
            crypto_err_len,

            math_tps,
            math_err,
            math_err_len,

            sort_tps,
            sort_err,
            sort_err_len,
        }
    }
}

macro_rules! impl_from_ram_config {
    ($typ:ident, $((($src_iters:ident : $tar_iters:ident), ($src_data_len:ident : $tar_data_len:ident))),*) => {
        impl From<&RamConfig> for Option<ram::$typ::Config> {
            fn from(value: &RamConfig) -> Self {
                if $(value.$src_iters > 0 && value.$src_data_len > 0)&&* {
                    Some(ram::$typ::Config {
                        $(
                            $tar_iters: value.$src_iters,
                            $tar_data_len: value.$src_data_len.try_into().unwrap(),
                        )*
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        }
    };
    ($typ:ident, $iters:ident, $src_data_len:ident) => {
        impl_from_ram_config!($typ, (($iters : iters), ($src_data_len : data_len)));
    };
}

impl_from_ram_config!(alloc, alloc_iters, alloc_data_len);
impl_from_ram_config!(
    access,
    ((access_seq_iters: seq_iters), (access_seq_data_len: seq_data_len)),
    ((access_rand_iters: rand_iters), (access_rand_data_len: rand_data_len)),
    ((access_concurr_iters: concurr_iters), (access_concurr_data_len: concurr_data_len))
);

type RamCombinedReports = (
    u64,
    Option<Result<ram::alloc::Report, ram::alloc::Error>>,
    Option<Result<ram::access::Report, ram::access::Error>>,
);

macro_rules! unpack_ram_report {
    ($rep:expr, $($val:ident),*) => {
        unpack_report!($rep, $(($val: Duration::ZERO)),*)
    };
    ($rep:expr) => {
        unpack_ram_report!($rep, avg_t)
    }
}

impl From<RamCombinedReports> for RamReport {
    fn from(value: RamCombinedReports) -> Self {
        let (alloc_avg_t, alloc_err, alloc_err_len) = unpack_ram_report!(value.1);
        let (access_seq_avg_t, access_rand_avg_t, access_concurr_avg_t, access_err, access_err_len) = unpack_ram_report!(
            value.2,
            seq_avg_t,
            rand_avg_t,
            concurr_avg_t
        );

        Self {
            total_mem: value.0,

            alloc_avg_t: alloc_avg_t.as_secs_f64(),
            alloc_err,
            alloc_err_len,

            access_seq_avg_t: access_seq_avg_t.as_secs_f64(),
            access_rand_avg_t: access_rand_avg_t.as_secs_f64(),
            access_concurr_avg_t: access_concurr_avg_t.as_secs_f64(),
            access_err,
            access_err_len,
        }
    }
}

macro_rules! impl_from_storage_config {
    ($typ:ident, $((($src_iters:ident : $tar_iters:ident), ($src_data_len:ident : $tar_data_len:ident))),*) => {
        impl From<&StorageConfig> for Option<storage::$typ::Config> {
            fn from(value: &StorageConfig) -> Self {
                if $(value.$src_iters > 0 && value.$src_data_len > 0)&&* {
                    Some(storage::$typ::Config {
                        $(
                            $tar_iters: value.$src_iters,
                            $tar_data_len: value.$src_data_len.try_into().unwrap(),
                        )*
                        ..Default::default()
                    })
                } else {
                    None
                }
            }
        }
    };
    ($typ:ident, $iters:ident, $src_data_len:ident) => {
        impl_from_storage_config!($typ, (($iters : iters), ($src_data_len : data_len)));
    };
}

impl_from_storage_config!(
    access,
    ((access_seq_iters: seq_iters), (access_seq_data_len_mb: seq_data_len_mb)),
    ((access_rand_iters: rand_iters), (access_rand_data_len_mb: rand_data_len_mb))
);

type StorageCombinedReports = (
    u64,
    Option<Result<storage::access::Report, storage::access::Error>>,
);

macro_rules! unpack_storage_report {
    ($rep:expr, $($val:ident),*) => {
        unpack_report!($rep, $(($val: Duration::ZERO)),*)
    };
    ($rep:expr) => {
        unpack_ram_report!($rep, avg_t)
    }
}

impl From<StorageCombinedReports> for StorageReport {
    fn from(value: StorageCombinedReports) -> Self {
        let (access_seq_avg_t, access_rand_avg_t, access_err, access_err_len) = unpack_storage_report!(
            value.1,
            seq_avg_t,
            rand_avg_t
        );

        Self {
            avail_storage: value.0,

            access_seq_avg_t: access_seq_avg_t.as_secs_f64(),
            access_rand_avg_t: access_rand_avg_t.as_secs_f64(),
            access_err,
            access_err_len,
        }
    }
}
