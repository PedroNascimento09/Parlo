#[cfg(not(windows))]
use std::fs;

pub struct Hardware {
    pub memory_mb: u64,
    pub free_disk_mb: u64,
    pub threads: u32,
}

impl Hardware {
    pub fn as_json(&self) -> String {
        format!(
            "{{\"memory_mb\":{},\"free_disk_mb\":{},\"threads\":{}}}",
            self.memory_mb, self.free_disk_mb, self.threads
        )
    }
}

pub fn detect() -> Hardware {
    Hardware {
        memory_mb: memory_mb(),
        free_disk_mb: free_disk_mb(&crate::models::models_root()),
        threads: std::thread::available_parallelism()
            .map(|count| count.get() as u32)
            .unwrap_or(1),
    }
}

#[cfg(windows)]
fn memory_mb() -> u64 {
    #[repr(C)]
    struct MemoryStatus {
        length: u32,
        memory_load: u32,
        total_physical: u64,
        available_physical: u64,
        total_page_file: u64,
        available_page_file: u64,
        total_virtual: u64,
        available_virtual: u64,
        available_extended_virtual: u64,
    }

    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatus) -> i32;
    }

    let mut status = MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        memory_load: 0,
        total_physical: 0,
        available_physical: 0,
        total_page_file: 0,
        available_page_file: 0,
        total_virtual: 0,
        available_virtual: 0,
        available_extended_virtual: 0,
    };
    if unsafe { GlobalMemoryStatusEx(&mut status) } == 0 {
        return 0;
    }
    status.total_physical / (1024 * 1024)
}

#[cfg(not(windows))]
fn memory_mb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1)?.parse::<u64>().ok())
        })
        .map(|kilobytes| kilobytes / 1024)
        .unwrap_or(0)
}

#[cfg(windows)]
fn free_disk_mb(path: &std::path::Path) -> u64 {
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
        fn GetDiskFreeSpaceExW(
            directory: *const u16,
            available: *mut u64,
            total: *mut u64,
            total_free: *mut u64,
        ) -> i32;
    }

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let mut available: u64 = 0;
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;
    if unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, &mut total, &mut total_free) }
        == 0
    {
        return 0;
    }
    available / (1024 * 1024)
}

#[cfg(not(windows))]
fn free_disk_mb(_path: &std::path::Path) -> u64 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_machine_reports_what_it_has() {
        let hardware = detect();
        assert!(hardware.threads >= 1);
        assert!(hardware.as_json().contains("\"threads\""));
    }
}
