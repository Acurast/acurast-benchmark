#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::{
    ffi::{CStr, CString},
    fmt::Debug,
    path::PathBuf,
    ptr::null_mut,
    time::Duration
};

use libc::c_char;

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
        }.transpose().map_err(err_to_cstring)
    }};
}

#[repr(C)]
pub struct CpuConfig {
    crypto_duration: usize,
    crypto_data_len: usize,
    
    math_duration: usize,
    math_data_len: usize,
    math_simd: bool,

    sort_duration: usize,
    sort_data_len: usize,
}

#[repr(C)]
pub struct CpuReport {
    crypto_tps: f64,
    math_tps: f64,
    sort_tps: f64,
    score: f64,

    err: *mut c_char,
}

#[no_mangle]
pub extern "C" fn bench_cpu(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let report = __bench_cpu(bench, config);

    Box::into_raw(Box::new(report.into()))
}

fn __bench_cpu(bench: &mut Bench, config: CpuConfig) -> Result<CpuCombinedReports, CString> {
    let crypto_report = bench!(bench, cpu, crypto, config)?;
    let math_report = bench!(bench, cpu, math, config)?;
    let sort_report = bench!(bench, cpu, sort, config)?;

    let report = (
        crypto_report,
        math_report,
        sort_report,
    );

    Ok(report)
}

#[no_mangle]
pub extern "C" fn bench_cpu_multithread(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let report = __bench_cpu_multithread(bench, config);

    Box::into_raw(Box::new(report.into()))
}

fn __bench_cpu_multithread(bench: &mut Bench, config: CpuConfig) -> Result<CpuCombinedReports, CString> {
    let crypto_report = bench!(bench, cpu, crypto_multithread, config)?;
    let math_report = bench!(bench, cpu, math_multithread, config)?;
    let sort_report = bench!(bench, cpu, sort_multithread, config)?;

    let report = (
        crypto_report,
        math_report,
        sort_report,
    );

    Ok(report)
}

#[no_mangle]
pub extern "C" fn drop_cpu_report(report: *const CpuReport) {
    unsafe {
        let report = Box::from_raw(report as *mut CpuReport);
        drop_string(report.err);

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
    access_seq_avg_t: f64,
    access_rand_avg_t: f64,
    access_concurr_avg_t: f64,
    score: f64,

    err: *mut c_char,
}

#[no_mangle]
pub extern "C" fn bench_ram(bench: *mut Bench, config: RamConfig) -> *const RamReport {
    let bench = unsafe { &mut *bench };
    let total_mem = bench.ram.total_mem();
    let report = __bench_ram(bench, config);
    let report = (
        total_mem,
        report,
    );

    Box::into_raw(Box::new(report.into()))
}

fn __bench_ram(bench: &mut Bench, config: RamConfig) -> Result<RamCombinedReports, CString> {
    let alloc_report = bench!(bench, ram, alloc, config)?;
    let access_report = bench!(bench, ram, access, config)?;

    let report = (alloc_report, access_report);

    Ok(report)
}

#[no_mangle]
pub extern "C" fn drop_ram_report(report: *const RamReport) {
    unsafe {
        let report = Box::from_raw(report as *mut RamReport);
        drop_string(report.err);

        drop(report);
    }
}

#[repr(C)]
pub struct StorageConfig {
    dir: *mut c_char,

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
    score: f64,

    err: *mut c_char,
}

#[no_mangle]
pub extern "C" fn bench_storage(bench: *mut Bench, config: StorageConfig) -> *const StorageReport {
    let bench = unsafe { &mut *bench };
    let avail_storage = bench.storage.avail_storage();
    let report = __bench_storage(bench, config);
    let report = (
        avail_storage,
        report,
    );

    Box::into_raw(Box::new(report.into()))
}

fn __bench_storage(bench: &mut Bench, config: StorageConfig) -> Result<StorageCombinedReports, CString> {
    let access_report = bench!(bench, storage, access, config)?;

    let report = access_report;

    Ok(report)
}

#[no_mangle]
pub extern "C" fn drop_storage_report(report: *const StorageReport) {
    unsafe {
        let report = Box::from_raw(report as *mut StorageReport);
        drop_string(report.err);

        drop(report);
    }
}

unsafe fn drop_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let str = CString::from_raw(ptr);
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

fn inv_t_score(score_t: Duration) -> f64 {
    let secs = score_t.as_secs_f64();
    if secs == 0. {
        0.
    } else {
        1. / secs
    }
}

macro_rules! unpack_report {
    ($rep:expr, $(($val:ident : $def:expr)),*) => {{
        match &$rep {
            Some(rep) => ($(rep.$val),*),
            None => ($($def),*),
        }
    }};
}

macro_rules! score {
    ($(($rep:expr => $($val:ident $(: $map:ident)?),*)),*) => {{
        let mut sum = 0.;
        let mut count = 0.;
        $(
            $(
                if let Some(rep) = &$rep {
                    sum += score!(@apply_map rep.$val $(, $map)?);
                    count += 1.;
                };
            )*
        )*
        
        if count == 0. {
            0.
        } else {
            sum / count
        }
    }};
    (@apply_map $val:expr, $map:ident) => {
        $map($val)
    };
    (@apply_map $val:expr) => {
        $val
    };
}

macro_rules! impl_from_cpu_config {
    ($typ:ident, $duration:ident, ($src_data_len:ident : $tar_data_len:ident) $(, ($src_extra:ident : $tar_extra:ident)),*) => {
        impl From<&CpuConfig> for Option<cpu::$typ::Config> {
            fn from(value: &CpuConfig) -> Self {
                if value.$duration > 0 && value.$src_data_len > 0 {
                    Some(cpu::$typ::Config {
                        duration: Duration::from_millis(value.$duration as u64),
                        $tar_data_len: value.$src_data_len.try_into().unwrap(),
                        $(
                            $tar_extra: value.$src_extra,
                        ),*
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
impl_from_cpu_config!(math, math_duration, (math_data_len: n), (math_simd: simd));
impl_from_cpu_config!(sort, sort_duration, sort_data_len);

type CpuCombinedReports = (
    Option<cpu::crypto::Report>,
    Option<cpu::math::Report>,
    Option<cpu::sort::Report>,
);

macro_rules! unpack_cpu_report {
    ($rep:expr) => {
        unpack_report!($rep, (tps: 0.))
    };
}

macro_rules! score_cpu {
    ($($rep:expr),*) => {
        score!($(($rep => tps)),*)
    };
}

impl From<Result<CpuCombinedReports, CString>> for CpuReport {
    fn from(value: Result<CpuCombinedReports, CString>) -> Self {
        let (crypto_tps, math_tps, sort_tps, score, err) = match value {
            Ok((crypto_report, math_report, sort_report)) => {
                let crypto_tps = unpack_cpu_report!(crypto_report);
                let math_tps = unpack_cpu_report!(math_report);
                let sort_tps = unpack_cpu_report!(sort_report);
                let score = score_cpu!(crypto_report, math_report, sort_report);

                (crypto_tps, math_tps, sort_tps, score, null_mut())
            },
            Err(err) => (0., 0., 0., 0., err.into_raw()),
        };

        Self {
            crypto_tps,
            math_tps,
            sort_tps,
            score,

            err,
        }
    }
}

macro_rules! impl_from_ram_config {
    ($typ:ident, $((($src_iters:ident : $tar_iters:ident), ($src_data_len:ident : $tar_data_len:ident))),*) => {
        impl From<&RamConfig> for Option<ram::$typ::Config> {
            fn from(value: &RamConfig) -> Self {
                if $((value.$src_iters > 0 && value.$src_data_len > 0))||* {
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
    Option<ram::alloc::Report>,
    Option<ram::access::Report>,
);

macro_rules! unpack_ram_report {
    ($rep:expr, $($val:ident),*) => {
        unpack_report!($rep, $(($val: Duration::ZERO)),*)
    };
    ($rep:expr) => {
        unpack_ram_report!($rep, avg_t)
    }
}

macro_rules! score_ram {
    ($(($rep:expr => $($val:ident),*)),*) => {
        score!($(($rep => $($val: inv_t_score),*)),*)
    };
}

impl From<(u64, Result<RamCombinedReports, CString>)> for RamReport {
    fn from(value: (u64, Result<RamCombinedReports, CString>)) -> Self {
        let (alloc_avg_t, access_seq_avg_t, access_rand_avg_t, access_concurr_avg_t, score, err) = match value.1 {
            Ok((alloc_report, access_report)) => {
                let alloc_avg_t = unpack_ram_report!(alloc_report);
                let (access_seq_avg_t, access_rand_avg_t, access_concurr_avg_t) = unpack_ram_report!(
                    access_report,
                    seq_avg_t,
                    rand_avg_t,
                    concurr_avg_t
                );
                let score = score_ram!(
                    (alloc_report => avg_t),
                    (access_report => seq_avg_t, rand_avg_t, concurr_avg_t)
                );

                (alloc_avg_t.as_secs_f64(), access_seq_avg_t.as_secs_f64(), access_rand_avg_t.as_secs_f64(), access_concurr_avg_t.as_secs_f64(), score, null_mut())
            },
            Err(err) => (0., 0., 0., 0., 0., err.into_raw()),
        };

        Self {
            total_mem: value.0,
            alloc_avg_t,
            access_seq_avg_t,
            access_rand_avg_t,
            access_concurr_avg_t,
            score,

            err,
        }
    }
}

macro_rules! impl_from_storage_config {
    ($typ:ident, $((($src_iters:ident : $tar_iters:ident), ($src_data_len:ident : $tar_data_len:ident))),*) => {
        impl From<&StorageConfig> for Option<storage::$typ::Config> {
            fn from(value: &StorageConfig) -> Self {
                if $((value.$src_iters > 0 && value.$src_data_len > 0))||* {
                    let default = storage::$typ::Config::default();
                    Some(storage::$typ::Config {
                        dir: unsafe { 
                            CStr::from_ptr(value.dir)
                                .to_str()
                                .map(|s| PathBuf::from(s))
                                .unwrap_or(default.dir)
                        },
                        $(
                            $tar_iters: value.$src_iters,
                            $tar_data_len: value.$src_data_len.try_into().unwrap(),
                        )*
                        ..default
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

type StorageCombinedReports = Option<storage::access::Report>;

macro_rules! unpack_storage_report {
    ($rep:expr, $($val:ident),*) => {
        unpack_report!($rep, $(($val: Duration::ZERO)),*)
    };
    ($rep:expr) => {
        unpack_storage_report!($rep, avg_t)
    }
}

macro_rules! score_storage {
    ($(($rep:expr => $($val:ident),*)),*) => {
        score!($(($rep => $($val: inv_t_score),*)),*)
    };
}

impl From<(u64, Result<StorageCombinedReports, CString>)> for StorageReport {
    fn from(value: (u64, Result<StorageCombinedReports, CString>)) -> Self {
        let (access_seq_avg_t, access_rand_avg_t, score, err) = match value.1 {
            Ok(access_report) => {
                let (access_seq_avg_t, access_rand_avg_t) = unpack_storage_report!(access_report, seq_avg_t, rand_avg_t);
                let score = score_storage!(
                    (access_report => seq_avg_t, rand_avg_t)
                );

                (access_seq_avg_t.as_secs_f64(), access_rand_avg_t.as_secs_f64(), score, null_mut())
            },
            Err(err) => (0., 0., 0., err.into_raw()),
        };

        Self {
            avail_storage: value.0,
            access_seq_avg_t,
            access_rand_avg_t,
            score,

            err,
        }
    }
}

fn err_to_cstring<E: Debug>(err: E) -> CString {
    let str = format!("{err:?}");
    let str = str.replace("\0", "");

    CString::new(str).unwrap()
}