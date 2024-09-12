// vim: cc=81
mod bluetooth;
mod utils;
mod cli;

use std::{env, io::{stdout, IsTerminal}, sync::{Arc, Mutex}, time::Duration};
use bluetooth::{*, devices::FilterBehaviour};
use bluez::DBusBluetoothManager;
use clap::ArgMatches;

fn main() {
    let mut command = cli::build_cli();
    let matches = command.get_matches_mut();
    let stdout_is_terminal = stdout().lock().is_terminal();
    if let Ok(mut bluetooth_manager) = DBusBluetoothManager::new() {
        bluetooth_manager.set_scan_display_hint(stdout_is_terminal);
        bluetooth_manager.update();
        let bluetooth_manager = Arc::new(Mutex::new(bluetooth_manager));

        // Initialize empty device list and set values
        let mut devicelist = DeviceList::new(Arc::clone(&bluetooth_manager));
        devicelist.set_quote_names(stdout_is_terminal); 
        devicelist.set_print_in_color(stdout_is_terminal); 

        match matches.subcommand() {
            Some(("list", sub_matches)) => {
                let long_output = sub_matches.get_flag("long_output");
                let linewise = sub_matches.get_flag("linewise");
                if sub_matches.get_flag("all") {
                    let timeout = get_timeout(
                        &sub_matches.get_one("timeout").copied(), 30);
                    bluetooth_manager
                        .lock().expect("Mutex should not be poisoned.")
                        .scan_mut(&Duration::from_secs(timeout))
                        .update();
                }
                devicelist.fill();
                devicelist.print(linewise, long_output);
            }
            Some(("connect", sub_matches)) => {
                let filter = sub_matches.get_one::<String>("filter")
                    .expect("filter is required");
                let count = devicelist
                    .fill()
                    .filtered_name(filter, get_behaviour(sub_matches))
                    .connect_all();
                println!("Connected {} devices.", count);
            }
            Some(("disconnect", sub_matches)) => {
                let filter = sub_matches.get_one::<String>("filter")
                    .expect("filter is required");
                let count = devicelist
                    .fill()
                    .filtered_name(filter, get_behaviour(sub_matches))
                    .disconnect_all();
                println!("Disconnected {} devices.", count);
            }
            Some(("info", sub_matches)) => {
                let filter = sub_matches.get_one::<String>("filter")
                    .expect("filter is required");
                devicelist
                    .fill()
                    .filtered_name(filter, get_behaviour(sub_matches)) 
                    .print_info_all();
            }
            Some(("pair", sub_matches)) => {
                let filter = sub_matches.get_one::<String>("filter")
                    .expect("filter is required");
                let timeout = get_timeout(
                    &sub_matches.get_one("timeout").copied(), 5);
                bluetooth_manager
                    .lock().expect("Mutex should not be poisoned.")
                    .scan_mut(&Duration::from_secs(timeout))
                    .update();
                let count = devicelist
                    .fill()
                    .filtered_name(filter, get_behaviour(sub_matches))
                    .pair_all();
                println!("Paired {} devices.", count);
            }
            Some(("unpair", sub_matches)) => {
                let filter = sub_matches.get_one::<String>("filter")
                    .expect("filter is required");
                let count = devicelist
                    .fill()
                    .filtered_name(filter, get_behaviour(sub_matches))
                    .unpair_all();
                println!("Unpaired {} devices.", count);
            },
            // Some(_) should be unreachable but just in case
            None | Some(_) => {
                let _ = command.print_help();
            },
        }
    }
}

fn get_timeout(param: &Option<u64>, default: u64) -> u64 {
    param.unwrap_or_else(|| { match env::var("BT_TIMEOUT") {
        Ok(var) => var.trim().parse().ok().unwrap_or(default),
        Err(_) => default,
    }})
}

fn get_behaviour(matches: &ArgMatches) -> FilterBehaviour {
    let partial = *matches.get_one::<bool>("partial").unwrap_or(&true) 
        && !matches.get_one::<bool>("no-partial").unwrap_or(&false);
    let regex = *matches.get_one::<bool>("regex").unwrap_or(&false) 
        && !matches.get_one::<bool>("no-regex").unwrap_or(&true);
    if partial {
        if regex {
            FilterBehaviour::ContainsRegex
        } else {
            FilterBehaviour::Contains
        }
    } else {
        if regex {
            FilterBehaviour::FullRegex
        } else {
            FilterBehaviour::Full
        }
    }
}
