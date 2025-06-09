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
    use ehatrom::{Eeprom, read_from_eeprom_i2c, write_to_eeprom_i2c};
    use std::env;
    use std::process;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: ehatrom <read|write|info> [options]");
        process::exit(1);
    }
    match args[1].as_str() {
        "read" => {
            // ehatrom read <i2c-dev> <address> <output.bin>
            if args.len() != 5 {
                eprintln!("Usage: ehatrom read <i2c-dev> <address> <output.bin>");
                process::exit(1);
            }
            let dev = &args[2];
            let addr = u16::from_str_radix(&args[3].trim_start_matches("0x"), 16).unwrap_or_else(|_| {
                eprintln!("Invalid address: {}", args[3]);
                process::exit(1);
            });
            let mut buf = vec![0u8; 256];
            #[cfg(target_os = "linux")]
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
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!("I2C read is only supported on Linux");
                process::exit(1);
            }
        }
        "write" => {
            // ehatrom write <i2c-dev> <address> <input.bin>
            if args.len() != 5 {
                eprintln!("Usage: ehatrom write <i2c-dev> <address> <input.bin>");
                process::exit(1);
            }
            let dev = &args[2];
            let addr = u16::from_str_radix(&args[3].trim_start_matches("0x"), 16).unwrap_or_else(|_| {
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
            #[cfg(target_os = "linux")]
            match write_to_eeprom_i2c(&data, dev, addr) {
                Ok(()) => {
                    println!("EEPROM written from {}", args[4]);
                }
                Err(e) => {
                    eprintln!("Write error: {e}");
                    process::exit(1);
                }
            }
            #[cfg(not(target_os = "linux"))]
            {
                eprintln!("I2C write is only supported on Linux");
                process::exit(1);
            }
        }
        "info" => {
            // ehatrom info <input.bin>
            if args.len() != 3 {
                eprintln!("Usage: ehatrom info <input.bin>");
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
                    println!("EEPROM info:\n{:#?}", eeprom);
                }
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Usage: ehatrom <read|write|info> [options]");
            process::exit(1);
        }
    }
}
