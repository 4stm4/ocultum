//  _  _       _             _  _
// | || |  ___| |_ _ __ ___ | || |
// | || |_/ __| __| '_ ` _ \| || |_
// |__   _\__ | |_| | | | | |__   _|
//   |_| |___/\__|_|_|_| |_|  |_|
//  ___   ___ _   _| | |_ _   _ _ __ ___
// / _ \ / __| | | | | __| | | | '_ ` _ \
//| (_) | (__| |_| | | |_| |_| | | | | | |
// \___/ \___|\__,_|_|\__|\__,_|_| |_| |_|

fn main() {
    // Import I2C functions only on Linux
    use ehatrom::Eeprom;
    #[cfg(all(target_os = "linux", feature = "linux"))]
    use ehatrom::{read_from_eeprom_i2c, write_to_eeprom_i2c};
    use std::env;
    use std::process;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ehatrom <read|write|show|detect> [options]");
        process::exit(1);
    }
    match args[1].as_str() {
        "read" => {
            // ehatrom read <i2c-dev> <address> <output.bin>
            if args.len() != 5 {
                eprintln!("Usage: ehatrom read <i2c-dev> <address> <output.bin>");
                process::exit(1);
            }
            #[cfg(all(target_os = "linux", feature = "linux"))]
            {
                let dev = &args[2];
                let addr = u16::from_str_radix(args[3].trim_start_matches("0x"), 16)
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid address: {}", args[3]);
                        process::exit(1);
                    });
                let buf = vec![0u8; 256];
                let mut buf = buf; // for compatibility with function signature
                match read_from_eeprom_i2c(&mut buf, dev, addr, 0) {
                    Ok(()) => {
                        if let Err(e) = std::fs::write(&args[4], &buf) {
                            eprintln!("Failed to write output: {e}");
                            process::exit(1);
                        }
                        println!("EEPROM read and saved to {}", args[4]);
                    }
                    Err(e) => {
                        eprintln!("Read error: {e}");
                        process::exit(1);
                    }
                }
            }
            #[cfg(not(all(target_os = "linux", feature = "linux")))]
            {
                eprintln!("I2C read is only supported on Linux with --features=linux");
                process::exit(1);
            }
        }
        "write" => {
            // ehatrom write <i2c-dev> <address> <input.bin>
            if args.len() != 5 {
                eprintln!("Usage: ehatrom write <i2c-dev> <address> <input.bin>");
                process::exit(1);
            }
            #[cfg(all(target_os = "linux", feature = "linux"))]
            {
                let dev = &args[2];
                let addr = u16::from_str_radix(args[3].trim_start_matches("0x"), 16)
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid address: {}", args[3]);
                        process::exit(1);
                    });
                let data = match std::fs::read(&args[4]) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("Failed to read input: {e}");
                        process::exit(1);
                    }
                };
                match write_to_eeprom_i2c(&data, dev, addr) {
                    Ok(()) => {
                        println!("EEPROM written from {}", args[4]);
                    }
                    Err(e) => {
                        eprintln!("Write error: {e}");
                        process::exit(1);
                    }
                }
            }
            #[cfg(not(all(target_os = "linux", feature = "linux")))]
            {
                eprintln!("I2C write is only supported on Linux with --features=linux");
                process::exit(1);
            }
        }
        "show" => {
            // ehatrom show <input.bin>
            if args.len() != 3 {
                eprintln!("Usage: ehatrom show <input.bin>");
                process::exit(1);
            }
            let data = match std::fs::read(&args[2]) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Failed to read input: {e}");
                    process::exit(1);
                }
            };
            match Eeprom::from_bytes(&data) {
                Ok(eeprom) => {
                    println!("EEPROM info:\n{eeprom:#?}");
                }
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    process::exit(1);
                }
            }
        }
        "detect" => {
            // ehatrom detect [i2c-dev]
            #[cfg(all(target_os = "linux", feature = "linux"))]
            {
                use ehatrom::detect_and_show_eeprom_info;
                let dev = if args.len() >= 3 {
                    &args[2]
                } else {
                    "/dev/i2c-1" // default
                };
                let possible_addrs = &[0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57];
                let read_len = 1024; // read up to 1KB
                match detect_and_show_eeprom_info(dev, possible_addrs, read_len) {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Detection error: {e}");
                        process::exit(1);
                    }
                }
            }
            #[cfg(not(all(target_os = "linux", feature = "linux")))]
            {
                eprintln!("EEPROM detection is only supported on Linux with --features=linux");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Usage: ehatrom <read|write|show|detect> [options]");
            process::exit(1);
        }
    }
}
