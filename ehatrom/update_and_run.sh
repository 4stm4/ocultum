# 4STM4 ocultum
#!/bin/bash
set -e

echo "=== git pull ==="
git pull

echo "=== build ==="
cargo build --release --features=linux

# Create EEPROM files
cargo run --example create_simple
cargo run --example create_test
cargo run --example create_advanced
cargo run --example create_custom_atoms

# Show created files
for f in tests/data/simple.eep tests/data/test.eep tests/data/advanced.eep tests/data/custom_atoms.eep; do
  if [ -f "$f" ]; then
    ./target/release/ehatrom show "$f"
  else
    echo "File not found: $f"
  fi
done

# Detect EEPROM HAT
if command -v i2cdetect >/dev/null 2>&1; then
  sudo i2cdetect -y 1 | grep -E "(50|UU)" || echo "EEPROM not found at 0x50"
else
  echo "i2cdetect not found, installing i2c-tools..."
  sudo apt-get update && sudo apt-get install -y i2c-tools
fi

# Work with real EEPROM
I2C_DEVICE="/dev/i2c-1"
EEPROM_ADDR="0x50"
BACKUP_FILE="eeprom_backup_$(date +%Y%m%d_%H%M%S).bin"
TEST_FILE="tests/data/test.eep"

if sudo ./target/release/ehatrom read "$I2C_DEVICE" "$EEPROM_ADDR" "$BACKUP_FILE" 2>/dev/null; then
  echo "EEPROM backup saved: $BACKUP_FILE"
  ./target/release/ehatrom show "$BACKUP_FILE" || echo "EEPROM data invalid"
  if sudo ./target/release/ehatrom write "$I2C_DEVICE" "$EEPROM_ADDR" "$TEST_FILE"; then
    echo "Test EEPROM written"
    VERIFY_FILE="eeprom_verify.bin"
    if sudo ./target/release/ehatrom read "$I2C_DEVICE" "$EEPROM_ADDR" "$VERIFY_FILE"; then
      ./target/release/ehatrom show "$VERIFY_FILE"
      if sudo ./target/release/ehatrom write "$I2C_DEVICE" "$EEPROM_ADDR" "$BACKUP_FILE"; then
        echo "EEPROM restored from backup"
        rm -f "$VERIFY_FILE"
      else
        echo "ERROR: Failed to restore EEPROM! Backup: $BACKUP_FILE"
      fi
    else
      echo "ERROR: Failed to read EEPROM for verification"
    fi
  else
    echo "ERROR: Failed to write test EEPROM"
  fi
else
  echo "ERROR: Failed to read EEPROM. Check HAT connection, I2C, permissions."
  ./target/release/ehatrom show "$TEST_FILE"
fi
