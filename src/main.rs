//! RDMA device discovery utility
//!
//! Query and display RDMA devices on the system in JSON format.

use clap::Parser;
use ruapc_rdma_sys::{DeviceConfig, Devices, GidType};

#[derive(Parser, Debug)]
#[command(name = "ruapc-rdma-sys")]
#[command(about = "Query and display RDMA devices in JSON format", long_about = None)]
struct Args {
    /// Filter by device name(s)
    #[arg(short = 'd', long, num_args = 0..)]
    devices: Vec<String>,

    /// Filter by GID type(s)
    #[arg(short = 'g', long, num_args = 0..)]
    gid_types: Vec<GidType>,

    /// Skip inactive ports
    #[arg(long)]
    skip_inactive: bool,

    /// Skip RoCEv2 link-local addresses
    #[arg(long)]
    skip_link_local: bool,

    /// Compact JSON output (default is pretty)
    #[arg(short = 'c', long)]
    compact: bool,
}

impl Args {
    /// Builds the device configuration from command-line arguments.
    fn to_config(&self) -> DeviceConfig {
        DeviceConfig {
            device_filter: self.devices.iter().cloned().collect(),
            gid_type_filter: self.gid_types.iter().cloned().collect(),
            skip_inactive_port: self.skip_inactive,
            roce_v2_skip_link_local_addr: self.skip_link_local,
        }
    }
}

fn main() -> Result<(), ruapc_rdma_sys::Error> {
    let args = Args::parse();
    let config = args.to_config();
    let devices = Devices::open(&config)?;

    let json: Vec<serde_json::Value> = devices
        .iter()
        .map(|d| serde_json::to_value(d.info()).unwrap())
        .collect();

    if args.compact {
        println!("{}", serde_json::to_string(&json).unwrap());
    } else {
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
    }

    Ok(())
}
