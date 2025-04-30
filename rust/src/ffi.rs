use std::{fmt::Debug, ptr::null, time::Duration};

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
        }.transpose().map_err(err_to_string)
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
    tps: f64,

    err: *const u8,
    err_len: usize,
}

#[no_mangle]
pub extern "C" fn bench_cpu(bench: *mut Bench, config: CpuConfig) -> *const CpuReport {
    let bench = unsafe { &mut *bench };
    let report = __bench_cpu(bench, config);

    Box::into_raw(Box::new(report.into()))
}

fn __bench_cpu(bench: &mut Bench, config: CpuConfig) -> Result<CpuCombinedReports, String> {
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

fn __bench_cpu_multithread(bench: &mut Bench, config: CpuConfig) -> Result<CpuCombinedReports, String> {
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
        drop_string(report.err, report.err_len);

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
    ops: f64,

    err: *const u8,
    err_len: usize,
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

fn __bench_ram(bench: &mut Bench, config: RamConfig) -> Result<RamCombinedReports, String> {
    let alloc_report = bench!(bench, ram, alloc, config)?;
    let access_report = bench!(bench, ram, access, config)?;

    let report = (alloc_report, access_report);

    Ok(report)
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

    access_seq_iters: usize,
    access_seq_data_len_mb: usize,

    access_rand_iters: usize,
    access_rand_data_len_mb: usize,
}

#[repr(C)]
pub struct StorageReport {
    avail_storage: u64,
    ops: f64,

    err: *const u8,
    err_len: usize,
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

fn __bench_storage(bench: &mut Bench, config: StorageConfig) -> Result<StorageCombinedReports, String> {
    let access_report = bench!(bench, storage, access, config)?;

    let report = access_report;

    Ok(report)
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
            _ => Self::HWCAP2(value.v),
        }
    }
}

trait Score {
    fn score(&self) -> f64;
}

fn inv_t_score(score_t: Duration) -> f64 {
    let secs = score_t.as_secs_f64();
    if secs <= 0. {
        0.
    } else {
        1. / secs
    }
}

macro_rules! unpack_report {
    ($rep:expr, f($val:ident : $def:expr)) => {{
        match &$rep {
            Some(rep) => rep.$val,
            None => $def,
        }
    }};
    ($rep:expr, m($val:ident : $def:expr)) => {{
        match &$rep {
            Some(rep) => rep.$val(),
            None => $def,
        }
    }};
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
    Option<cpu::crypto::Report>,
    Option<cpu::math::Report>,
    Option<cpu::sort::Report>,
);

impl From<Result<CpuCombinedReports, String>> for CpuReport {
    fn from(value: Result<CpuCombinedReports, String>) -> Self {
        let (tps, err, err_len) = match value {
            Ok(reports) => (reports.score(), null(), 0),
            Err(err) => (0., err.as_ptr(), err.len()),
        };

        Self {
            tps,

            err,
            err_len,
        }
    }
}

impl Score for CpuCombinedReports {
    fn score(&self) -> f64 {
        let crypto = unpack_report!(self.0, f(tps: 0.));
        let math = unpack_report!(self.1, f(tps: 0.));
        let sort = unpack_report!(self.2, f(tps: 0.));

        (crypto + math + sort) / 3.
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

impl From<(u64, Result<RamCombinedReports, String>)> for RamReport {
    fn from(value: (u64, Result<RamCombinedReports, String>)) -> Self {
        let (ops, err, err_len) = match value.1 {
            Ok((alloc_report, access_report)) => ((alloc_report, access_report).score(), null(), 0),
            Err(err) => (0., err.as_ptr(), err.len()),
        };

        Self {
            total_mem: value.0,
            ops,

            err,
            err_len,
        }
    }
}

impl Score for ram::alloc::Report {
    fn score(&self) -> f64 {
        inv_t_score(self.avg_t)
    }
}

impl Score for ram::access::Report {
    fn score(&self) -> f64 {
        let seq = inv_t_score(self.seq_avg_t);
        let rand = inv_t_score(self.rand_avg_t);
        let concurr = inv_t_score(self.concurr_avg_t);

        (seq + rand + concurr) / 3.
    }
}

impl Score for (Option<ram::alloc::Report>, Option<ram::access::Report>) {
    fn score(&self) -> f64 {
        let alloc = unpack_report!(self.0, m(score: 0.));
        let access = unpack_report!(self.1, m(score: 0.));

        (alloc + access) / 2.
    }
}

macro_rules! impl_from_storage_config {
    ($typ:ident, $((($src_iters:ident : $tar_iters:ident), ($src_data_len:ident : $tar_data_len:ident))),*) => {
        impl From<&StorageConfig> for Option<storage::$typ::Config> {
            fn from(value: &StorageConfig) -> Self {
                if $((value.$src_iters > 0 && value.$src_data_len > 0))||* {
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

type StorageCombinedReports = Option<storage::access::Report>;

impl From<(u64, Result<StorageCombinedReports, String>)> for StorageReport {
    fn from(value: (u64, Result<StorageCombinedReports, String>)) -> Self {
        let (ops, err, err_len) = match value.1 {
            Ok(access_report) => (access_report.score(), null(), 0),
            Err(err) => (0., err.as_ptr(), err.len()),
        };

        Self {
            avail_storage: value.0,
            ops,

            err,
            err_len,
        }
    }
}

impl Score for storage::access::Report {
    fn score(&self) -> f64 {
        let seq = inv_t_score(self.seq_avg_t);
        let rand = inv_t_score(self.rand_avg_t);

        (seq + rand) / 2.
    }
}

impl Score for Option<storage::access::Report> {
    fn score(&self) -> f64 {
        let access = unpack_report!(self, m(score: 0.));

        access
    }
}

fn err_to_string<E: Debug>(err: E) -> String {
    format!("{err:?}")
}