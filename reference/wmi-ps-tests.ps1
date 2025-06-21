# System Information WMI Script
# Gathers various WMI information that can help identify VMs and system details

Write-Host "=== System Information WMI Query ===" -ForegroundColor Cyan
Write-Host ""

# Win32_ComputerSystem
Write-Host "--- Win32_ComputerSystem ---" -ForegroundColor Yellow
try {
    $computerSystem = Get-WmiObject -Class Win32_ComputerSystem
    Write-Host "ChassisBootupState: $($computerSystem.ChassisBootupState)"
    Write-Host "ChassisSKUNumber: $($computerSystem.ChassisSKUNumber)"
    Write-Host "KeyboardPasswordStatus: $($computerSystem.KeyboardPasswordStatus)"
    Write-Host "Manufacturer: $($computerSystem.Manufacturer)"
    Write-Host "Model: $($computerSystem.Model)"
    Write-Host "Name: $($computerSystem.Name)"
    Write-Host "PowerOnPasswordStatus: $($computerSystem.PowerOnPasswordStatus)"
    Write-Host "PowerSupplyState: $($computerSystem.PowerSupplyState)"
    Write-Host "ResetCapability: $($computerSystem.ResetCapability)"
    Write-Host "SystemType: $($computerSystem.SystemType)"
    Write-Host "ThermalState: $($computerSystem.ThermalState)"
} catch {
    Write-Host "Error querying Win32_ComputerSystem: $_" -ForegroundColor Red
}
Write-Host ""

# Win32_BIOS
Write-Host "--- Win32_BIOS ---" -ForegroundColor Yellow
try {
    $bios = Get-WmiObject -Class Win32_BIOS
    $bios | Format-List * | Out-String | Write-Host
} catch {
    Write-Host "Error querying Win32_BIOS: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_Card
Write-Host "--- CIM_Card ---" -ForegroundColor Yellow
try {
    $cards = Get-WmiObject -Class CIM_Card
    if ($cards) {
        $count = ($cards | Measure-Object).Count
        Write-Host "Found $count card(s)"
        if ($count -gt 10) {
            Write-Host "Showing first 10 entries (truncated)..." -ForegroundColor Gray
            $cards | Select-Object -First 10 | Format-List * | Out-String | Write-Host
        } else {
            $cards | Format-List * | Out-String | Write-Host
        }
    } else {
        Write-Host "No CIM_Card entries found"
    }
} catch {
    Write-Host "Error querying CIM_Card: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_Chassis
Write-Host "--- CIM_Chassis ---" -ForegroundColor Yellow
try {
    $chassis = Get-WmiObject -Class CIM_Chassis
    if ($chassis) {
        $chassis | Format-List * | Out-String | Write-Host
    } else {
        Write-Host "No CIM_Chassis entries found"
    }
} catch {
    Write-Host "Error querying CIM_Chassis: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_ConnectedTo
Write-Host "--- CIM_ConnectedTo ---" -ForegroundColor Yellow
try {
    $connected = Get-WmiObject -Class CIM_ConnectedTo
    if ($connected) {
        $count = ($connected | Measure-Object).Count
        Write-Host "Found $count connection(s)"
        if ($count -gt 10) {
            Write-Host "Showing first 10 entries (truncated)..." -ForegroundColor Gray
            $connected | Select-Object -First 10 | Format-List * | Out-String | Write-Host
        } else {
            $connected | Format-List * | Out-String | Write-Host
        }
    } else {
        Write-Host "No CIM_ConnectedTo entries found"
    }
} catch {
    Write-Host "Error querying CIM_ConnectedTo: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_HeatPipe
Write-Host "--- CIM_HeatPipe ---" -ForegroundColor Yellow
try {
    $heatpipe = Get-WmiObject -Class CIM_HeatPipe
    if ($heatpipe) {
        $heatpipe | Format-List * | Out-String | Write-Host
    } else {
        Write-Host "No CIM_HeatPipe entries found"
    }
} catch {
    Write-Host "Error querying CIM_HeatPipe: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_PowerSupply
Write-Host "--- CIM_PowerSupply ---" -ForegroundColor Yellow
try {
    $powersupply = Get-WmiObject -Class CIM_PowerSupply
    if ($powersupply) {
        $powersupply | Format-List * | Out-String | Write-Host
    } else {
        Write-Host "No CIM_PowerSupply entries found"
    }
} catch {
    Write-Host "Error querying CIM_PowerSupply: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_Slot
Write-Host "--- CIM_Slot ---" -ForegroundColor Yellow
try {
    $slots = Get-WmiObject -Class CIM_Slot
    if ($slots) {
        $count = ($slots | Measure-Object).Count
        Write-Host "Found $count slot(s)"
        if ($count -gt 20) {
            Write-Host "Showing first 20 entries (truncated)..." -ForegroundColor Gray
            $slots | Select-Object -First 20 | Format-List * | Out-String | Write-Host
        } else {
            $slots | Format-List * | Out-String | Write-Host
        }
    } else {
        Write-Host "No CIM_Slot entries found"
    }
} catch {
    Write-Host "Error querying CIM_Slot: $_" -ForegroundColor Red
}
Write-Host ""

# CIM_UserDevice
Write-Host "--- CIM_UserDevice ---" -ForegroundColor Yellow
try {
    $userdevice = Get-WmiObject -Class CIM_UserDevice
    if ($userdevice) {
        $count = ($userdevice | Measure-Object).Count
        Write-Host "Found $count user device(s)"
        if ($count -gt 10) {
            Write-Host "Showing first 10 entries (truncated)..." -ForegroundColor Gray
            $userdevice | Select-Object -First 10 | Format-List * | Out-String | Write-Host
        } else {
            $userdevice | Format-List * | Out-String | Write-Host
        }
    } else {
        Write-Host "No CIM_UserDevice entries found"
    }
} catch {
    Write-Host "Error querying CIM_UserDevice: $_" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Script Complete ===" -ForegroundColor Cyan