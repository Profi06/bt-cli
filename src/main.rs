mod bluetooth;
mod term_utils;
mod bluez;
mod cli;

use std::{env, io::{stdout, IsTerminal}};
use bluetooth::*;
use clap::ArgMatches;

fn main() {
    // Initialize empty device list and set values
    let mut devicelist = DeviceList::new();
    let stdout_is_terminal = stdout().lock().is_terminal();
    devicelist.set_quote_names(stdout_is_terminal); 
    devicelist.set_print_in_color(stdout_is_terminal); 
    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
        Some(("list", sub_matches)) => {
            let long_output = sub_matches.get_flag("long_output");
            let linewise = sub_matches.get_flag("linewise");
            let all = sub_matches.get_flag("all");
            let timeout = get_timeout(
                &sub_matches.get_one("timeout").copied(), 30);

            devicelist.fill(if all { timeout } else { None });
            devicelist.print(linewise, long_output);
        }
        Some(("connect", sub_matches)) => {
            let filter = sub_matches.get_one::<String>("filter")
                .expect("filter is required");
            let count = devicelist
                .fill(None)
                .filtered_name(filter, get_behaviour(sub_matches))
                .connect_all();
            println!("Connected {} devices.", count);
        }
        Some(("disconnect", sub_matches)) => {
            let filter = sub_matches.get_one::<String>("filter")
                .expect("filter is required");
            let count = devicelist
                .fill(None)
                .filtered_name(filter, get_behaviour(sub_matches))
                .disconnect_all();
            println!("Disconnected {} devices.", count);
        }
        Some(("info", sub_matches)) => {
            let filter = sub_matches.get_one::<String>("filter")
                .expect("filter is required");
            devicelist.fill(None)
                .filtered_name(filter, get_behaviour(sub_matches)) 
                .print_info_all();
        }
        Some(("pair", sub_matches)) => {
            let filter = sub_matches.get_one::<String>("filter")
                .expect("filter is required");
            let timeout = get_timeout(
                &sub_matches.get_one("timeout").copied(), 5);
            let count = devicelist
                .fill(timeout)
                .filtered_name(filter, get_behaviour(sub_matches))
                .pair_all();
            println!("Paired {} devices.", count);
        }
        Some(("unpair", sub_matches)) => {
            let filter = sub_matches.get_one::<String>("filter")
                .expect("filter is required");
            let count = devicelist
                .fill(None)
                .filtered_name(filter, get_behaviour(sub_matches))
                .unpair_all();
            println!("Unpaired {} devices.", count);
        },
        None => {
            devicelist.fill(None);
            devicelist.print(false, false);
        },
        Some(_) => unreachable!(),
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
