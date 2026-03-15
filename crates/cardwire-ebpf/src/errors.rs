use thiserror::Error;
use std::io;
#[derive(Error, Debug)]

pub enum CardwireBPFError {
    #[error("LSM Not Enabled")]
    LSMNotEnabled,
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
}