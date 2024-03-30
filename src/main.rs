mod devices;

use clap::{Parser, Subcommand};
use colored::*;
use devices::*;


#[derive(Parser)]
#[command(version, about, long_about = None)]
    #[command(propagate_version = true)]
struct BtCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl BtCli {
    pub fn command(&self) -> Commands{
        self.command.clone().unwrap_or(Commands::Ls { long_output: false, add_unpaired: false, timeout: None })
    }
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// List bluetooth devices
    Ls {
        /// use a long listing format
        #[arg(short, long, default_value_t = false)]
        long_output: bool,
        /// Scan for nearby discoverable unpaired 
        /// devices and include them in the output
        #[arg(short = 'a', long = "add_unpaired")]
        add_unpaired: bool,
        /// use with --add-unpaired or -a to
        /// end device scan after timeout seconds
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Connect to a bluetooth device
    Connect {
        /// Name of the device to connect
        name: String,
        /// end device connection attempt after timeout seconds
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Get detailed information about a bluetooth device
    Info {
        /// Name of the device
        name: String,
    },
    /// Attempts to pair with a device
    Add {
        /// Name of the device to pair
        name: String,
        /// timeout for scanning and pairing attempts in seconds
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    Rm {
        /// Name of the device to unpair
        name: String,
    },
}

fn main() {
    // Shell
    let cli = BtCli::parse();
    match &cli.command {
        Some(Commands::Ls { long_output, add_unpaired, timeout }) => {
            DeviceList::new(if *add_unpaired { timeout.or(Some(2u32)) } else { None }).print(*long_output);
        }
        Some(Commands::Connect { name, timeout }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                if device.connect(timeout.unwrap_or(30u32)) {
                    println!("{} connected.", device.name_colored());
                }
            };
        }
        Some(Commands::Info { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                println!("{:?}", device);
            }
        }
        Some(Commands::Add { name, timeout }) => {
            for device in DeviceList::new(*timeout).devices_with_name(name) {
                if device.pair(timeout.unwrap_or(30u32)) {
                    println!("{} paired.", device.name_colored());
                    device.connect(60u32);
                }
            };
        }
        Some(Commands::Rm { name }) => {
            for device in DeviceList::new(None).devices_with_name(name) {
                device.unpair();
            }
        }
        None => DeviceList::new(None).print(false),
    }
}


