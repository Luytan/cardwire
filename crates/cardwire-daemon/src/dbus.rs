/// dbus function to interact with the gpus
use log::{info, warn};
use zbus::{fdo, interface};

use crate::config::Config;
use crate::models::{Daemon, Modes};
use cardwire_core::gpu::GpuRow;
#[interface(name = "com.cardwire.daemon")]
impl Daemon {
    pub(crate) async fn set_mode(&self, mode: Modes) -> fdo::Result<String> {
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
                // 1 if integrated, 0 if hybrid
                let _block_gpu: bool = mode == Modes::Integrated;
                // Loop to find the non default gpu and block it,
                // TODO: better method?
                for gpu in self.state.gpu_list.values() {
                    if !gpu.is_default() {
                        //block dgpu
                    }
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

        format!("Available modes: {}\nCurrent Mode: {}", current, current)
    }

    pub(crate) async fn set_gpu_block(&self, gpu_id: u32, blocked: bool) -> fdo::Result<String> {
        let gpu = self
            .state
            .gpu_list
            .get(&(gpu_id as usize))
            .ok_or_else(|| fdo::Error::InvalidArgs(format!("Unknown gpu id={}", gpu_id)))?;

        // block gpu
        let now_blocked = true;
        info!(
            "Set GPU {} ({}) block={} (effective={})",
            gpu_id,
            gpu.pci_address(),
            blocked,
            now_blocked
        );

        Ok(format!(
            "GPU {} block {} (effective={})",
            gpu_id,
            if blocked { "on" } else { "off" },
            now_blocked
        ))
    }

    pub(crate) async fn get_gpu_info(&self) -> fdo::Result<String> {
        Ok(format!(
            "GPU 1:\n
            Name: Radeon
            "
        ))
    }

    pub(crate) async fn list_gpus(&self) -> Vec<GpuRow> {
        //self.list_gpu_rows().await
        let mut rows = Vec::with_capacity(self.state.gpu_list.len());
        for gpu in self.state.gpu_list.values() {
            let blocked = false;
            rows.push((
                gpu.id() as u32,
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
