use std::{collections::HashMap, ffi::OsString, path::PathBuf};
use sysinfo::{Pid, ProcessStatus, System, Uid};

pub struct Telemetry {
    // cpu
    pub processes: HashMap<Pid, Process>,
    pub cpu_count: usize,
    pub cpu_usage: f32,

    // mem
    pub mem_usage: f32,
    pub mem_total_bytes: u64,
    pub mem_used_bytes: u64,
    pub mem_free_bytes: u64,
    pub mem_total_str: String,
    pub mem_used_str: String,
    pub mem_free_str: String,

    // swap
    pub swap_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_free_bytes: u64,
    pub swap_total_str: String,
    pub swap_used_str: String,
    pub swap_free_str: String,
}

pub struct Process {
    pub name: OsString,
    pub cmd: Vec<OsString>,
    pub exe: Option<PathBuf>,
    pub pid: Pid,
    pub user_id: Option<Uid>,
    pub cwd: Option<PathBuf>,
    pub root: Option<PathBuf>,
    pub status: ProcessStatus,
    pub start_time: u64,
    pub cpu_usage: f32,
    pub disk_usage_bytes: u64,
    pub disk_usage_str: String,
    pub read_bytes: u64,
    pub written_bytes: u64,
}

pub fn get_metrics(sys: &System) -> Telemetry {
    let mut processes: HashMap<Pid, Process> = HashMap::with_capacity(sys.processes().len());

    // clone the processes
    for (pid, proc) in sys.processes() {
        processes.insert(
            *pid,
            Process {
                name: proc.name().to_os_string(),
                cmd: proc.cmd().to_vec(),
                exe: proc.exe().map(|p| p.to_path_buf()),
                pid: *pid,
                user_id: proc.user_id().cloned(),
                cwd: proc.cwd().map(|p| p.to_path_buf()),
                root: proc.root().map(|p| p.to_path_buf()),
                status: proc.status(),
                start_time: proc.start_time(),
                cpu_usage: proc.cpu_usage(),
                disk_usage_bytes: proc.disk_usage().read_bytes + proc.disk_usage().written_bytes,
                disk_usage_str: format_memory(
                    proc.disk_usage().read_bytes + proc.disk_usage().written_bytes,
                ),
                read_bytes: proc.disk_usage().read_bytes,
                written_bytes: proc.disk_usage().written_bytes,
            },
        );
    }

    Telemetry {
        // cpu
        processes,
        cpu_count: sys.cpus().len(),
        cpu_usage: sys.global_cpu_usage(),

        // mem
        mem_usage: if (sys.total_memory() as f32) > 0.0 {
            ((sys.used_memory() as f32) / (sys.total_memory() as f32)) * 100.0
        } else {
            0.0
        },
        mem_total_bytes: sys.total_memory(),
        mem_used_bytes: sys.used_memory(),
        mem_free_bytes: sys.free_memory(),
        mem_total_str: format_memory(sys.total_memory()),
        mem_used_str: format_memory(sys.used_memory()),
        mem_free_str: format_memory(sys.free_memory()),

        // swap
        swap_total_bytes: sys.total_swap(),
        swap_used_bytes: sys.used_swap(),
        swap_free_bytes: sys.free_swap(),
        swap_total_str: format_memory(sys.total_swap()),
        swap_used_str: format_memory(sys.used_swap()),
        swap_free_str: format_memory(sys.free_swap()),
    }
}

pub fn refresh(sys: &mut System) {
    sys.refresh_cpu_usage();
    sys.refresh_memory();
}

fn format_memory(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}
