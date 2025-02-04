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

#[derive(Debug)]
pub enum Error {
    Crypto(crypto::Error),
    Math(math::Error),
    Sort(sort::Error),
}