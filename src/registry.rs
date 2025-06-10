use windows_registry::LOCAL_MACHINE;

use crate::flags::Flags;

// TODO need to add virtualbox, parallels, etc. keys
pub fn score_registry(flags: &mut Flags) -> anyhow::Result<()> {
    if let Ok(bios) = LOCAL_MACHINE.open("HARDWARE\\DESCRIPTION\\System\\BIOS") {
        if check_eq(&bios.get_string("BIOSVendor"), "VMware, Inc.") {
            flags.end_all_penalty();
        }

        if check_starts_with(&bios.get_string("BIOSVersion"), "VMW") {
            flags.large_penalty();
        }
    }

    if let Ok(device_attributes) = LOCAL_MACHINE
        .open("SYSTEM\\ControlSet001\\Control\\SecureBoot\\Servicing\\DeviceAttributes")
    {
        if check_eq(
            &device_attributes.get_string("FirmwareManufacturer"),
            "VMware, Inc.",
        ) {
            flags.large_penalty();
        }
    }

    let _ = check_objects(flags);
    let _ = check_drivers(flags);
    let _ = check_control_devices(flags);
    let _ = check_device_classes(flags);
    let _ = check_device_containers(flags);
    let _ = check_device_control_video(flags);
    let _ = check_control_enums(flags);
    let _ = check_driver_packages(flags);
    let _ = check_hardware_configs(flags);

    Ok(())
}

fn check_objects(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\BCD00000000\Objects\{2c6bd1b8-fcaf-11ef-b1b9-b5f21293fd47}\Elements\12000004
    let root = LOCAL_MACHINE.open("BCD00000000\\Objects")?;

    for key in root.keys()? {
        let Ok(key) = root.open(key) else { continue };
        let Ok(elements_root) = key.open("Elements") else {
            continue;
        };

        let Ok(elements) = elements_root.keys() else {
            continue;
        };

        for element_container in elements {
            let Ok(element_container) = elements_root.open(element_container) else {
                continue;
            };

            if check_eq_arr(
                &element_container.get_string("Element"),
                &[
                    "EFI VMware Virtual SCSI Hard Drive (0.0)",
                    "EFI VMware Virtual SATA CDROM Drive (1.0)",
                ],
            ) {
                flags.end_all_penalty();
            }
        }
    }

    Ok(())
}

fn check_drivers(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\DriverOperations\1\1714
    let root = LOCAL_MACHINE.open("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Component Based Servicing\\DriverOperations\\1")?;
    for key in root.keys()? {
        let Ok(key) = root.open(key) else { continue };
        if check_contains(&key.get_string("Identity"), "dual_vmxnet3") {
            flags.large_penalty();
        }

        if check_contains(&key.get_string("Inf"), "vmxnet3.inf") {
            flags.large_penalty();
        }
    }

    Ok(())
}

fn check_control_devices(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\0000
    let root = LOCAL_MACHINE.open("SYSTEM\\ControlSet001\\Control\\Class")?;
    for class in root.keys()? {
        let Ok(class_key) = root.open(class) else {
            continue;
        };
        let Ok(sub_classes) = class_key.keys() else {
            continue;
        };
        for sub_class in sub_classes {
            let Ok(sub_class_key) = class_key.open(sub_class) else {
                continue;
            };
            if check_eq(
                &sub_class_key.get_string("HardwareInformation.ChipType"),
                "VMware",
            ) {
                flags.large_penalty();
            }

            if check_eq(
                &sub_class_key.get_string("HardwareInformation.DacType"),
                "VMware",
            ) {
                flags.large_penalty();
            }
        }
    }

    Ok(())
}

fn check_device_classes(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Control\DeviceClasses\{53f56307-b6bf-11d0-94f2-00a0c91efb8b}\##?#SCSI#Disk&Ven_VMware_&Prod_VMware_Virtual_S#5&1ec51bf7&0&000000#{53f56307-b6bf-11d0-94f2-00a0c91efb8b}
    let root = LOCAL_MACHINE.open("SYSTEM\\ControlSet001\\Control\\DeviceClasses")?;
    for class in root.keys()? {
        let Ok(class_key) = root.open(class) else {
            continue;
        };
        let Ok(sub_classes) = class_key.keys() else {
            continue;
        };
        for sub_class in sub_classes {
            if contains_arr(&sub_class, &["Ven_VMware_&Prod_VMware_Virtual_S"]) {
                flags.large_penalty();
            }

            let Ok(sub_class_key) = class_key.open(&sub_class) else {
                continue;
            };

            if check_contains(
                &sub_class_key.get_string("DeviceInstance"),
                "Ven_VMware_&Prod_VMware_Virtual_S",
            ) {
                flags.large_penalty();
            }
        }
    }

    Ok(())
}

fn check_device_containers(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Control\DeviceContainers\{00000000-0000-0000-FFFF-FFFFFFFFFFFF}\BaseContainers\{00000000-0000-0000-FFFF-FFFFFFFFFFFF}]
    let root = LOCAL_MACHINE.open("SYSTEM\\ControlSet001\\Control\\DeviceContainers")?;
    for container in root.keys()? {
        let Ok(container_key) = root.open(container) else {
            continue;
        };
        let Ok(base_containers) = container_key.open("BaseContainers") else {
            continue;
        };

        for base_container in base_containers.keys()? {
            let Ok(base_container_key) = base_containers.open(base_container) else {
                continue;
            };
            let Ok(keys) = base_container_key.keys() else {
                continue;
            };

            for key in keys {
                if contains_arr(
                    &key,
                    &[
                        "MFG_VMware__Inc",
                        "Prod_VMware_Virtual_S",
                        "Ven_NECVMWare",
                        "Prod_VMware_SATA_CD01",
                        "Ven_VMware_",
                    ],
                ) {
                    flags.large_penalty();
                }
            }
        }
    }
    Ok(())
}

fn check_device_control_video(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Control\Video\{35B4EA8F-FCAF-11EF-BDD8-806E6F6E6963}\0000
    let root = LOCAL_MACHINE.open("SYSTEM\\ControlSet001\\Control\\Video")?;
    for video_key in root.keys()? {
        let Ok(video_key) = root.open(video_key) else {
            continue;
        };
        let Ok(sub_keys) = video_key.keys() else {
            continue;
        };

        for sub_key in sub_keys {
            let Ok(sub_key) = video_key.open(sub_key) else {
                continue;
            };

            if check_eq(
                &sub_key.get_string("HardwareInformation.ChipType"),
                "VMware",
            ) {
                flags.large_penalty();
            }

            if check_eq(&sub_key.get_string("HardwareInformation.DacType"), "VMware") {
                flags.large_penalty();
            }
        }
    }

    Ok(())
}

fn check_control_enums(flags: &mut Flags) -> anyhow::Result<()> {
    let root = LOCAL_MACHINE.open("SYSTEM\\ControlSet001\\Enum")?;

    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Enum\PCI\VEN_15AD&DEV_0740&SUBSYS_074015AD&REV_10\3&61aaa01&0&3F
    if let Ok(pci_key_root) = root.open("PCI") {
        for pci_key in pci_key_root.keys()? {
            let Ok(pci_key) = pci_key_root.open(pci_key) else {
                continue;
            };
            let Ok(sub_keys) = pci_key.keys() else {
                continue;
            };

            for sub_key in sub_keys {
                let Ok(sub_key) = pci_key.open(sub_key) else {
                    continue;
                };

                if check_contains_arr(
                    &sub_key.get_string("DeviceDesc"),
                    &["vmwarebusdevicedesc", "VMware VMCI"],
                ) {
                    flags.large_penalty();
                }
            }
        }
    }

    // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Enum\SCSI\CdRom&Ven_NECVMWar&Prod_VMware_SATA_CD01
    let scsi = root.open("SCSI")?;
    for scsi_key in scsi.keys()? {
        if contains_arr(
            &scsi_key,
            &[
                "Ven_NECVMWar",
                "Prod_VMware_SATA_CD01",
                "Prod_VMware_Virtual_S",
            ],
        ) {
            flags.large_penalty();
        }

        let Ok(scsi_key) = scsi.open(scsi_key) else {
            continue;
        };
        let Ok(sub_keys) = scsi_key.keys() else {
            continue;
        };

        // HKEY_LOCAL_MACHINE\SYSTEM\ControlSet001\Enum\SCSI\CdRom&Ven_NECVMWar&Prod_VMware_SATA_CD01\5&260e6d66&0&010000
        for sub_key in sub_keys {
            let Ok(sub_key) = scsi_key.open(sub_key) else {
                continue;
            };

            let Ok(deeper_sub_keys) = sub_key.keys() else {
                continue;
            };

            for deeper_sub_key in deeper_sub_keys {
                let Ok(deeper_sub_key) = sub_key.open(deeper_sub_key) else {
                    continue;
                };

                if check_contains_arr(
                    &deeper_sub_key.get_string("FriendlyName"),
                    &["NECVMWar", "VMware"],
                ) {
                    flags.large_penalty();
                }
            }
        }
    }

    Ok(())
}

fn check_driver_packages(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\DriverDatabase\DriverPackages\pvscsii.inf_amd64_9aaa769bfa6923b1
    let root = LOCAL_MACHINE.open("SYSTEM\\DriverDatabase\\DriverPackages")?;
    for package in root.keys()? {
        let Ok(package_key) = root.open(package) else {
            continue;
        };

        // HKEY_LOCAL_MACHINE\SYSTEM\DriverDatabase\DriverPackages\pvscsii.inf_amd64_9aaa769bfa6923b1\Strings
        if let Ok(strings_key) = package_key.open("Strings") {
            if strings_key.get_string("loc.vmwarebusdevicedesc").is_ok() {
                flags.large_penalty();
            }
        }

        // HKEY_LOCAL_MACHINE\SYSTEM\DriverDatabase\DriverPackages\pvscsii.inf_amd64_9aaa769bfa6923b1\Descriptors\PCI\VEN_15AD&DEV_07C0
        if let Ok(descriptors_key) = package_key.open("Descriptors\\PCI") {
            let Ok(descriptors) = descriptors_key.keys() else {
                continue;
            };

            for descriptor in descriptors {
                let Ok(descriptor_key) = descriptors_key.open(descriptor) else {
                    continue;
                };

                if check_eq(
                    &descriptor_key.get_string("Configuration"),
                    "vmci.install.x64.NT",
                ) {
                    flags.large_penalty();
                }

                if check_eq(
                    &descriptor_key.get_string("Description"),
                    "%loc.vmwarebusdevicedesc%",
                ) {
                    flags.large_penalty();
                }
            }
        }
    }

    Ok(())
}

fn check_hardware_configs(flags: &mut Flags) -> anyhow::Result<()> {
    // HKEY_LOCAL_MACHINE\SYSTEM\HardwareConfig\{fed94d56-fdf8-7d3d-659e-15eb6b73a5d6}
    let root = LOCAL_MACHINE.open("SYSTEM\\HardwareConfig")?;
    for config in root.keys()? {
        let Ok(config_key) = root.open(config) else {
            continue;
        };

        if check_eq(&config_key.get_string("BIOSVendor"), "VMware, Inc.") {
            flags.large_penalty();
        }

        if let Ok(computer_ids) = config_key.open("ComputerIds")
            && let Ok(computer_id_values) = computer_ids.values()
        {
            for (_, value) in computer_id_values {
                let Ok(value) = TryInto::<String>::try_into(value) else {
                    continue;
                };

                if contains_arr(&value, &["VMware, Inc.", "VMW2"]) {
                    flags.large_penalty();
                }
            }
        }
    }

    Ok(())
}

#[inline]
fn check_eq(k: &windows_registry::Result<String>, value: &str) -> bool {
    check_eq_arr(k, &[value])
}

#[inline]
fn check_eq_arr(k: &windows_registry::Result<String>, value: &[&str]) -> bool {
    let Ok(v) = k else { return false };
    for val in value {
        if v.to_lowercase() == val.to_lowercase() {
            return true;
        }
    }

    false
}

#[inline]
fn check_starts_with(k: &windows_registry::Result<String>, value: &str) -> bool {
    check_starts_with_arr(k, &[value])
}

#[inline]
fn check_starts_with_arr(k: &windows_registry::Result<String>, value: &[&str]) -> bool {
    let Ok(v) = k else { return false };
    for val in value {
        if v.to_lowercase().starts_with(&val.to_lowercase()) {
            return true;
        }
    }

    false
}

#[inline]
fn check_contains(k: &windows_registry::Result<String>, value: &str) -> bool {
    check_contains_arr(k, &[value])
}

#[inline]
fn check_contains_arr(k: &windows_registry::Result<String>, value: &[&str]) -> bool {
    let Ok(v) = k else { return false };
    contains_arr(&v, value)
}

#[inline]
fn contains_arr(k: &str, value: &[&str]) -> bool {
    for val in value {
        if k.to_lowercase().contains(&val.to_lowercase()) {
            return true;
        }
    }

    false
}
