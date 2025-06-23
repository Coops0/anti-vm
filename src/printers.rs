use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

use crate::flags::Flags;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32Printer {
    name: String, // != "Microsoft Print to PDF"
    printer_state: u32, // 13 = Not Available, 24 = Server_Unknown
    printer_status: u16, // 1 = Other, 2 = Unknown
}

pub fn score_printers(flags: &mut Flags) -> anyhow::Result<()> {
    let com_con = unsafe { COMLibrary::assume_initialized() };
    let wmi_con = WMIConnection::new(com_con)?;

    let mut printers = wmi_con
        .raw_query::<Win32Printer>("SELECT Name, PrinterState, PrinterStatus FROM Win32_Printer")
        .unwrap_or_default();

    printers.retain(is_printer_valid);

    match printers.len() {
        0 => {},
        1 => flags.medium_bonus(),
        _ => flags.large_bonus()
    }

    Ok(())
}

fn is_printer_valid(printer: &Win32Printer) -> bool {
    if printer.name == "Microsoft Print to PDF" {
        return false; 
    }

    if  printer.printer_state == 13 || printer.printer_state == 24 {
        return false; 
    }

    if printer.printer_status == 1 || printer.printer_status == 2 {
        return false; 
    }

    true
}