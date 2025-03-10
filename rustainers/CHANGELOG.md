# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.15.1](https://github.com/wefoxplatform/rustainers/compare/v0.15.0...v0.15.1) - 2025-02-18

### Added

- support alternative TLS backends (#46)

### Fixed

- enable openssl removal from dependency tree (#49)

### Other

- replace strum by derive_more

## [0.15.0](https://github.com/wefoxplatform/rustainers/compare/v0.14.0...v0.15.0) - 2025-01-26

### Added

- bump dependencies (#45)

### Fixed

- build, test & run on machine with ipv4 & ipv6 enabled (#42)

### Other

- remove async-trait dependency (#44)
- fix mardown syntax (#43)

## [0.14.0](https://github.com/wefoxplatform/rustainers/compare/v0.13.1...v0.14.0) - 2024-12-20

### Added

- Allow using self-signed certificates in HTTPS health tests (#39)

### Fixed

- Add documentation and fix clippy lints (#41)

## [0.13.1](https://github.com/wefoxplatform/rustainers/compare/v0.13.0...v0.13.1) - 2024-09-21

### Added

- allow retrieve host port for generic image ([#32](https://github.com/wefoxplatform/rustainers/pull/32))
- Adds NATS image ([#31](https://github.com/wefoxplatform/rustainers/pull/31))

## [0.13.0](https://github-ilaborie/wefoxplatform/rustainers/compare/v0.12.0...v0.13.0) - 2024-09-11

### Added

- Docker in docker ([#24](https://github-ilaborie/wefoxplatform/rustainers/pull/24))
- add mosquitto image ([#19](https://github-ilaborie/wefoxplatform/rustainers/pull/19))

### Fixed

- minor changes ([#28](https://github-ilaborie/wefoxplatform/rustainers/pull/28))
- Prefix all short-name images for new Podman compatibility ([#26](https://github-ilaborie/wefoxplatform/rustainers/pull/26))

### Other

- Adjust examples for API changes of dependencies
- Update MSRV to 1.70
- Update dependencies to their latest versions
- add more Clippy lint + fix ([#23](https://github-ilaborie/wefoxplatform/rustainers/pull/23))
