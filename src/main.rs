mod bluetooth;
mod term_utils;
mod bluez;

use clap::{command, value_parser, Arg, ArgAction, Command};
use std::env;
use bluetooth::*;

fn main() {
    let name_arg = Arg::new("name")
        .index(1)
        .required(true)
        .help("Device name");
    let full_match_arg = Arg::new("full_match")
        .short('f').long("full_match")
        .action(ArgAction::SetTrue)
        .help("If set the device name must be an exact match");
    let timeout_arg = Arg::new("timeout")
        .short('t').long("timeout")
        .value_parser(value_parser!(u32))
        .help("Timeout for scanning and pairing attempts in seconds")
        .long_help("Timeout for scanning and pairing attempts in seconds\nDefault can be controlled with environment variable BT_TIMEOUT");

    let matches = command!()
        .propagate_version(true)
        .subcommands([
            Command::new("list")
                .visible_alias("ls")
                .before_help("List bluetooth devices")
                .args([
                    Arg::new("long_output")
                        .short('l').long("long_output")
                        .help("Use a long listing format")
                        .action(ArgAction::SetTrue),
                    Arg::new("linewise")
                        .short('1').long("linewise")
                        .help("Only print one device per line")
                        .long_help("Only print one device per line. Has no effect on\nlong listing format, where this is always the case")
                        .action(ArgAction::SetTrue), 
                    Arg::new("all")
                        .short('a').long("all")
                        .help("Scan for nearby discoverable unpaired devices and include them in the output")
                        .action(ArgAction::SetTrue),
                    timeout_arg.clone()
                        .requires("all")
                ]),
            Command::new("connect")
                .visible_alias("c")
                .before_help("Connect to a bluetooth device")
                .args([name_arg.clone(), full_match_arg.clone()]),
            Command::new("disconnect")
                .visible_alias("dc")
                .before_help("Disconnect from a bluetooth device")
                .args([name_arg.clone(), full_match_arg.clone()]),
            Command::new("info")
                .visible_alias("i")
                .before_help("Get detailed information about a bluetooth device")
                .args([name_arg.clone(), full_match_arg.clone()]),
            Command::new("pair")
                .visible_alias("p")
                .alias("add")
                .before_help("Pair with a bluetooth device")
                .args([name_arg.clone(), full_match_arg.clone(), timeout_arg.clone()]),
            Command::new("unpair")
                .visible_alias("up")
                .aliases(["remove", "rm"])
                .before_help("Remove a bluetooth device")
                .args([name_arg.clone(), full_match_arg.clone()]),
        ])
        .get_matches();

    // Shell
    let mut devicelist = DeviceList::new();
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
            let name = sub_matches.get_one::<String>("name").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(full_match))
                .connect_all();
            println!("Connected {} devices.", count);
        }
        Some(("disconnect", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            let count = devicelist
                .fill(None)
                .filtered_name(name, get_behaviour(full_match))
                .disconnect_all();
            println!("Disconnected {} devices.", count);
        }
        Some(("info", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").expect("name is required");
            let full_match = sub_matches.get_flag("full_match");
            devicelist.fill(None)
                .filtered_name(name, get_behaviour(full_match)) 
                .print_info_colored_all();
        }
        Some(("pair", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").expect("name is required");
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
            let name = sub_matches.get_one::<String>("name").expect("name is required");
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
