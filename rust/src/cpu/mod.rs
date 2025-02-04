use std::fmt;

pub(crate) mod crypto;
pub(crate) mod math;
pub(crate) mod sort;

pub struct Config {
    pub crypto: crypto::Config,
    pub math: math::Config,
    pub sort: sort::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            crypto: Default::default(),
            math: Default::default(),
            sort: Default::default(),
        }
    }
}

pub struct Report {
    pub crypto: crypto::Report,
    pub math: math::Report,
    pub sort: sort::Report,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let i = "::::";
        writeln!(f, "CPU")?;
        writeln!(f, "{i} {}", self.crypto)?;
        writeln!(f, "{i} {}", self.math)?;
        write!(f, "{i} {}", self.sort)?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    Crypto(crypto::Error),
    Math(math::Error),
    Sort(sort::Error),
}
