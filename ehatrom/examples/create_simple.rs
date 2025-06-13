fn main() {
    // Create a minimal valid EEPROM header
    let mut data = Vec::new();

    // Signature: "R-Pi"
    data.extend_from_slice(b"R-Pi");
    // Version: 1
    data.push(1);
    // Reserved: 0
    data.push(0);
    // Number of atoms: 0 (Little Endian)
    data.extend_from_slice(&0u16.to_le_bytes());
    // EEPROM length: 16 (header + CRC, Little Endian)
    data.extend_from_slice(&16u32.to_le_bytes());

    // Add CRC32 (will be calculated by the library, but we need something)
    data.extend_from_slice(&0u32.to_le_bytes());

    std::fs::write("tests/data/simple.eep", data).expect("Failed to write test file");
    println!("Created tests/data/simple.eep");
}
