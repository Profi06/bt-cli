// vim: cc=81
mod bluetooth;
mod utils;
mod cli;

use std::{env, io::{stdout, IsTerminal}, sync::Arc};
use bluetooth::{*, devices::FilterBehaviour};
use bluez::DBusBluetoothManager;
use clap::ArgMatches;

fn main() {
    // Initialize empty device list and set values
    if let Ok(mut bluetooth_manager) = DBusBluetoothManager::new() {
        bluetooth_manager.update();
        let bluetooth_manager = Arc::new(bluetooth_manager);
        let mut devicelist = DeviceList::new(Arc::clone(&bluetooth_manager));

        let stdout_is_terminal = stdout().lock().is_terminal();
        devicelist.set_quote_names(stdout_is_terminal); 
        devicelist.set_print_in_color(stdout_is_terminal); 
        let mut command = cli::build_cli();
        let matches = command.get_matches_mut();

        match matches.subcommand() {
            Some(("list", sub_matches)) => {
                let long_output = sub_matches.get_flag("long_output");
                let linewise = sub_matches.get_flag("linewise");
                let all = sub_matches.get_flag("all");
                // TODO: Use if all == true
                let timeout = get_timeout(
                    &sub_matches.get_one("timeout").copied(), 30);

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

fn get_timeout(param: &Option<u32>, default: u32) -> Option<u32> {
    param.or_else(|| {match env::var("BT_TIMEOUT") {
        Ok(var) => var.trim().parse().ok().or(Some(default)),
        Err(_) => Some(default),
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
