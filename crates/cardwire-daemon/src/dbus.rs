/// dbus function to interact with the gpus
use log::{info, warn};
use zbus::{fdo, interface};

use crate::config::Config;
use crate::models::{Daemon, Modes};
use cardwire_core::gpu::{GpuRow, block_gpu, is_gpu_blocked};
#[interface(name = "com.github.luytan.cardwire")]
impl Daemon {
    pub(crate) async fn set_mode(&self, mode: String) -> fdo::Result<String> {
        let mode = match mode.to_ascii_lowercase().as_str() {
            "integrated" => Modes::Integrated,
            "hybrid" => Modes::Hybrid,
            "manual" => Modes::Manual,
            _ => {
                return Err(fdo::Error::InvalidArgs(
                    "Unknown mode. Expected: integrated, hybrid, or manual".to_string(),
                ));
            }
        };
        let mut current_config = self.state.config.write().await;

        match mode {
            // Integrated/Hybrid only works on laptop with two gpus, will decline if the computer has more than 2 gpus
            Modes::Integrated | Modes::Hybrid => {
                if self.state.gpu_list.len() != 2 {
                    return Err(fdo::Error::NotSupported(format!(
                        "{} mode is not supported on computer with more or less than 2 gpus, please use manual mode",
                        mode
                    )));
                }
                // block in integrated mode, unblock in hybrid mode
                let block: bool = mode == Modes::Integrated;
                // Loop to find the non default gpu and block it,
                // TODO: better method?
                let mut blocker = self.state.ebpf_blocker.write().await;
                for gpu in self.state.gpu_list.values() {
                    if !gpu.is_default() {
                        block_gpu(&mut blocker, gpu, block)
                            .map_err(|err| fdo::Error::Failed(err.to_string()))?;
                    };
                }
            }
            // Mode manual should return all gpus to a non blocked state and allow gpu <id> block on/off
            Modes::Manual => {}
        }

        *current_config = Config { mode };

        if let Err(err) = current_config.save_mode_to_config() {
            warn!("Failed to save mode to config: {}", err);
        }
        info!("Switched to {}", mode);
        Ok(format!("Set mode to {}", mode))
    }

    pub(crate) async fn get_mode(&self) -> String {
        let current = self.state.get_mode().await;

        //let available_modes = self.list_mode().await.join(", ");

        format!(
            "Available modes: {} {} {}\nCurrent Mode: {}",
            Modes::Integrated,
            Modes::Hybrid,
            Modes::Manual,
            current
        )
    }

    pub(crate) async fn set_gpu_block(&self, gpu_id: u32, block: bool) -> fdo::Result<String> {
        let gpu = self
            .state
            .gpu_list
            .get(&(gpu_id as usize))
            .ok_or_else(|| fdo::Error::InvalidArgs(format!("Unknown gpu id={}", gpu_id)))?;

        // prevent default gpu from being blocked
        if gpu.is_default() {
            warn!(
                "Cannot set block state for GPU {}: device is marked as default",
                gpu_id
            );
            return Err(fdo::Error::AccessDenied(format!(
                "GPU {} is the default device and cannot be blocked",
                gpu_id
            )));
        }

        let mut blocker = self.state.ebpf_blocker.write().await;
        block_gpu(&mut blocker, gpu, block).map_err(|err| fdo::Error::Failed(err.to_string()))?;

        info!("Set GPU {} ({}) block={}", gpu_id, gpu.pci_address(), block);

        Ok(format!(
            "GPU {} block {}",
            gpu_id,
            if block { "on" } else { "off" }
        ))
    }

    pub(crate) async fn list_gpus(&self) -> Vec<GpuRow> {
        //self.list_gpu_rows().await
        let mut rows = Vec::with_capacity(self.state.gpu_list.len());
        let blocker = self.state.ebpf_blocker.read().await;
        for gpu in self.state.gpu_list.values() {
            let blocked: bool =
                is_gpu_blocked(&blocker, gpu).expect("Couldn't check gpu's lock state");
            rows.push((
                gpu.id(),
                gpu.name().to_string(),
                gpu.pci_address().to_string(),
                gpu.render_node().to_string(),
                gpu.is_default(),
                blocked,
            ));
        }
        rows.sort_by_key(|row| row.0);
        rows
    }
}
