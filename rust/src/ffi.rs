use crate::{Bench, CpuFeatures};

#[derive(uniffi::Record)]
pub struct Auxval {
    hwcap: u64,
    hwcap2: u64,

    aes_mask: AuxvalMask,
    sha2_mask: AuxvalMask,
    sve_mask: AuxvalMask,
    i8mm_mask: AuxvalMask,
}

impl Auxval {
    fn aes(&self) -> bool {
        self.is_supported(&self.aes_mask)
    }

    fn sha2(&self) -> bool {
        self.is_supported(&self.sha2_mask)
    }

    fn sve(&self) -> bool {
        self.is_supported(&self.sve_mask)
    }

    fn i8mm(&self) -> bool {
        self.is_supported(&self.i8mm_mask)
    }

    fn is_supported(&self, mask: &AuxvalMask) -> bool {
        let (vector, mask) = match mask {
            AuxvalMask::HWCAP(mask) => (self.hwcap, mask),
            AuxvalMask::HWCAP2(mask) => (self.hwcap2, mask),
        };

        vector & mask != 0
    }
}

#[derive(uniffi::Enum)]
pub enum AuxvalMask {
    HWCAP(u64),
    HWCAP2(u64),
}

#[derive(uniffi::Object)]
struct FFI {
    bench: Bench,
}

#[uniffi::export]
impl FFI {
    #[uniffi::constructor]
    fn new(auxval: Auxval) -> Self {
        let bench = Bench::new(CpuFeatures { 
            aes: auxval.aes(), 
            sha2: auxval.sha2(), 
            sve: auxval.sve(), 
            i8mm: auxval.i8mm(), 
        });

        Self { bench }
    }
}