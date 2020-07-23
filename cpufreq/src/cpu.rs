use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::{self, Utf8Error};
use std::{fs, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Discover(#[from] io::Error),

    #[error(transparent)]
    InvalidFrequency(#[from] ParseIntError),

    #[error(transparent)]
    MalformedFrequency(#[from] Utf8Error),

    #[error("cannot find scaling frequency file for CPU: {0}")]
    MissingScalingFile(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct CPU {
    current: PathBuf,
    max: PathBuf,
    min: PathBuf,
}

impl CPU {
    pub fn discover(path: &PathBuf) -> Result<Vec<Self>> {
        path.read_dir()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|path| path.is_dir())
            .filter(|path| {
                let mut freq = path.clone();
                freq.push("cpufreq");
                freq.is_dir()
            })
            .map(|path| Ok(Self::new(&path)?))
            .collect()
    }

    fn new(path: &PathBuf) -> Result<Self> {
        let mut root = path.clone();

        root.push("cpufreq");

        Ok(Self {
            current: path_buf_file(&root, "scaling_cur_freq")?,
            max: path_buf_file(&root, "scaling_max_freq")?,
            min: path_buf_file(&root, "scaling_min_freq")?,
        })
    }

    pub fn current_freq(&self) -> Result<Frequency> {
        file_freq(&self.current)
    }

    pub fn max_freq(&self) -> Result<Frequency> {
        file_freq(&self.max)
    }

    pub fn min_freq(&self) -> Result<Frequency> {
        file_freq(&self.min)
    }
}

fn path_buf_file(root: &PathBuf, file: &str) -> Result<PathBuf> {
    let mut pb = root.clone();

    pb.push(file);

    if pb.exists() {
        Ok(pb)
    } else {
        Err(Error::MissingScalingFile(pb.to_string_lossy().into()))
    }
}

fn file_freq(file: &PathBuf) -> Result<Frequency> {
    Ok(str::from_utf8(fs::read(file)?.as_slice())?
        .trim()
        .parse::<Frequency>()?)
}

pub type Frequency = u64;
