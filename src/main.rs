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
        /// Default is environment variable BT_TIMEOUT or 5
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Connect to a bluetooth device
    Connect {
        /// Name of the device to connect
        name: String,

        /// If false the device name must only contain the given name, otherwise it must
        /// match exactly.
        #[arg(short, long, default_value_t = false)]
        full_match: bool,
    },
    /// Disconnect from a bluetooth device
    Disconnect {
        /// Name of the device to disconnect
        name: String,
        
        /// If false the device name must only contain the given name, otherwise it must
        /// match exactly.
        #[arg(short, long, default_value_t = false)]
        full_match: bool,
    },
    /// Get detailed information about a bluetooth device
    Info {
        /// Name of the device
        name: String,
        
        /// If false the device name must only contain the given name, otherwise it must
        /// match exactly.
        #[arg(short, long, default_value_t = false)]
        full_match: bool,
    },
    /// Pair with a bluetooth device
    Pair {
        /// Name of the device to pair
        name: String,
        
        /// If false the device name must only contain the given name, otherwise it must
        /// match exactly.
        #[arg(short, long, default_value_t = false)]
        full_match: bool,

        /// timeout for scanning and pairing attempts in seconds
        #[arg(short, long)]
        timeout: Option<u32>, 
    },
    /// Remove a bluetooth device
    Unpair {
        /// Name of the device to unpair
        name: String,

        /// If false the device name must only contain the given name, otherwise it must
        /// match exactly.
        #[arg(short, long, default_value_t = false)]
        full_match: bool,
    },
}

fn main() {
    // Shell
    let cli = BtCli::parse();
    let mut devicelist = DeviceList::new();
    match &cli.command {
        Some(Commands::Ls { long_output, linewise, add_unpaired, timeout }) => {
            devicelist.fill(if *add_unpaired { get_timeout(timeout, Some(30)) } else { None });
            devicelist.print(*linewise, *long_output);
        }
        Some(Commands::Connect { name, full_match }) => {
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(*full_match))
                .connect_all();
            println!("Connected {} devices.", count);
        }
        Some(Commands::Disconnect { name, full_match }) => {
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(*full_match))
                .disconnect_all();
            println!("Disconnected {} devices.", count);
        }
        Some(Commands::Info { name, full_match }) => {
            devicelist.fill(None);
            for device in devicelist.filtered_name(name, get_behaviour(*full_match)) {
                let mut device = device.lock().expect("Mutex should not be poisoned.");
                device.update_info();
                println!("{}", device.info_colored());
            }
        }
        Some(Commands::Pair { name, full_match, timeout }) => {
            println!("Scanning for nearby pairable devices...");
            let count = devicelist
                .fill(get_timeout(timeout, Some(5)))
                .filtered_name(name, get_behaviour(*full_match))
                .pair_all();
            println!("Paired {} devices.", count);
        }
        Some(Commands::Unpair { name, full_match }) => {
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(*full_match))
                .unpair_all();
            println!("Unpaired {} devices.", count);
        }
        None => {
            devicelist.fill(None);
            devicelist.print(false, false);
        },
    }
}

fn get_timeout(param: &Option<u32>, default: Option<u32>) -> Option<u32> {
    param.or_else(|| {match env::var("BT_TIMEOUT") {
        Ok(var) => var.trim().parse().ok().or(default),
        Err(_) => default,
    }})
}

fn get_behaviour(full_match: bool) -> FilterBehaviour {
    if full_match {
        FilterBehaviour::Full
    } else {
        FilterBehaviour::Contains
    }
}
