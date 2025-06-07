use windows::Win32::System::SystemInformation::{
    self, GetNativeSystemInfo, GetPhysicallyInstalledSystemMemory, PROCESSOR_ARCHITECTURE,
    SYSTEM_INFO, *,
};

use crate::flags::Flags;

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

    score_system_info(flags)?;

    Ok(())
}

// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getcomputernameexa
// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getintegrateddisplaysize

fn score_system_info(flags: &mut Flags) -> anyhow::Result<()> {
    let mut system_info = SYSTEM_INFO::default();
    unsafe {
        GetNativeSystemInfo(&mut system_info);
    }
    
    score_processor_arch(flags, unsafe { system_info.Anonymous.Anonymous.wProcessorArchitecture });

    debug_system_info(&system_info);
    Ok(())
}

fn score_processor_arch(flags: &mut Flags, arch: PROCESSOR_ARCHITECTURE) {
    // TODO
    // ROCESSOR_ARCHITECTURE_AMD64 x64 (AMD or Intel)
    // PROCESSOR_ARCHITECTURE_ARM ARM
    // PROCESSOR_ARCHITECTURE_ARM64 ARM64
    // PROCESSOR_ARCHITECTURE_IA64 Intel Itanium-based
    // PROCESSOR_ARCHITECTURE_INTEL x86
    match arch {
        PROCESSOR_ARCHITECTURE_ALPHA => {}
        PROCESSOR_ARCHITECTURE_ALPHA64 => {}
        PROCESSOR_ARCHITECTURE_AMD64 => {}
        PROCESSOR_ARCHITECTURE_ARM => {}
        PROCESSOR_ARCHITECTURE_ARM32_ON_WIN64 => {}
        PROCESSOR_ARCHITECTURE_ARM64 => {}
        PROCESSOR_ARCHITECTURE_IA32_ON_WIN64 => {}
        PROCESSOR_ARCHITECTURE_IA32_ON_ARM64 => {}
        PROCESSOR_ARCHITECTURE_IA64 => {}
        PROCESSOR_ARCHITECTURE_INTEL => {}
        PROCESSOR_ARCHITECTURE_MIPS => {}
        PROCESSOR_ARCHITECTURE_MSIL => {}
        PROCESSOR_ARCHITECTURE_NEUTRAL => {}
        PROCESSOR_ARCHITECTURE_PPC => {}
        PROCESSOR_ARCHITECTURE_SHX => {}
        // PROCESSOR_ARCHITECTURE_UNKNOWN => {}
        _ => {}
    }
}

fn debug_system_info(si: &SYSTEM_INFO) {
    println!("Processor Architecture: {:?}", unsafe {
        si.Anonymous.Anonymous.wProcessorArchitecture // desktop: PROCESSOR_ARCHITECTURE_AMD64
    });
    println!("Number of Processors: {}", si.dwNumberOfProcessors); // desktop: 16
}

// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-isprocessorfeaturepresent ???