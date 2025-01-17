#[cfg(any(target_os = "android", target_os = "ios"))]
uniffi::setup_scaffolding!();

#[cfg(any(target_os = "android", target_os = "ios"))]
pub mod ffi;

pub struct CpuFeatures {
    aes: bool,
    sha2: bool,
    sve: bool,
    i8mm: bool,
}

pub struct Bench {
    features: CpuFeatures,
}

impl Bench {
    fn new(features: CpuFeatures) -> Self {
        Self { features }
    }
}