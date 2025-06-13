# Demo: ehatrom detect --all

This document demonstrates how the new `detect --all` functionality works on a Linux system with multiple I2C devices.

## Example Output on Raspberry Pi

```bash
$ sudo ./ehatrom detect --all
Found 3 I2C device(s): ["/dev/i2c-0", "/dev/i2c-1", "/dev/i2c-20"]

=== Scanning /dev/i2c-0 ===
Scanning I2C bus /dev/i2c-0 for HAT EEPROM...
Checking addresses: [80]
Trying 0x50... Found HAT EEPROM!
First 16 bytes: [52, 2D, 50, 69, 01, 00, 02, 00, 70, 00, 52, 00, 11, 00, 01, 00]
EEPROM found at 0x50 on /dev/i2c-0
EEPROM info:
Eeprom {
    header: EepromHeader {
        signature: [82, 45, 80, 105],  // "R-Pi"
        version: 1,
        reserved: 0,
        numatoms: 2,
        eeplen: 112,
    },
    vendor_info: VendorInfoAtom { 
        vendor_id: 21321, 
        product_id: 19792, 
        product_ver: 1, 
        vendor: "SimpleHAT", 
        product: "TestBoard", 
        uuid: [0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255] 
    },
    // ... more fields
}

=== Scanning /dev/i2c-1 ===
Scanning I2C bus /dev/i2c-1 for HAT EEPROM...
Checking addresses: [80]
Trying 0x50... read error: Remote I/O error (os error 121)
No valid Raspberry Pi HAT EEPROM found on bus /dev/i2c-1

=== Scanning /dev/i2c-20 ===
Scanning I2C bus /dev/i2c-20 for HAT EEPROM...
Checking addresses: [80]
Trying 0x50... no HAT signature (first 4 bytes: [00, 00, 00, 00])
No valid Raspberry Pi HAT EEPROM found on bus /dev/i2c-20
```

## Example Output when No HAT is Found

```bash
$ sudo ./ehatrom detect --all
Found 2 I2C device(s): ["/dev/i2c-0", "/dev/i2c-1"]

=== Scanning /dev/i2c-0 ===
Scanning I2C bus /dev/i2c-0 for HAT EEPROM...
Checking addresses: [80]
Trying 0x50... read error: Remote I/O error (os error 121)
No valid Raspberry Pi HAT EEPROM found on bus /dev/i2c-0

=== Scanning /dev/i2c-1 ===
Scanning I2C bus /dev/i2c-1 for HAT EEPROM...
Checking addresses: [80]
Trying 0x50... read error: Remote I/O error (os error 121)
No valid Raspberry Pi HAT EEPROM found on bus /dev/i2c-1

No HAT EEPROM found on any I2C device.
This could mean:
  • No HAT is connected
  • HAT EEPROM is not programmed
  • HAT uses a different I2C address
  • Permissions issue (try running with sudo)
```

## Usage Scenarios

### 1. Quick Detection (Default Behavior)
```bash
# Scan the standard HAT I2C bus
$ sudo ehatrom detect
```

### 2. Specific Device
```bash
# Scan a specific I2C device
$ sudo ehatrom detect /dev/i2c-1
```

### 3. Auto-Discovery (New Feature)
```bash
# Automatically find and scan all I2C devices
$ sudo ehatrom detect --all
```

## Benefits

1. **User-Friendly**: No need to know which I2C bus the HAT is connected to
2. **Comprehensive**: Scans all available I2C devices automatically
3. **Informative**: Shows all available I2C devices even if no HAT is found
4. **Safe**: Proper error handling for each device
5. **Standards Compliant**: Follows HAT specification (looks for R-Pi signature at 0x50)
