use windows::Devices::Usb::UsbDeviceDescriptor;
use windows::Foundation::{GuidHelper, IPropertyValue, IReference};
use windows::Win32::Devices::DeviceAndDriverInstallation;
use windows::{
    Devices::{
        Enumeration::{DeviceInformation, DeviceInformationKind},
        Usb::{self, UsbDevice, UsbDeviceClass},
    },
    Win32::Devices::DeviceAndDriverInstallation::*,
};
use windows_collections::IIterable;
use windows_core::{GUID, HSTRING, Interface, h, w};

use crate::util::get_devices_iter;

pub fn get_usb_devices() -> anyhow::Result<usize> {
    // let mut devices = GUIDS
    //     .iter()
    //     .filter_map(|guid| UsbDevice::GetDeviceSelectorGuidOnly(*guid).ok())
    //     .filter_map(|selector| get_devices_iter(&selector).ok())
    //     .flatten()
    //     .collect::<Vec<_>>();

    let properties: IIterable<HSTRING> = REQUESTED_PROPERTIES
        .iter()
        .map(|s| HSTRING::from(*s))
        .collect::<Vec<HSTRING>>()
        .into();

    let devices_collection =
        DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(
            &HSTRING::new(),
            &properties,
            DeviceInformationKind::DeviceInterface,
        )?
        .get()?;

    let devices_size = devices_collection.Size()? as usize;
    if devices_size == 0 {
        return Ok(0);
    }

    let mut devices = vec![None; devices_size];
    devices_collection.GetMany(0, &mut devices)?;

    println!("Found {} USB devices", devices_size);

    for device in devices.into_iter().flatten() {
        let Ok(properties) = device.Properties() else {
            continue;
        };

            println!("----");
        for property in properties.into_iter() {
            let Ok(key) = property.Key() else {
                continue;
            };

            let Ok(value) = property.Value() else {
                continue;
            };

            println!("property {key:?} {}", value.GetRuntimeClassName().unwrap_or_default());

            let value = match value.cast::<IReference<HSTRING>>().and_then(|v| v.Value()) {
                Ok(value) => value,
                Err(why) => {
                    continue;
                }
            };

            println!("Property: {key:?} = {}", value.to_string_lossy());
        }

        // let protocol_id = match properties.Lookup(h!("System.Devices.Aep.ProtocolId")) {
        //     Ok(protocol_id) => protocol_id,
        //     Err(why) => {
        //         println!("Failed to get ProtocolId: {why:?}");
        //         continue;
        //     }
        // };

        // println!("pass");

        // let protocol_id = match protocol_id.cast::<IReference<GUID>>() {
        //     Ok(protocol_id) => protocol_id,
        //     Err(why) => {
        //         println!("Failed to cast ProtocolId: {why:?}");
        //         continue;
        //     }
        // };

        // println!("{protocol_id:?}");
    }

    // devices.dedup_by(|a, b| a.Id() == b.Id());

    // for device in devices {
    //     println!("Device: {:?}", device);
    // }

    Ok(0)
}

// using System;
// using System.Collections.Generic;
// using System.Management; // reference required

// namespace cSharpUtilities
// {
//     class UsbBrowser
//     {

//         public static void PrintUsbDevices()
//         {
//             IList<ManagementBaseObject> usbDevices = GetUsbDevices();

//             foreach (ManagementBaseObject usbDevice in usbDevices)
//             {
//                 Console.WriteLine("----- DEVICE -----");
//                 foreach (var property in usbDevice.Properties)
//                 {
//                     Console.WriteLine(string.Format("{0}: {1}", property.Name, property.Value));
//                 }
//                 Console.WriteLine("------------------");
//             }
//         }

//         public static IList<ManagementBaseObject> GetUsbDevices()
//         {
//             IList<string> usbDeviceAddresses = LookUpUsbDeviceAddresses();

//             List<ManagementBaseObject> usbDevices = new List<ManagementBaseObject>();

//             foreach (string usbDeviceAddress in usbDeviceAddresses)
//             {
//                 // query MI for the PNP device info
//                 // address must be escaped to be used in the query; luckily, the form we extracted previously is already escaped
//                 ManagementObjectCollection curMoc = QueryMi("Select * from Win32_PnPEntity where PNPDeviceID = " + usbDeviceAddress);
//                 foreach (ManagementBaseObject device in curMoc)
//                 {
//                     usbDevices.Add(device);
//                 }
//             }

//             return usbDevices;
//         }

//         public static IList<string> LookUpUsbDeviceAddresses()
//         {
//             // this query gets the addressing information for connected USB devices
//             ManagementObjectCollection usbDeviceAddressInfo = QueryMi(@"Select * from Win32_USBControllerDevice");

//             List<string> usbDeviceAddresses = new List<string>();

//             foreach(var device in usbDeviceAddressInfo)
//             {
//                 string curPnpAddress = (string)device.GetPropertyValue("Dependent");
//                 // split out the address portion of the data; note that this includes escaped backslashes and quotes
//                 curPnpAddress = curPnpAddress.Split(new String[] { "DeviceID=" }, 2, StringSplitOptions.None)[1];

//                 usbDeviceAddresses.Add(curPnpAddress);
//             }

//             return usbDeviceAddresses;
//         }

//         // run a query against Windows Management Infrastructure (MI) and return the resulting collection
//         public static ManagementObjectCollection QueryMi(string query)
//         {
//             ManagementObjectSearcher managementObjectSearcher = new ManagementObjectSearcher(query);
//             ManagementObjectCollection result = managementObjectSearcher.Get();

//             managementObjectSearcher.Dispose();
//             return result;
//         }

//     }

// }

const GUIDS: &[GUID] = &[
    GUID_BUS_RESOURCE_UPDATE_INTERFACE,
    GUID_BUS_TYPE_1394,
    GUID_BUS_TYPE_ACPI,
    GUID_BUS_TYPE_AVC,
    GUID_BUS_TYPE_DOT4PRT,
    GUID_BUS_TYPE_EISA,
    GUID_BUS_TYPE_HID,
    GUID_BUS_TYPE_INTERNAL,
    GUID_BUS_TYPE_IRDA,
    GUID_BUS_TYPE_ISAPNP,
    GUID_BUS_TYPE_LPTENUM,
    GUID_BUS_TYPE_MCA,
    GUID_BUS_TYPE_PCI,
    GUID_BUS_TYPE_PCMCIA,
    GUID_BUS_TYPE_SCM,
    GUID_BUS_TYPE_SD,
    GUID_BUS_TYPE_SERENUM,
    GUID_BUS_TYPE_SW_DEVICE,
    GUID_BUS_TYPE_USB,
    GUID_BUS_TYPE_USBPRINT,
    GUID_D3COLD_AUX_POWER_AND_TIMING_INTERFACE,
    GUID_D3COLD_SUPPORT_INTERFACE,
    GUID_DEVCLASS_1394,
    GUID_DEVCLASS_1394DEBUG,
    GUID_DEVCLASS_61883,
    GUID_DEVCLASS_ADAPTER,
    GUID_DEVCLASS_APMSUPPORT,
    GUID_DEVCLASS_AVC,
    GUID_DEVCLASS_BATTERY,
    GUID_DEVCLASS_BIOMETRIC,
    GUID_DEVCLASS_BLUETOOTH,
    GUID_DEVCLASS_CAMERA,
    GUID_DEVCLASS_CDROM,
    GUID_DEVCLASS_COMPUTEACCELERATOR,
    GUID_DEVCLASS_COMPUTER,
    GUID_DEVCLASS_DECODER,
    GUID_DEVCLASS_DISKDRIVE,
    GUID_DEVCLASS_DISPLAY,
    GUID_DEVCLASS_DOT4,
    GUID_DEVCLASS_DOT4PRINT,
    GUID_DEVCLASS_EHSTORAGESILO,
    GUID_DEVCLASS_ENUM1394,
    GUID_DEVCLASS_EXTENSION,
    GUID_DEVCLASS_FDC,
    GUID_DEVCLASS_FIRMWARE,
    GUID_DEVCLASS_FLOPPYDISK,
    GUID_DEVCLASS_FSFILTER_ACTIVITYMONITOR,
    GUID_DEVCLASS_FSFILTER_ANTIVIRUS,
    GUID_DEVCLASS_FSFILTER_BOTTOM,
    GUID_DEVCLASS_FSFILTER_CFSMETADATASERVER,
    GUID_DEVCLASS_FSFILTER_COMPRESSION,
    GUID_DEVCLASS_FSFILTER_CONTENTSCREENER,
    GUID_DEVCLASS_FSFILTER_CONTINUOUSBACKUP,
    GUID_DEVCLASS_FSFILTER_COPYPROTECTION,
    GUID_DEVCLASS_FSFILTER_ENCRYPTION,
    GUID_DEVCLASS_FSFILTER_HSM,
    GUID_DEVCLASS_FSFILTER_INFRASTRUCTURE,
    GUID_DEVCLASS_FSFILTER_OPENFILEBACKUP,
    GUID_DEVCLASS_FSFILTER_PHYSICALQUOTAMANAGEMENT,
    GUID_DEVCLASS_FSFILTER_QUOTAMANAGEMENT,
    GUID_DEVCLASS_FSFILTER_REPLICATION,
    GUID_DEVCLASS_FSFILTER_SECURITYENHANCER,
    GUID_DEVCLASS_FSFILTER_SYSTEM,
    GUID_DEVCLASS_FSFILTER_SYSTEMRECOVERY,
    GUID_DEVCLASS_FSFILTER_TOP,
    GUID_DEVCLASS_FSFILTER_UNDELETE,
    GUID_DEVCLASS_FSFILTER_VIRTUALIZATION,
    GUID_DEVCLASS_GENERIC,
    GUID_DEVCLASS_GPS,
    GUID_DEVCLASS_HDC,
    GUID_DEVCLASS_HIDCLASS,
    GUID_DEVCLASS_HOLOGRAPHIC,
    GUID_DEVCLASS_IMAGE,
    GUID_DEVCLASS_INFINIBAND,
    GUID_DEVCLASS_INFRARED,
    GUID_DEVCLASS_KEYBOARD,
    GUID_DEVCLASS_LEGACYDRIVER,
    GUID_DEVCLASS_MEDIA,
    GUID_DEVCLASS_MEDIUM_CHANGER,
    GUID_DEVCLASS_MEMORY,
    GUID_DEVCLASS_MODEM,
    GUID_DEVCLASS_MONITOR,
    GUID_DEVCLASS_MOUSE,
    GUID_DEVCLASS_MTD,
    GUID_DEVCLASS_MULTIFUNCTION,
    GUID_DEVCLASS_MULTIPORTSERIAL,
    GUID_DEVCLASS_NET,
    GUID_DEVCLASS_NETCLIENT,
    GUID_DEVCLASS_NETDRIVER,
    GUID_DEVCLASS_NETSERVICE,
    GUID_DEVCLASS_NETTRANS,
    GUID_DEVCLASS_NETUIO,
    GUID_DEVCLASS_NODRIVER,
    GUID_DEVCLASS_PCMCIA,
    GUID_DEVCLASS_PNPPRINTERS,
    GUID_DEVCLASS_PORTS,
    GUID_DEVCLASS_PRIMITIVE,
    GUID_DEVCLASS_PRINTER,
    GUID_DEVCLASS_PRINTERUPGRADE,
    GUID_DEVCLASS_PRINTQUEUE,
    GUID_DEVCLASS_PROCESSOR,
    GUID_DEVCLASS_SBP2,
    GUID_DEVCLASS_SCMDISK,
    GUID_DEVCLASS_SCMVOLUME,
    GUID_DEVCLASS_SCSIADAPTER,
    GUID_DEVCLASS_SECURITYACCELERATOR,
    GUID_DEVCLASS_SENSOR,
    GUID_DEVCLASS_SIDESHOW,
    GUID_DEVCLASS_SMARTCARDREADER,
    GUID_DEVCLASS_SMRDISK,
    GUID_DEVCLASS_SMRVOLUME,
    GUID_DEVCLASS_SOFTWARECOMPONENT,
    GUID_DEVCLASS_SOUND,
    GUID_DEVCLASS_SYSTEM,
    GUID_DEVCLASS_TAPEDRIVE,
    GUID_DEVCLASS_UCM,
    GUID_DEVCLASS_UNKNOWN,
    GUID_DEVCLASS_USB,
    GUID_DEVCLASS_VOLUME,
    GUID_DEVCLASS_VOLUMESNAPSHOT,
    GUID_DEVCLASS_WCEUSBS,
    GUID_DEVCLASS_WPD,
];

const REQUESTED_PROPERTIES: &[&str] = &[
    "System.Devices.GlyphIcon",
    "System.Devices.Aep.AepId",
    "System.Devices.Aep.CanPair",
    "System.Devices.Aep.Category",
    "System.Devices.Aep.ContainerId",
    "System.Devices.Aep.DeviceAddress",
    "System.Devices.Aep.IsConnected",
    "System.Devices.Aep.IsPaired",
    "System.Devices.Aep.IsPresent",
    "System.Devices.Aep.Manufacturer",
    "System.Devices.Aep.ModelId",
    "System.Devices.Aep.ModelName",
    "System.Devices.Aep.ProtocolId",
    "System.Devices.Aep.SignalStrength",
    // these are specific to bluetooth
    "System.Devices.Aep.Bluetooth.LastSeenTime",
    "System.Devices.Aep.Bluetooth.IssueInquiry",
    "System.Devices.Aep.Bluetooth.Le.ActiveScanning",
    "System.Devices.Aep.Bluetooth.Le.AddressType",
    "System.Devices.Aep.Bluetooth.Le.Appearance",
    "System.Devices.Aep.Bluetooth.Le.Appearance.Category",
    "System.Devices.Aep.Bluetooth.Le.Appearance.Subcategory",
    "System.Devices.Aep.Bluetooth.Le.IsConnectable",
    "System.Devices.Aep.Bluetooth.Le.ScanInterval",
    "System.Devices.Aep.Bluetooth.Le.ScanResponse",
    "System.Devices.Aep.Bluetooth.Le.ScanWindow",
];
