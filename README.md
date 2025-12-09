# Rust Anti VM
A new-gen VM detector, using novel techniques and a confidence system, powered by Windows-RS.

Tested again VirtualBox, VMWare, HyperV with specific detections for them. (Should detect other providers too)
Tested against stealh VMs.

### Techniques
- A confidence system, with penalties and bonuses awarded for unique characteristics
- Detect if Windows is activated
  - Check if license is pirated against a known list of keys
- If Windows is set to auto-logon
- If computer has a valid battery
  - Charge rate
  - Design capacity
  - Full charge capacity
  - Remaining capacity
- Any bluetooth adapters, with validations
  - If classic secure connections supported
  - Low energy supposed
  - Different bluetooth roles
  - Advertisement offload
- Any displays, with bonuses for valid data and penalties for suspicious data (with specific checks for VMWare and VirtualBox)
  - Connector type, e.x. HDMI, VGA, DisplayPort
  - Number of displays
  - If display is wireless, internal, or wired
  - Supports HDR
  - Weird display name
  - Physical size
  - Max/min brightness
  - Resolution sizes with checks for weird sizes
- Graphics card detection
  - Checks description, caption, video proccessor, device ID, name, display drivers, "nf_section", adapter DAC type
- Thorough installed apps detection checks through start menu programs, scores based off number of programs
  - Resolves all shortcut files and counts number of valid non-windows programs
  - Checks for Steam and validates based on number of installed games
- Using a Microsoft account (bonus) or local account (penalty)
- OS specific datapoints
  - How long ago was OS installed
    - Does the days since installation metric match different sources
  - If using windows professional
- Any valid printers connected
- Extensive registry checks
  - Custom macro system to enable flexible registry recursion
  - Searches BIOS, Device Attributes, Object Elements, Drivers, Control Set, Serices, Device Classes, Device Containers, Video, PCI, SCSI, Hardware Config
- System info
  - How much ram (awards bonuses and penalties)
  - Number of processors
  - How long has Windows been running/awake for
  - More display checks
  - How large is the installation disk
  - How much space of that disk is used, not including Windows installation
- Enumerate system devices and PCI devices for VM giveaways
- Enumerate USB devices
  - Detect VM devices
  - List of valid manufacters and awards bonus points (e.x. Intel, Razer)
- Physical computer info
  - Manufacturer, Model, Power on password status
  - Wide variety of BIOS info
  - Internal device connections
  - Chassis information
- Valid WiFi adapters, and whether they are connected through WiFi or not

There are no resources to patch a majority of these settings, and VM software makes it difficult or impossible to change these.
You can customize the threshold of detection.
