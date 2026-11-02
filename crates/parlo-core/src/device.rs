#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub cores: usize,
    pub memory_gb: u64,
    pub avx2: bool,
}

impl Device {
    pub fn describe(&self) -> String {
        let mut parts = vec![format!(
            "{} {}",
            self.cores,
            if self.cores == 1 { "core" } else { "cores" }
        )];
        if self.memory_gb > 0 {
            parts.push(format!("{} GB", self.memory_gb));
        }
        if cfg!(target_arch = "x86_64") {
            parts.push(if self.avx2 {
                "AVX2".to_string()
            } else {
                "no AVX2".to_string()
            });
        }
        parts.join(", ")
    }
}

pub fn profile() -> Device {
    let cores = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    Device {
        cores,
        memory_gb: total_memory_gb().unwrap_or(0),
        avx2: avx2(),
    }
}

#[cfg(target_arch = "x86_64")]
fn avx2() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
fn avx2() -> bool {
    false
}

#[cfg(windows)]
fn total_memory_gb() -> Option<u64> {
    #[repr(C)]
    struct Status {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut Status) -> i32;
    }

    let mut status = Status {
        length: std::mem::size_of::<Status>() as u32,
        memory_load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page_file: 0,
        avail_page_file: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended_virtual: 0,
    };
    let done = unsafe { GlobalMemoryStatusEx(&mut status) };
    if done == 0 || status.total_phys == 0 {
        return None;
    }
    Some(status.total_phys / (1024 * 1024 * 1024))
}

#[cfg(not(windows))]
fn total_memory_gb() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_describes_itself_without_inventing_a_memory_figure() {
        let device = Device {
            cores: 8,
            memory_gb: 0,
            avx2: true,
        };
        let described = device.describe();
        assert!(described.contains("8 cores"));
        assert!(!described.contains("GB"));
    }

    #[test]
    fn a_described_machine_names_its_memory_and_its_instruction_set() {
        let device = Device {
            cores: 1,
            memory_gb: 16,
            avx2: false,
        };
        let described = device.describe();
        assert!(described.contains("1 core"));
        assert!(described.contains("16 GB"));
        if cfg!(target_arch = "x86_64") {
            assert!(described.contains("no AVX2"));
        }
    }

    #[test]
    fn this_machine_reports_at_least_one_core() {
        assert!(profile().cores >= 1);
    }
}
