use crate::{Bench, CpuFeatures};

#[derive(Debug)]
pub struct Auxval {
    pub(crate) hwcap: u64,
    pub(crate) hwcap2: u64,

    pub(crate) sve_mask: AuxvalMask,
    pub(crate) i8mm_mask: AuxvalMask,
}

impl Auxval {
    pub(crate) fn sve(&self) -> bool {
        self.is_supported(&self.sve_mask)
    }

    pub(crate) fn i8mm(&self) -> bool {
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

#[derive(Debug)]
pub enum AuxvalMask {
    HWCAP(u64),
    HWCAP2(u64),
}

impl Bench {
    pub(crate) fn with_auxval(total_ram: u64, avail_storage: u64, auxval: Auxval) -> Self {
        Self::with_features(total_ram, avail_storage, CpuFeatures { 
            num_cores: num_cpus::get(),
            sve: auxval.sve(), 
            i8mm: auxval.i8mm(), 
        })
    }
}