use std::path::Path;

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub virt: String, // "physical", "kvm", "vmware", "docker", etc.
}

pub fn detect() -> HostInfo {
    HostInfo {
        model: read_dmi("product_name").or_else(read_dt_model),
        vendor: read_dmi("sys_vendor"),
        virt: detect_virt(),
    }
}

fn read_dmi(field: &str) -> Option<String> {
    let path = format!("/sys/class/dmi/id/{field}");
    let s = std::fs::read_to_string(&path).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() || s == "To Be Filled By O.E.M." || s == "Default string" {
        None
    } else {
        Some(s)
    }
}

fn read_dt_model() -> Option<String> {
    // ARM SBCs (Raspberry Pi, etc.) expose model here.
    let raw = std::fs::read_to_string("/proc/device-tree/model").ok()?;
    let s = raw.trim_end_matches('\0').trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn detect_virt() -> String {
    if Path::new("/.dockerenv").exists() {
        return "docker".into();
    }
    if Path::new("/run/.containerenv").exists() {
        return "podman".into();
    }

    if let Ok(cg) = std::fs::read_to_string("/proc/1/cgroup") {
        if cg.contains("kubepods") {
            return "kubernetes".into();
        }
        if cg.contains("/docker/") {
            return "docker".into();
        }
        if cg.contains("/lxc/") || cg.contains(":/lxc.") {
            return "lxc".into();
        }
    }

    if let Ok(t) = std::fs::read_to_string("/sys/hypervisor/type") {
        let t = t.trim();
        if !t.is_empty() && t != "none" {
            return t.into();
        }
    }

    if let Some(vendor) = read_dmi("sys_vendor") {
        let v = vendor.to_ascii_lowercase();
        if v.contains("qemu") {
            return "kvm".into();
        }
        if v.contains("vmware") {
            return "vmware".into();
        }
        if v.contains("microsoft") {
            return "hyperv".into();
        }
        if v.contains("virtualbox") || v.contains("innotek") {
            return "virtualbox".into();
        }
        if v.contains("xen") {
            return "xen".into();
        }
        if v.contains("bochs") {
            return "bochs".into();
        }
        if v.contains("amazon") {
            return "ec2".into();
        }
        if v.contains("google") {
            return "gcp".into();
        }
    }

    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        if cpuinfo.contains("hypervisor") {
            return "vm".into();
        }
    }

    "physical".into()
}
