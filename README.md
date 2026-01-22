# ruapc-rdma-sys

Low-level Rust FFI bindings to [libibverbs](https://github.com/linux-rdma/rdma-core) for RDMA (Remote Direct Memory Access) operations. This crate provides type-safe device management, safe wrapper types, and JSON serialization support.

This crate is part of the [ruapc](https://github.com/SF-Zhou/ruapc) project.

## Requirements

- Rust 1.85+
- libibverbs development headers (`libibverbs-dev` on Debian/Ubuntu)
- pkg-config

### Installing Dependencies

**Ubuntu/Debian:**
```bash
sudo apt-get install libibverbs-dev pkg-config
```

**Fedora/RHEL:**
```bash
sudo dnf install libibverbs-devel pkg-config
```

**Arch Linux:**
```bash
sudo pacman -S libibverbs pkgconf
```

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
ruapc-rdma-sys = "0.1.0"
```

### Device Discovery

```rust
use ruapc_rdma_sys::Devices;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let devices = Devices::available()?;
    println!("Found {} RDMA device(s)", devices.len());

    for device in &devices {
        let info = device.info();
        println!("  {}: {} ports", info.name, info.ports.len());
    }

    Ok(())
}
```

### Device Filtering

```rust
use ruapc_rdma_sys::{Devices, DeviceConfig, GidType};
use std::collections::HashSet;

let mut config = DeviceConfig::default();
config.device_filter = HashSet::from(["mlx5_0".to_string()]);
config.gid_type_filter = HashSet::from([GidType::RoCEv2]);
config.skip_inactive_port = true;

let devices = Devices::open(&config)?;
```

## CLI Tool

Query RDMA devices from the command line:

```bash
cargo install ruapc-rdma-sys
ruapc-rdma-sys
```

Filter by device or GID type:

```bash
ruapc-rdma-sys -d mlx5_0
ruapc-rdma-sys --gid-types RoCEv2 --skip-inactive
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
