use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Gpu {
    pub vendor: String,
    pub model: String,
}

pub fn list() -> Vec<Gpu> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir("/sys/bus/pci/devices") else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let Some(class) = read_hex(&path.join("class")) else {
            continue;
        };
        // PCI base class: 0x03 = display controller (VGA/3D/other).
        if (class >> 16) != 0x03 {
            continue;
        }
        let vendor_id = read_hex(&path.join("vendor")).unwrap_or(0);
        let device_id = read_hex(&path.join("device")).unwrap_or(0);
        out.push(Gpu {
            vendor: vendor_name(vendor_id as u16).to_string(),
            model: format!("{:04x}:{:04x}", vendor_id as u16, device_id as u16),
        });
    }
    out
}

fn read_hex(p: &PathBuf) -> Option<u32> {
    let raw = std::fs::read_to_string(p).ok()?;
    let s = raw.trim().trim_start_matches("0x");
    u32::from_str_radix(s, 16).ok()
}

fn vendor_name(id: u16) -> &'static str {
    match id {
        0x10de => "NVIDIA",
        0x1002 => "AMD",
        0x8086 => "Intel",
        0x1af4 => "Red Hat (virtio)",
        0x1414 => "Microsoft",
        0x15ad => "VMware",
        0x80ee => "VirtualBox",
        0x1234 => "QEMU",
        0x1d0f => "Amazon",
        _ => "Unknown",
    }
}
