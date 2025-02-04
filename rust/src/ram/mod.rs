use core::fmt;

pub(crate) mod alloc;
pub(crate) mod access;

pub struct Config {
    pub alloc: alloc::Config,
    pub access: access::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            alloc: Default::default(),
            access: Default::default(),
        }
    }
}

pub struct Report {
    pub total_mem: u64,
    pub alloc: alloc::Report,
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "RAM")?;
        writeln!(f, "{i} total mem {} MB", self.total_mem / 1024 / 1024)?;
        writeln!(f, "{i} {}", self.alloc)?;
        write!(f, "{i} {}", self.access)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Alloc(alloc::Error),
    Access(access::Error),
}