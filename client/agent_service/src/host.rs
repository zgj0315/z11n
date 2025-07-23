use sysinfo::System;

use crate::proto::SystemInfo;

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
    let system = SystemInfo {
        name,
        kernel_version,
        os_version,
        host_name,
        cpu_arch,
        kernel_long_version,
        total_memory,
        total_swap,
    };
    Ok(system)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_test() -> anyhow::Result<()> {
        let _ = tracing_subscriber::fmt().with_ansi(true).try_init();
        system()?;
        Ok(())
    }
}
