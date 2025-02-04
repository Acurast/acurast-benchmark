use core::fmt;

use crate::utils::GB;

pub(crate) mod access;

pub struct Config {
    pub access: access::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            access: Default::default(),
        }
    }
}

pub struct Report {
    pub avail_storage: u64,
    pub access: access::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "Storage")?;
        writeln!(
            f,
            "{i} available storage {:.2} GB",
            (self.avail_storage as f64) / GB as f64,
        )?;
        write!(f, "{i} {}", self.access)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Access(access::Error),
}
