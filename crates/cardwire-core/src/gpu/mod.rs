mod discover;
mod ebpf;
mod models;

pub use discover::read_gpu;
pub use ebpf::{block_gpu, is_gpu_blocked};
pub use models::{Gpu, GpuRow};
