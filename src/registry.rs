use crate::{
    any_value_contains, contains, eq,
    flags::{Flags, Level},
    key_contains, recurse, recurse_into,
    registry_macros::execute_checks,
    rule, starts_with,
};
use windows_registry::LOCAL_MACHINE;

pub fn score_registry(flags: &mut Flags) {
    let rules = vec![
        rule!("HARDWARE\\DESCRIPTION\\System\\BIOS" => {
            eq!("BIOSVendor", "VMware, Inc." => EndAll),
            starts_with!("BIOSVersion", "VMW" => Large),
        }),
        rule!("SYSTEM\\ControlSet001\\Control\\SecureBoot\\Servicing\\DeviceAttributes" => {
            eq!("FirmwareManufacturer", "VMware, Inc." => Large),
        }),
        rule!("BCD00000000\\Objects" => {
            recurse!(
                recurse_into!("Elements" => {
                    recurse!(
                        contains!("Element", "VMware" | "VBOX" => EndAll),
                    )
                })
            ),
        }),
        rule!("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\DriverOperations\\1" => {
            recurse!(
                contains!("Identity", "dual_vmxnet3" => Large),
                contains!("Inf", "vmxnet3.inf" => Large),
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Control\\Class" => {
            recurse!(
                recurse!(
                    eq!("HardwareInformation.ChipType", "VMware" => Large),
                    eq!("HardwareInformation.DacType", "VMware" => Large),
                )
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Services" => {
            recurse!(
                recurse!(
                    contains!("Name", "VirtualBox" => Large),
                )
            )
        }),
        rule!("SYSTEM\\ControlSet001\\Control\\DeviceClasses" => {
            recurse!(
                key_contains!("Ven_VMware_&Prod_VMware_Virtual_S" | "CDRom" => Large),
                recurse!(
                    contains!("DeviceInstance", "Ven_VMware_&Prod_VMware_Virtual_S" => Large),
                )
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Control\\DeviceContainers" => {
            recurse!(
                recurse_into!("BaseContainers" => {
                    recurse!(
                        recurse!(
                            key_contains!(
                                "MFG_VMware__Inc" |
                                "Prod_VMware_Virtual_S" |
                                "Ven_NECVMWare" |
                                "Prod_VMware_SATA_CD01" |
                                "Ven_VMware_" => Large
                            ),
                        )
                    )
                })
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Control\\Video" => {
            recurse!(
                recurse!(
                    contains!("Service", "VBox" => Large),
                    contains!("DeviceDesc", "VirtualBox" => Large),
                    contains!("DriverDesc", "VirtualBox" => Large),
                    contains!("ProviderName", "Oracle" => Large),
                    contains!("InfSection", "VBox" => Large),
                    eq!("HardwareInformation.ChipType", "VMware" => Large),
                    eq!("HardwareInformation.DacType", "VMware" => Large),
                )
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Enum\\PCI" => {
            recurse!(
                recurse!(
                    contains!("DeviceDesc", "vmwarebusdevicedesc" | "VMware VMCI" => Large),
                    contains!("DeviceDesc", "Microsoft PS/2" => Medium)
                )
            ),
        }),
        rule!("SYSTEM\\ControlSet001\\Enum\\SCSI" => {
            recurse!(
                key_contains!(
                    "Ven_NECVMWar" |
                    "Prod_VMware_SATA_CD01" |
                    "Prod_VMware_Virtual_S" => Large
                ),
                recurse!(
                    recurse!(
                        contains!("FriendlyName", "NECVMWar" | "VMware" | "VBOX" => Large),
                        contains!("HardwareID", "VMware" | "VBOX" => Large),
                        contains!("DeviceDesc", "Microsoft PS/2" => Medium)
                    )
                )
            ),
        }),
        rule!("SYSTEM\\DriverDatabase\\DriverPackages" => {
            recurse!(
                recurse_into!("Strings" => {
                    contains!("loc.vmwarebusdevicedesc" => Large),
                }),
                recurse_into!("Descriptors\\PCI" => {
                    recurse!(
                        eq!("Configuration", "vmci.install.x64.NT" => Large),
                        eq!("Description", "%loc.vmwarebusdevicedesc%" => Large),
                    )
                })
            ),
        }),
        rule!("SYSTEM\\HardwareConfig" => {
            recurse!(
                eq!("BIOSVendor", "VMware, Inc." => Large),
                recurse_into!("ComputerIds" => {
                    any_value_contains!("VMware, Inc." | "VMW2" | "VirtualBox" | "Virtual Machine" | "Oracle" => Large)
                })
            ),
        }),
    ];

    for rule in rules {
        if let Ok(root) = LOCAL_MACHINE.open(rule.path) {
            execute_checks(flags, &root, &rule.checks);
        }
    }
}
