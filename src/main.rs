mod bluetooth;
mod term_utils;
mod bluez;
mod cli;

use std::{env, io::{stdout, IsTerminal}};
use bluetooth::*;

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
            let timeout = get_timeout(&sub_matches.get_one("timeout").copied(), Some(30));

            devicelist.fill(if all { timeout } else { None });
            devicelist.print(linewise, long_output);
        }
        Some(("connect", sub_matches)) => {
            let name = sub_matches.get_one::<String>("filter").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(full_match))
                .connect_all();
            println!("Connected {} devices.", count);
        }
        Some(("disconnect", sub_matches)) => {
            let name = sub_matches.get_one::<String>("filter").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(full_match))
                .disconnect_all();
            println!("Disconnected {} devices.", count);
        }
        Some(("info", sub_matches)) => {
            let name = sub_matches.get_one::<String>("filter").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            devicelist.fill(None)
                .filtered_name(name, get_behaviour(full_match)) 
                .print_info_all();
        }
        Some(("pair", sub_matches)) => {
            let name = sub_matches.get_one::<String>("filter").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let timeout = get_timeout(&sub_matches.get_one("timeout").copied(), Some(5));
            println!("Scanning for nearby pairable devices...");
            let count = devicelist
                .fill(timeout)
                .filtered_name(name, get_behaviour(full_match))
                .pair_all();
            println!("Paired {} devices.", count);
        }
        Some(("unpair", sub_matches)) => {
            let name = sub_matches.get_one::<String>("filter").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(full_match))
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
