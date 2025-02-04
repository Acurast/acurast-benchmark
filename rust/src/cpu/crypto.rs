use std::{fmt, hint::black_box, time::Duration};

use aes::cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit};
use rand::RngCore;
use sha2::Digest;

use crate::{utils::{Expirable, GetValue, Timeout}, CpuFeatures};

type Cipher = aes::Aes256;
type Hasher = sha2::Sha256;

const ENC_KEY_SIZE: usize = 32;
const ENC_BLOCK_SIZE: usize = 16;

const HASH_SIZE: usize = 32;
const EMPTY_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

pub(crate) fn bench(_features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let mut report_builder = ReportBuilder::new(context.timeout.duration);

    while !context.timeout.reached() {
        context.reset_data();

        let bytes = black_box(
            encryption::run_test(
                &context.cipher,
                &context.data[..],
                &mut context.encrypted[..],
                &mut context.decrypted[..],
                Some(&context.timeout),
            )
        );
        if bytes.is_ok() && context.data != context.decrypted {
            return Err(Error::EncryptionMismatch(context.data, context.decrypted));
        }
        report_builder.add(bytes);

        if context.timeout.reached() {
            break;
        }

        let bytes = black_box(
            hash::run_test(
                &mut context.hasher, 
                &context.data[..], 
                &mut context.hash[..],
                Some(&context.timeout),
            )
        );
        if bytes.is_ok() && context.hash == EMPTY_HASH {
            return Err(Error::HashEmpty);
        }
        report_builder.add(bytes);
    }

    Ok(report_builder.build())
}

pub(crate) fn bench_multithread(features: &CpuFeatures, config: Config) -> Result<Report, Error> {
    let mut context = Context::new(config);
    let threadpool = rayon::ThreadPoolBuilder::new().num_threads(features.num_cores).build().unwrap();
    let mut result_builder = ReportBuilder::new(context.timeout.duration);

    while !context.timeout.reached() {
        context.rng.fill_bytes(&mut context.data);
        
        let bytes = black_box(
            encryption::run_test_multithread(
                &threadpool,
                &context.cipher,
                &context.data[..],
                &mut context.encrypted[..],
                &mut context.decrypted[..],
                Some(&context.timeout)
            )
        );
        if bytes.is_ok() && context.data != context.decrypted {
            return Err(Error::EncryptionMismatch(context.data, context.decrypted));
        }
        result_builder.add(bytes);
    }

    Ok(result_builder.build())
}

mod encryption {
    use crate::utils::{slice_from_ptr_mut, AddValue, GetValue};

    use super::*;

    pub(super) fn run_test(
        cipher: &Cipher,
        data: &[u8],
        enc_output: &mut [u8],
        dec_output: &mut [u8],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64> {
        let mut bytes_count = 0u64;
        for i in 0..num_blocks(data) {
            timeout.reached_with_err(bytes_count)?;

            let (data, enc_output, dec_output) = block(data, enc_output.as_mut_ptr(), dec_output.as_mut_ptr(), i);
            bytes_count = encrypt_decrypt_block(cipher, data, enc_output, dec_output, timeout).add(bytes_count)?;
        }

        Ok(bytes_count)
    }
    
    pub(super) fn run_test_multithread(
        threadpool: &rayon::ThreadPool,
        cipher: &Cipher,
        data: &[u8],
        enc_output: &mut [u8],
        dec_output: &mut [u8],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64> {
        let num_blocks = num_blocks(data);
        let mut results = Vec::with_capacity(num_blocks);

        threadpool.install(|| {
            rayon::scope(|s| {
                for i in 0..num_blocks {
                    results.insert(i, Err(0));

                    if timeout.reached() {
                        break;
                    }
        
                    let (data, enc_output, dec_output) = block(data, enc_output.as_mut_ptr(), dec_output.as_mut_ptr(), i);
                    let results = slice_from_ptr_mut(results.as_mut_ptr(), i, i + 1);

                    s.spawn(move |_| {
                        let result = encrypt_decrypt_block(cipher, data, enc_output, dec_output, timeout);
                        results[0] = result;
                    });
                }
            })
        });

        results.into_iter().fold(Ok(0), |acc, next| {
            match (acc, next) {
                (Ok(acc), Ok(next)) => Ok(acc + next),
                _ => Err(acc.value() + next.value()),
            }
        })
    }
    
    fn num_blocks(data: &[u8]) -> usize {
        data.len().div_ceil(ENC_BLOCK_SIZE)
    }

    fn block<'a>(data: &'a[u8], enc_output_ptr: *mut u8, dec_output_ptr: *mut u8, idx: usize) -> (&'a [u8], &'a mut [u8], &'a mut [u8]) {
        let start = idx * ENC_BLOCK_SIZE;
        let end = usize::min(start + ENC_BLOCK_SIZE, data.len());

        let data = &data[start..end];
        let enc_output = slice_from_ptr_mut(enc_output_ptr, start, end);
        let dec_output = slice_from_ptr_mut(dec_output_ptr, start, end);

        (data, enc_output, dec_output)
    }

    fn encrypt_decrypt_block(
        cipher: &Cipher,
        data: &[u8],
        enc_output: &mut [u8],
        dec_output: &mut [u8],
        timeout: Option<&Timeout>,
    ) -> Result<u64, u64> {
        let mut bytes_count = 0u64;

        let data_block = GenericArray::from_slice(data);
        let enc_block = GenericArray::from_mut_slice(enc_output);
        let dec_block = GenericArray::from_mut_slice(dec_output);
    
        cipher.encrypt_block_b2b(data_block, enc_block);
        bytes_count += enc_block.len() as u64;

        timeout.reached_with_err(bytes_count)?;

        cipher.decrypt_block_b2b(enc_block, dec_block);
        bytes_count += dec_block.len() as u64;

        Ok(bytes_count)
    }
}

mod hash {
    use super::*;

    pub(super) fn run_test(
        hasher: &mut Hasher,
        data: &[u8],
        output: &mut [u8],
        _timeout: Option<&Timeout>,
    ) -> Result<u64, u64> {
        let out = GenericArray::from_mut_slice(output);

        hasher.update(&data[..]);
        hasher.finalize_into_reset(out);

        Ok(output.len() as u64)
    }
}

pub struct Config {
    pub rng: Box<dyn rand::RngCore>,

    pub duration: Duration,

    pub enc_key: Option<[u8; ENC_KEY_SIZE]>,
    pub data_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            rng: Box::new(rand::thread_rng()),
            duration: Duration::from_secs(10),
            enc_key: None,
            data_len: 4096,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    EncryptionMismatch(Vec<u8>, Vec<u8>),
    HashEmpty,
}

#[derive(Debug)]
pub struct Report {
    pub duration: Duration,
    pub bytes_count: u64,
    pub tps: f64,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "crypto ... {} bytes/s", self.tps.floor())
    }
}

struct ReportBuilder {
    duration: Duration,
    bytes_count: u64,
}

impl ReportBuilder {
    fn new(duration: Duration) -> Self {
        Self { duration, bytes_count: 0 }
    }

    fn add(&mut self, result: Result<u64, u64>) {
        self.bytes_count += result.value();
    }

    fn build(self) -> Report {
        Report { 
            duration: self.duration,
            bytes_count: self.bytes_count,
            tps: self.bytes_count as f64 / self.duration.as_secs_f64(),
        }
    }
}

struct Context {
    rng: Box<dyn rand::RngCore>,

    cipher: Cipher,
    hasher: Hasher,

    data: Vec<u8>,
    encrypted: Vec<u8>,
    decrypted: Vec<u8>,
    hash: [u8; HASH_SIZE],

    timeout: Timeout,
}

impl Context {
    fn new(mut config: Config) -> Self {
        let key = match config.enc_key {
            Some(key) => key,
            None => {
                let mut key = [0u8; ENC_KEY_SIZE];
                config.rng.fill_bytes(&mut key);
    
                key
            },
        };
        let key = GenericArray::from(key);
        let cipher = Cipher::new(&key);

        let hasher = Hasher::new();

        let data_len = config.data_len - (config.data_len % 16);

        let mut data = Vec::with_capacity(data_len);
        unsafe { data.set_len(data_len) };

        let mut encrypted = Vec::with_capacity(data_len);
        unsafe { encrypted.set_len(data_len) };

        let mut decrypted = Vec::with_capacity(data_len);
        unsafe { decrypted.set_len(data_len) };

        let hash = [0u8; HASH_SIZE];

        let timeout = Timeout::new(config.duration);
    
        Self {
            rng: config.rng,
            cipher,
            hasher,
            data,
            encrypted,
            decrypted,
            hash,
            timeout,
        }
    }

    fn reset_data(&mut self) {
        self.rng.fill_bytes(&mut self.data);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_bench() {
        let duration = Duration::from_millis(1000);
        let start = Instant::now();
        let result = bench(
            &CpuFeatures { num_cores: 1, sve: false, i8mm: false },
            Config {
                duration,
                data_len: 1024,
                ..Default::default()
            },
        );
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.bytes_count > 0);
        assert!(result.tps > 0.);
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
                data_len: 1024,
                ..Default::default()
            },
        );
        let elapsed = start.elapsed();

        assert!(result.is_ok(), "expected success");
        let result = result.unwrap();
        assert!(result.bytes_count > 0);
        assert!(result.tps > 0.);
        assert!(elapsed >= duration && elapsed <= duration + Duration::from_millis(100));

        println!("{result}");
    }

    #[test]
    fn test_encryption() {
        let key= hex!("cc4a401b59245e80b1ccc86d4eea62322b04b0c890488a5a53e7306c2e46517d");
        let data = hex!("42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b");
        let enc_expected = hex!("cde6f2e8b795f296d026564f419c86c0c04f173ecba2da93e6100d8a7b04b3c1");

        let key = GenericArray::from(key);
        let cipher = Cipher::new(&key);

        let mut enc_output = [0u8; 32];
        let mut dec_output = [0u8; 32];

        let result = encryption::run_test(&cipher, &data, &mut enc_output, &mut dec_output, None);

        assert_eq!(true, result.is_ok(), "expected success");
        assert_eq!(64, result.unwrap());
        assert_eq!(enc_output, enc_expected);
        assert_eq!(dec_output, data);
    }

    #[test]
    fn test_encryption_multithread() {
        let key= hex!("cc4a401b59245e80b1ccc86d4eea62322b04b0c890488a5a53e7306c2e46517d");
        let data = hex!("42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b");
        let enc_expected = hex!("cde6f2e8b795f296d026564f419c86c0c04f173ecba2da93e6100d8a7b04b3c1cde6f2e8b795f296d026564f419c86c0c04f173ecba2da93e6100d8a7b04b3c1cde6f2e8b795f296d026564f419c86c0c04f173ecba2da93e6100d8a7b04b3c1cde6f2e8b795f296d026564f419c86c0c04f173ecba2da93e6100d8a7b04b3c1");

        let key = GenericArray::from(key);
        let cipher = Cipher::new(&key);

        let mut enc_output = [0u8; 128];
        let mut dec_output = [0u8; 128];

        let threadpool = rayon::ThreadPoolBuilder::new().num_threads(2).build().unwrap();
        let result = encryption::run_test_multithread(&threadpool, &cipher, &data, &mut enc_output, &mut dec_output, None);

        assert!(result.is_ok(), "expected success");
        assert_eq!(256, result.unwrap());
        assert_eq!(enc_output, enc_expected);
        assert_eq!(dec_output, data);
    }

    #[test]
    fn test_hash() {
        let data = hex!("42de0be8e330b60d3dca3e5ab4f06f54d53ae89c30060236c41f4984a411ea0b");
        let hash_expected = hex!("293ad79b5ee95cfeb84918f4f592f10d280754c6de7ca786cb2f68189e2a8f9e");

        let mut hasher = Hasher::new();

        let mut hash_output = [0u8; HASH_SIZE];

        let result = hash::run_test(&mut hasher, &data, &mut hash_output, None);

        assert!(result.is_ok(), "expected success");
        assert_eq!(HASH_SIZE as u64, result.unwrap());
        assert_eq!(hash_output, hash_expected)
    }
}