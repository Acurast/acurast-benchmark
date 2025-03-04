use std::{
    env::temp_dir,
    fmt,
    fs::{remove_file, File, OpenOptions},
    hint::black_box,
    io::{self, Write},
    os::fd::AsRawFd,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[cfg(any(target_os = "macos", target_os = "ios"))]
use libc::{fcntl, F_FULLFSYNC, F_NOCACHE};
#[cfg(any(target_os = "linux", target_os = "android"))]
use libc::{posix_fadvise, POSIX_FADV_DONTNEED};

use rand::Rng;

use crate::{
    utils::{vec_with_len, Avg, MB},
    CpuFeatures,
};

pub(crate) fn bench(_features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.seq_iters);

    let mut start: Instant;
    for _ in 0..context.seq_iters {
        let mut file = context.open_file().map_err(|err| Error::IO(err))?;
        context.reset_write_buf();
        context.reset_read_buf();

        start = Instant::now();
        black_box(sequential::run_test(
            &mut file,
            &mut context.write_buf_mb,
            &mut context.read_buf_mb,
            context.seq_size_mb,
        )?);
        report_builder.add_seq(start.elapsed());

        remove_file(context.file_path.clone()).map_err(|err| Error::IO(err))?;
    }

    for _ in 0..context.rand_iters {
        let mut file = context.open_file().map_err(|err| Error::IO(err))?;
        for _ in 0..context.seq_size_mb {
            file.write_all(&mut context.write_buf_mb)
                .map_err(|err| Error::IO(err))?;
        }
        context.reset_write_buf();
        context.reset_read_buf();

        let write_offsets = context.random_offsets(context.rand_size_mb);
        let read_offsets = context.random_offsets(context.rand_size_mb);

        start = Instant::now();
        black_box(random::run_test(
            &mut file,
            &mut context.write_buf_mb,
            &mut context.read_buf_mb,
            &write_offsets,
            &read_offsets,
        )?);
        report_builder.add_rand(start.elapsed());

        remove_file(context.file_path.clone()).map_err(|err| Error::IO(err))?;
    }

    let _ = remove_file(context.file_path.clone());

    Ok(report_builder.build())
}

mod sequential {
    use std::io::{Read, Seek, Write};

    use super::*;

    pub(super) fn run_test(
        file: &mut File,
        write_buf_mb: &mut [u8],
        read_buf_mb: &mut [u8],
        size_mb: usize,
    ) -> Result<(), Error> {
        for _ in 0..size_mb {
            file.write_all(write_buf_mb).map_err(|err| Error::IO(err))?;
            file.sync_all().map_err(|err| Error::IO(err))?;
        }

        file.rewind().map_err(|err| Error::IO(err))?;

        for _ in 0..size_mb {
            file.read_exact(read_buf_mb).map_err(|err| Error::IO(err))?;
            if write_buf_mb != read_buf_mb {
                return Err(Error::InvalidData(
                    write_buf_mb.to_vec(),
                    read_buf_mb.to_vec(),
                ));
            }
        }

        Ok(())
    }
}

mod random {
    use std::io::{Read, Seek, SeekFrom};

    use super::*;

    pub(super) fn run_test(
        file: &mut File,
        write_buf_mb: &mut [u8],
        read_buf_mb: &mut [u8],
        write_offsets: &[u64],
        read_offsets: &[u64],
    ) -> Result<(), Error> {
        for &offset in write_offsets {
            file.seek(SeekFrom::Start(offset))
                .map_err(|err| Error::IO(err))?;
            file.write_all(write_buf_mb).map_err(|err| Error::IO(err))?;
            file.sync_all().map_err(|err| Error::IO(err))?;
        }

        for &offset in read_offsets {
            file.seek(SeekFrom::Start(offset))
                .map_err(|err| Error::IO(err))?;
            file.read_exact(read_buf_mb).map_err(|err| Error::IO(err))?;

            // there's no trivial way to verify if data is correctly read back,
            // skipping check
        }

        Ok(())
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,

    pub dir: PathBuf,

    pub seq_iters: usize,
    pub seq_data_len_mb: usize,

    pub rand_iters: usize,
    pub rand_data_len_mb: usize,
}

impl Default for Config {
    fn default() -> Self {
        let iters = 10;
        let data_len_mb = 500;

        Self {
            rng: Box::new(rand::thread_rng()),
            dir: temp_dir(),

            seq_iters: iters,
            seq_data_len_mb: data_len_mb,

            rand_iters: iters,
            rand_data_len_mb: data_len_mb,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidData(Vec<u8>, Vec<u8>),
    IO(io::Error),
}

pub struct Report {
    pub seq_avg_t: Duration,
    pub rand_avg_t: Duration,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "sequential read/write ... {:.6} s",
            self.seq_avg_t.as_secs_f64()
        )?;
        write!(
            f,
            "random read/write ... {:.6} s",
            self.rand_avg_t.as_secs_f64()
        )?;

        Ok(())
    }
}

struct ReportBuilder {
    seq_ts: Vec<Duration>,
    rand_ts: Vec<Duration>,
}

impl ReportBuilder {
    fn new(iters: usize) -> Self {
        Self {
            seq_ts: Vec::with_capacity(iters),
            rand_ts: Vec::with_capacity(iters),
        }
    }

    fn add_seq(&mut self, time: Duration) {
        self.seq_ts.push(time);
    }

    fn add_rand(&mut self, time: Duration) {
        self.rand_ts.push(time);
    }

    fn build(self) -> Report {
        Report {
            seq_avg_t: self.seq_ts.avg(),
            rand_avg_t: self.rand_ts.avg(),
        }
    }
}

struct Context {
    rng: Box<dyn rand::RngCore>,

    file_path: PathBuf,

    write_buf_mb: Vec<u8>,
    read_buf_mb: Vec<u8>,

    seq_iters: usize,
    seq_size_mb: usize,
    
    rand_iters: usize,
    rand_size_mb: usize,
}

impl Context {
    fn new(config: Config) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis();
        let mut file_path = config.dir;
        file_path.push(format!("{timestamp}.bench"));

        let write_buf_mb = vec_with_len!(MB);
        let read_buf_mb = Vec::with_capacity(MB);

        Self {
            rng: config.rng,
            file_path,

            write_buf_mb,
            read_buf_mb,

            seq_iters: config.seq_iters,
            seq_size_mb: config.seq_data_len_mb,

            rand_iters: config.rand_iters,
            rand_size_mb: config.rand_data_len_mb,
        }
    }

    fn open_file(&mut self) -> io::Result<File> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(self.file_path.clone())?;

        let fd = file.as_raw_fd();

        #[cfg(any(target_os = "linux", target_os = "android"))]
        unsafe {
            posix_fadvise(fd, 0, 0, POSIX_FADV_DONTNEED);
        };

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        unsafe {
            fcntl(fd, F_NOCACHE, 1);
            fcntl(fd, F_FULLFSYNC);
        };

        Ok(file)
    }

    fn reset_write_buf(&mut self) {
        self.rng.fill_bytes(&mut self.write_buf_mb);
    }

    fn reset_read_buf(&mut self) {
        self.read_buf_mb.clear();
        self.read_buf_mb.resize(MB, 0);
    }

    fn random_offsets(&mut self, size: usize) -> Vec<u64> {
        (0..size)
            .map(|_| self.rng.gen_range(0..self.rand_size_mb) as u64 * MB as u64)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench() {
        let iters = 2;
        let data_len_mb = 10;

        let result = bench(
            &CpuFeatures {
                num_cores: 8,
                sve: false,
                i8mm: false,
            },
            Config {
                seq_iters: iters,
                seq_data_len_mb: data_len_mb,
                rand_iters: iters,
                rand_data_len_mb: data_len_mb,
                ..Default::default()
            },
        );

        if let Err(e) = result {
            println!("{:?}", e);
            return;
        }

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.seq_avg_t > Duration::ZERO);
        assert!(result.rand_avg_t > Duration::ZERO);

        println!("{result}");
    }
}
