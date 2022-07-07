use std::str::FromStr;

use anyhow::bail;

// pub mod bench;
pub mod simple;

pub enum App {
    Simple,
    // Bench,
}

impl App {
    pub fn function(&self) -> &'static dyn Fn(f32) {
        match self {
            App::Simple => &simple::app,
            // App::Bench => &bench::app,
        }
    }
}

impl FromStr for App {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "simple" => Ok(Self::Simple),
            // "bench" => Ok(Self::Bench),
            unknown => bail!("unknown app '{unknown}', included apps are: simple, bench"),
        }
    }
}
