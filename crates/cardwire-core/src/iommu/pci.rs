use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;

use super::errors::IommuError;
use super::groups::read_iommu_groups;

#[derive(Clone)]
pub struct Device {
    pub pci_address: String,
    pub iommu_group: usize,
    pub vendor_id: String,
    pub device_id: String,
    pub vendor_name: String,
    pub device_name: String,
    pub driver: String,
    pub class: String,
}

pub fn read_pci_devices() -> Result<HashMap<String, Device>, IommuError> {
    let iommu_groups = read_iommu_groups()?;
    let pci_names = load_pci_name_db(Path::new("/usr/share/hwdata/pci.ids"))?;

    let mut devices_map = HashMap::new();
    for (group_id, group) in iommu_groups {
        for pci_address in group.devices {
            let vendor_id = get_vendor_id(&pci_address)?;
            let device_id = get_device_id(&pci_address)?;

            let vendor_key = normalize_pci_id(&vendor_id);
            let device_key = normalize_pci_id(&device_id);

            let vendor_name = pci_names
                .vendors
                .get(&vendor_key)
                .cloned()
                .unwrap_or_else(|| "unknown vendor".to_string());
            let device_name = pci_names
                .devices
                .get(&(vendor_key.clone(), device_key.clone()))
                .cloned()
                .unwrap_or_else(|| "unknown device".to_string());

            let device = Device {
                pci_address: pci_address.clone(),
                iommu_group: group_id,
                vendor_id,
                device_id,
                vendor_name,
                device_name,
                driver: get_driver(&pci_address),
                class: get_class(&pci_address)?,
            };
            devices_map.insert(pci_address, device);
        }
    }

    Ok(devices_map)
}

fn get_vendor_id(pci_address: &str) -> io::Result<String> {
    read_sysfs_trim(
        Path::new("/sys/bus/pci/devices")
            .join(pci_address)
            .join("vendor"),
    )
}

fn get_device_id(pci_address: &str) -> io::Result<String> {
    read_sysfs_trim(
        Path::new("/sys/bus/pci/devices")
            .join(pci_address)
            .join("device"),
    )
}

fn get_class(pci_address: &str) -> io::Result<String> {
    read_sysfs_trim(
        Path::new("/sys/bus/pci/devices")
            .join(pci_address)
            .join("class"),
    )
}

fn get_driver(pci_address: &str) -> String {
    fs::read_link(
        Path::new("/sys/bus/pci/devices")
            .join(pci_address)
            .join("driver"),
    )
    .ok()
    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    .unwrap_or_else(|| "none".to_string())
}

fn read_sysfs_trim(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path).map(|content| content.trim_end().to_string())
}

#[derive(Default)]
struct PciNameDb {
    vendors: HashMap<String, String>,
    devices: HashMap<(String, String), String>,
}

fn load_pci_name_db(path: &Path) -> io::Result<PciNameDb> {
    if !path.exists() {
        return Ok(PciNameDb::default());
    }

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut db = PciNameDb::default();
    let mut current_vendor: Option<String> = None;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if !line.starts_with('\t') {
            current_vendor = parse_pci_ids_line(&line).map(|(vendor_id, vendor_name)| {
                db.vendors.insert(vendor_id.clone(), vendor_name);
                vendor_id
            });
            continue;
        }

        if !line.starts_with("\t\t")
            && let (Some(vendor_id), Some((device_id, device_name))) = (
                current_vendor.as_ref(),
                parse_pci_ids_line(line.trim_start_matches('\t')),
            )
        {
            db.devices
                .insert((vendor_id.clone(), device_id), device_name);
        }
    }

    Ok(db)
}

fn parse_pci_ids_line(line: &str) -> Option<(String, String)> {
    let mut split = line.split_whitespace();
    let raw_id = split.next()?;
    if raw_id.len() != 4 || !raw_id.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let name = split.collect::<Vec<_>>().join(" ");
    if name.is_empty() {
        return None;
    }

    Some((raw_id.to_ascii_lowercase(), name))
}

fn normalize_pci_id(raw: &str) -> String {
    raw.trim()
        .trim_start_matches("0x")
        .trim_start_matches("0X")
        .to_ascii_lowercase()
}
