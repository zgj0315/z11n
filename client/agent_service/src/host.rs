use sysinfo::System;

use crate::proto::{Process, SystemInfo};

pub fn system() -> anyhow::Result<SystemInfo> {
    let mut system = System::new_all();
    system.refresh_all();
    log::info!("{system:?}");
    let name = System::name();
    let kernel_version = System::kernel_version();
    let os_version = System::os_version();
    let host_name = System::host_name();
    let cpu_arch = System::cpu_arch();
    let kernel_long_version = System::kernel_long_version();
    let total_memory = system.total_memory();
    let total_swap = system.total_swap();
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        let pid = pid.as_u32();
        let name = process.name().to_string_lossy().to_string();
        let exe = process.exe().map(|exe| exe.to_string_lossy().to_string());
        let status = process.status().to_string();
        processes.push(Process {
            pid,
            name,
            exe,
            status,
        });
    }
    let system = SystemInfo {
        name,
        kernel_version,
        os_version,
        host_name,
        cpu_arch,
        kernel_long_version,
        total_memory,
        total_swap,
        processes,
    };
    Ok(system)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_test() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt().with_ansi(true).try_init();
        let system = system()?;
        log::info!("{system:?}");
        Ok(())
    }
}
