# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] — 2025-06-13
- **BREAKING**: Removed all serde dependencies (serde, serde_json, serde_yaml, serde-xml-rs)
- **BREAKING**: Removed JSON/YAML/XML CLI commands, kept only `read`, `write`, `show` 
- **NEW**: Custom CRC32 implementation - no external dependencies for CRC32
- **NEW**: Reorganized project structure - all tests in `tests/`, examples in `examples/`
- **NEW**: Enhanced update_and_run.sh script with examples demonstration
- **IMPROVED**: Display implementations for all main types (Eeprom, EepromHeader, AtomHeader, etc.)
- **IMPROVED**: Better error handling and CLI user experience
- **IMPROVED**: Bare-metal compatibility - minimal dependencies
- **IMPROVED**: Comprehensive documentation with usage examples
- **FIXED**: All clippy warnings and formatting issues
- **FIXED**: I2C function import issues with proper feature gating
- Documentation: Updated README, CLI usage examples, Russian comments → English
- Tests: 16 comprehensive tests including performance tests for CRC32
- Zero external dependencies by default, only `i2cdev` with "linux" feature
- CI: Docker and local CI scripts for quality assurance

## [0.1.0] — 2025-06-08
- Initial release: Raspberry Pi HAT EEPROM library
- Full EEPROM (de)serialization, CRC32, I2C (Linux), custom atoms
- 100% test coverage, CLI example, EN/RU docs, CI, Docker, ASCII logo
