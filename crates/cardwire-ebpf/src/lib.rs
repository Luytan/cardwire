mod errors;

use aya::maps::{HashMap, MapError};
use aya::programs::Lsm;
use aya::{Btf, Ebpf};
use flate2::read::GzDecoder;
use std::fs::File;
use std::io::{BufReader, Error as IoError, ErrorKind, Read};
use crate::errors::CardwireBPFError;

pub struct EbpfBlocker {
    ebpf: Ebpf,
}

impl EbpfBlocker {
    fn missing_entity(kind: &str, name: &str) -> IoError {
        IoError::new(ErrorKind::NotFound, format!("{} not found: {}", kind, name))
    }
    /*
        Checks if bpf/lsm is enabled in the kernel
     */
    fn is_bpf_enabled() -> Result<(), CardwireBPFError> {
        // Method 1
        if let Ok(lsm) = std::fs::read_to_string("/sys/kernel/security/lsm") {
            match lsm.contains("bpf"){
                true => return Ok(()),
                false => return Err(CardwireBPFError::LSMNotEnabled)
            };
        };

        // Method 2
        let file = match File::open("/proc/config.gz") {
            Ok(f) => f,
            Err(_) => return Err(CardwireBPFError::LSMNotEnabled),
        };

        let file = BufReader::new(file);
        let mut gz = GzDecoder::new(file);
        let mut config = String::new();

        gz.read_to_string(&mut config)?;

        match config.contains("CONFIG_BPF_LSM=y"){
            true => return Ok(()),
            false => return Err(CardwireBPFError::LSMNotEnabled)
        }
    }

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::is_bpf_enabled()?;
        let mut ebpf = Ebpf::load_file(concat!(env!("OUT_DIR"), "/bpf.o"))?;

        let btf = Btf::from_sys_fs()?;

        let program: &mut Lsm = ebpf
            .program_mut("file_open")
            .ok_or_else(|| Self::missing_entity("program", "file_open"))?
            .try_into()?;
        program.load("file_open", &btf)?;
        program.attach()?;

        Ok(Self { ebpf })
    }

    fn pci_key(pci: &str) -> [u8; 16] {
        let mut key = [0u8; 16];
        let bytes = pci.as_bytes();
        let len = bytes.len().min(15);
        key[..len].copy_from_slice(&bytes[..len]);
        key[len] = 0;
        key
    }

    pub fn block_id(&mut self, id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut map: HashMap<_, u32, u8> = HashMap::try_from(
            self.ebpf
                .map_mut("BLOCKED_IDS")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_IDS"))?,
        )?;
        map.insert(id, 1, 0)?;
        Ok(())
    }

    pub fn unblock_id(&mut self, id: u32) -> Result<(), Box<dyn std::error::Error>> {
        let mut map: HashMap<_, u32, u8> = HashMap::try_from(
            self.ebpf
                .map_mut("BLOCKED_IDS")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_IDS"))?,
        )?;
        let _ = map.remove(&id);
        Ok(())
    }

    pub fn block_pci(&mut self, pci: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut map: HashMap<_, [u8; 16], u8> = HashMap::try_from(
            self.ebpf
                .map_mut("BLOCKED_PCI")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_PCI"))?,
        )?;
        let key = Self::pci_key(pci);
        map.insert(key, 1, 0)?;
        Ok(())
    }

    pub fn unblock_pci(&mut self, pci: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut map: HashMap<_, [u8; 16], u8> = HashMap::try_from(
            self.ebpf
                .map_mut("BLOCKED_PCI")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_PCI"))?,
        )?;
        let key = Self::pci_key(pci);
        let _ = map.remove(&key);
        Ok(())
    }

    pub fn is_id_blocked(&self, id: u32) -> Result<bool, Box<dyn std::error::Error>> {
        let map: HashMap<_, u32, u8> = HashMap::try_from(
            self.ebpf
                .map("BLOCKED_IDS")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_IDS"))?,
        )?;
        match map.get(&id, 0) {
            Ok(_) => Ok(true),
            Err(MapError::KeyNotFound) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    pub fn is_pci_blocked(&self, pci: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let map: HashMap<_, [u8; 16], u8> = HashMap::try_from(
            self.ebpf
                .map("BLOCKED_PCI")
                .ok_or_else(|| Self::missing_entity("map", "BLOCKED_PCI"))?,
        )?;
        let key = Self::pci_key(pci);
        match map.get(&key, 0) {
            Ok(_) => Ok(true),
            Err(MapError::KeyNotFound) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }
}
