// vim: cc=81
use clap::{command, value_parser, Arg, ArgAction, ArgGroup, Command};

pub fn build_cli() -> Command {
    let name_arg = Arg::new("filter")
        .index(1)
        .required(true)
        .help("Device filter.");
    let timeout_arg = Arg::new("timeout")
        .short('t').long("timeout")
        .value_parser(value_parser!(u32))
        .help("Timeout for scanning and pairing attempts in seconds")
        .long_help("Timeout for scanning and pairing attempts in seconds\n\
            Default can be controlled with environment variable BT_TIMEOUT");

    let color_arg = Arg::new("color")
        .short('c').long("color")
        .action(ArgAction::SetTrue)
        .conflicts_with("no-color")
        .help("Uses ANSI escape sequences to print with text formatting and \
            color if used");
    let no_color_arg = Arg::new("no-color")
        .short('C').long("no-color")
        .action(ArgAction::SetTrue)
        .conflicts_with("color")
        .help("Disallow usage of ANSI escape sequences");
    let color_arg_group = ArgGroup::new("color group")
        .args(["color", "no-color"]);

    let partial_arg = Arg::new("partial")
        .short('p').long("partial")
        .action(ArgAction::SetTrue)
        .help("If set the filter must only match part of the device name.");
    let no_partial_arg = Arg::new("no-partial")
        .short('P').long("no-partial")
        .action(ArgAction::SetTrue)
        .help("If set the filter must match the full device name.");
    let partial_arg_group = ArgGroup::new("partial group")
        .args(["partial", "no-partial"]);

    let regex_arg = Arg::new("regex")
        .short('r').long("regex")
        .action(ArgAction::SetTrue)
        .help("If set the filter is interpreted as a regex pattern.");
    let no_regex_arg = Arg::new("no-regex")
        .short('R').long("no-regex")
        .action(ArgAction::SetTrue)
        .help("If set the filter is applied literally");
    let regex_arg_group = ArgGroup::new("regex group")
        .args(["regex", "no-regex"]);

    let address_arg = Arg::new("address")
        .short('a').long("address")
        .action(ArgAction::SetTrue)
        .help("Filter is matched against the address instead of name.");
    let fields_arg = Arg::new("filter-fields")
        .short('f').long("filter-fields")
        .action(ArgAction::SetTrue);
    let filter_arg_group = ArgGroup::new("filter group")
        .args(["address", "filter-fields"]);


    command!()
        .propagate_version(true)
        .args([
            color_arg,
            no_color_arg,
        ])
        .group(color_arg_group)
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
                    partial_arg.clone(),
                    no_partial_arg.clone(),
                    regex_arg.clone(),
                    no_regex_arg.clone(),
                    address_arg.clone(),
                    fields_arg.clone(),
                    timeout_arg.clone(),
                ]).groups([
                    partial_arg_group.clone(),
                    regex_arg_group.clone(),
                    filter_arg_group.clone(),
                ]),
            Command::new("disconnect")
                .visible_alias("dc")
                .before_help("Disconnect from a bluetooth device")
                .args([
                    name_arg.clone(), 
                    partial_arg.clone(),
                    no_partial_arg.clone(),
                    regex_arg.clone(),
                    no_regex_arg.clone(),
                    address_arg.clone(),
                    fields_arg.clone(),
                ]).groups([
                    partial_arg_group.clone(),
                    regex_arg_group.clone(),
                    filter_arg_group.clone(),
                ]),
            Command::new("info")
                .visible_alias("i")
                .before_help("Get detailed information about a bluetooth device")
                .args([
                    name_arg.clone(), 
                    partial_arg.clone(),
                    no_partial_arg.clone(),
                    regex_arg.clone(),
                    no_regex_arg.clone(),
                    address_arg.clone(),
                    fields_arg.clone(),
                ]).groups([
                    partial_arg_group.clone(),
                    regex_arg_group.clone(),
                    filter_arg_group.clone(),
                ]),
            Command::new("pair")
                .visible_alias("p")
                .before_help("Pair with a bluetooth device")
                .args([
                    name_arg.clone(), 
                    partial_arg.clone(),
                    no_partial_arg.clone(),
                    regex_arg.clone(),
                    no_regex_arg.clone(),
                    address_arg.clone(),
                    fields_arg.clone(),
                    timeout_arg.clone(),
                ]).groups([
                    partial_arg_group.clone(),
                    regex_arg_group.clone(),
                    filter_arg_group.clone(),
                ]),
            Command::new("unpair")
                .visible_alias("up")
                .before_help("Unpair from a bluetooth device")
                .args([
                    name_arg.clone(), 
                    partial_arg.clone(),
                    no_partial_arg.clone(),
                    regex_arg.clone(),
                    no_regex_arg.clone(),
                    address_arg.clone(),
                    fields_arg.clone(),
                ]).groups([
                    partial_arg_group.clone(),
                    regex_arg_group.clone(),
                    filter_arg_group.clone(),
                ]),
    ])
}
