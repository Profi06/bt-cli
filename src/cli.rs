use clap::{command, value_parser, Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    let name_arg = Arg::new("filter")
        .index(1)
        .required(true)
        .help("Device filter.");
    let full_match_arg = Arg::new("full_match")
        .short('f').long("full_match")
        .action(ArgAction::SetTrue)
        .help("If set the device name must be an exact match");
    let timeout_arg = Arg::new("timeout")
        .short('t').long("timeout")
        .value_parser(value_parser!(u32))
        .help("Timeout for scanning and pairing attempts in seconds")
        .long_help("Timeout for scanning and pairing attempts in seconds\n\
            Default can be controlled with environment variable BT_TIMEOUT");

    command!()
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
                        .long_help("Only print one device per line. Has no \
                            effect on long listing format, where this is \
                            always the case")
                        .action(ArgAction::SetTrue), 
                    Arg::new("all")
                        .short('a').long("all")
                        .help("Scan for nearby discoverable unpaired devices \
                            and include them in the output")
                        .action(ArgAction::SetTrue),
                    timeout_arg.clone()
                        .requires("all")
                ]),
            Command::new("connect")
                .visible_alias("c")
                .before_help("Connect to a bluetooth device")
                .args([
                    name_arg.clone(), 
                    full_match_arg.clone()
                ]),
            Command::new("disconnect")
                .visible_alias("dc")
                .before_help("Disconnect from a bluetooth device")
                .args([
                    name_arg.clone(), 
                    full_match_arg.clone()
                ]),
            Command::new("info")
                .visible_alias("i")
                .before_help("Get detailed information about a bluetooth device")
                .args([
                    name_arg.clone(), 
                    full_match_arg.clone()
                ]),
            Command::new("pair")
                .visible_alias("p")
                .alias("add")
                .before_help("Pair with a bluetooth device")
                .args([
                    name_arg.clone(), 
                    full_match_arg.clone(), 
                    timeout_arg.clone()
                ]),
            Command::new("unpair")
                .visible_alias("up")
                .aliases(["remove", "rm"])
                .before_help("Remove a bluetooth device")
                .args([
                    name_arg.clone(), 
                    full_match_arg.clone()
                ]),
        ])
}
