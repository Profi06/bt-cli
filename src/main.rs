mod bluetooth;

use clap::{Parser, Subcommand};
use std::env;
use bluetooth::*;


#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct BtCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// List bluetooth devices
    Ls {
        /// use a long listing format
        #[arg(short, long, default_value_t = false)]
        long_output: bool,
        /// Only print one device per line. Has no effect on 
        /// long listing format, where this is always the case
        #[arg(short = '1', long, default_value_t = false)]
        linewise: bool, 

        /// Scan for nearby discoverable unpaired 
        /// devices and include them in the output
        #[arg(short = 'a', long = "add_unpaired")]
        add_unpaired: bool,
        /// Use with --add-unpaired or -a to
        /// end device scan after timeout seconds.
        /// Default is environment variable BT_TIMEOUT or 30
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Connect to a bluetooth device
    Connect {
        /// Name of the device to connect
        name: String,
    },
    /// Disconnect from a bluetooth device
    Disconnect {
        /// Name of the device to disconnect
        name: String,
    },
    /// Get detailed information about a bluetooth device
    Info {
        /// Name of the device
        name: String,
    },
    /// Pair with a bluetooth device
    Add {
        /// Name of the device to pair
        name: String,
        /// timeout for scanning and pairing attempts in seconds
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Remove a bluetooth device
    Rm {
        /// Name of the device to unpair
        name: String,
    },
}

fn main() {
    // Shell
    let cli = BtCli::parse();
    match &cli.command {
        Some(Commands::Ls { long_output, linewise, add_unpaired, timeout }) => {
            DeviceList::new(if *add_unpaired { get_timeout(timeout, Some(30)) } else { None })
                .print(*linewise, *long_output);
        }
        Some(Commands::Connect { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                device.connect();
            };
        }
        Some(Commands::Disconnect { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                device.disconnect();
            };
        }
        Some(Commands::Info { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                device.update_info();
                println!("{}", device.info_colored());
            }
        }
        Some(Commands::Add { name, timeout }) => {
            println!("Scanning for nearby pairable devices...");
            for device in DeviceList::new(get_timeout(timeout, Some(30))).devices_with_name(name) {
                if device.pair() {
                    device.trust();
                    device.connect();
                }
            };
        }
        Some(Commands::Rm { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                device.unpair();
            }
        }
        None => DeviceList::new(None).print(false, false),
    }
}

fn get_timeout(param: &Option<u32>, default: Option<u32>) -> Option<u32> {
    param.or_else(|| {match env::var("BT_TIMEOUT") {
        Ok(var) => var.trim().parse().ok().or(default),
        Err(_) => default,
    }})
}
