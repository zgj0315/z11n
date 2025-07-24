use crate::proto::{DiskInfo, NetworkInfo, ProcessInfo, SystemInfo};
use sysinfo::{Disks, Networks, System};

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
        processes.push(ProcessInfo {
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

pub fn disk() -> anyhow::Result<Vec<DiskInfo>> {
    let mut r = Vec::new();
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        let disk = DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            file_system: disk.file_system().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            kind: disk.kind().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            is_removable: disk.is_removable(),
            is_read_only: disk.is_read_only(),
        };
        r.push(disk);
    }

    Ok(r)
}

pub fn network() -> anyhow::Result<Vec<NetworkInfo>> {
    let mut r = Vec::new();
    let networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in &networks {
        let mut addrs = Vec::new();
        for ip_network in data.ip_networks() {
            addrs.push(ip_network.addr.to_string());
        }
        let network = NetworkInfo {
            interface_name: interface_name.to_string(),
            total_received: data.total_received(),
            total_transmitted: data.total_transmitted(),
            addrs,
        };
        r.push(network);
    }
    Ok(r)
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

    #[test]
    fn disk_test() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt().with_ansi(true).try_init();
        let disk = disk()?;
        log::info!("{disk:?}");
        Ok(())
    }

    #[test]
    fn network_test() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt().with_ansi(true).try_init();
        let network = network()?;
        log::info!("{network:?}");
        Ok(())
    }
}
