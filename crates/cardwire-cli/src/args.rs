use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Set {
        mode: String,
    },
    Get,
    List,
    Gpu {
        id: u32,
        #[command(subcommand)]
        command: GpuCommands,
    },
}

#[derive(Subcommand)]
pub enum GpuCommands {
    Block { state: String },
    Info,
}
