use windows::Win32::System::SystemInformation::{
    self, GetNativeSystemInfo, GetPhysicallyInstalledSystemMemory, PROCESSOR_ARCHITECTURE,
    SYSTEM_INFO, *,
};
use windows_core::PWSTR;

use crate::{flags::Flags, util::inspect};

pub fn score_sysinfo(flags: &mut Flags) -> anyhow::Result<()> {
    let mut memory_in_kilos = 0u64;
    unsafe {
        GetPhysicallyInstalledSystemMemory(&mut memory_in_kilos)?;
    };

    let memory_in_gigs = memory_in_kilos / (1024 * 1024);
    println!("memory installed: {memory_in_gigs}GB");

    match memory_in_gigs {
        0..=2 => flags.extreme_penalty(),
        3..=6 => flags.large_penalty(),
        7 => flags.medium_penalty(),
        8..=12 => {}
        13..=24 => flags.small_bonus(),
        25..=64 => flags.large_bonus(),
        _ => flags.extreme_bonus(),
    };

    let mut system_info = SYSTEM_INFO::default();
    unsafe {
        // Maybe should use GetSystemInfo instead?
        GetNativeSystemInfo(&mut system_info);
    }

    // Only useful field is processors
    // The architecture is the same as host architecture
    println!("Number of Processors: {}", system_info.dwNumberOfProcessors);
    match system_info.dwNumberOfProcessors {
        0..=1 => flags.large_penalty(),
        2 => flags.medium_penalty(),
        3..=4 => {}
        5..=8 => flags.medium_bonus(),
        _ => flags.extreme_bonus(),
    };

    let tick_count_ms = unsafe { GetTickCount() };
    let tick_count_sec = tick_count_ms / 1000;
    println!("Tick Count: {tick_count_sec}s");

    match tick_count_sec {
        0..=60 => flags.extreme_penalty(),
        61..=180 => flags.large_penalty(),
        _ => {}
    }

    // If has valid integrated display like a laptop
    // This is also checked in displays.rs but in a different way
    if matches!(inspect("integrated display size", unsafe { GetIntegratedDisplaySize() }), Ok(size) if size > 256.0)
    {
        flags.medium_bonus();
    }

    // ComputerNameDnsDomain
    // The name of the DNS domain assigned to the local computer. If the local computer is a node in a cluster, lpBuffer receives the DNS domain name of the cluster virtual server.
    // ComputerNameDnsFullyQualified
    // The fully qualified DNS name that uniquely identifies the local computer. This name is a combination of the DNS host name and the DNS domain name, using the form HostName.DomainName. If the local computer is a node in a cluster, lpBuffer receives the fully qualified DNS name of the cluster virtual server.
    // ComputerNameDnsHostname
    // The DNS host name of the local computer. If the local computer is a node in a cluster, lpBuffer receives the DNS host name of the cluster virtual server.
    // ComputerNameNetBIOS
    // The NetBIOS name of the local computer. If the local computer is a node in a cluster, lpBuffer receives the NetBIOS name of the cluster virtual server.
    // ComputerNamePhysicalDnsDomain
    // The name of the DNS domain assigned to the local computer. If the local computer is a node in a cluster, lpBuffer receives the DNS domain name of the local computer, not the name of the cluster virtual server.
    // ComputerNamePhysicalDnsFullyQualified
    // The fully qualified DNS name that uniquely identifies the computer. If the local computer is a node in a cluster, lpBuffer receives the fully qualified DNS name of the local computer, not the name of the cluster virtual server.
    // The fully qualified DNS name is a combination of the DNS host name and the DNS domain name, using the form HostName.DomainName.
    // ComputerNamePhysicalDnsHostname
    // The DNS host name of the local computer. If the local computer is a node in a cluster, lpBuffer receives the DNS host name of the local computer, not the name of the cluster virtual server.
    // ComputerNamePhysicalNetBIOS
    // The NetBIOS name of the local computer. If the local computer is a node in a cluster, lpBuffer receives the NetBIOS name of the local computer, not the name of the cluster virtual server.

    for name_type in [
        ComputerNameDnsDomain, // blank
        ComputerNameDnsFullyQualified, // cooper-desktop
        ComputerNameDnsHostname, // cooper-desktop
        // ComputerNameMax,
        ComputerNameNetBIOS, // COOPER-DESKTOP
        ComputerNamePhysicalDnsDomain, // blank
        ComputerNamePhysicalDnsFullyQualified, // cooper-desktop
        ComputerNamePhysicalDnsHostname, // cooper-desktop
        ComputerNamePhysicalNetBIOS, // COOPER-DESKTOP
    ] {
        let mut name_buffer = [0u16; 256];

        let name_buffer_ptr = PWSTR(name_buffer.as_mut_ptr());
        let mut name_buffer_size = name_buffer.len() as u32;

        unsafe {
            GetComputerNameExW(name_type, Some(name_buffer_ptr), &mut name_buffer_size)?;
        }

        let name = unsafe { name_buffer_ptr.to_string()? };
        println!("Computer Name ({:?}): {}", name_type, name);
    }

    Ok(())
}

// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getcomputernameexa
