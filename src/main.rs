use std::process::{Command, Output};
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
struct BtCli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Devices {
        /// Adds address to output
        #[arg(short, long, default_value_t = false)]
        long_output: bool,
    },
    Connect {
        name: String,
    }
}

fn main() {
    // Shell
    let cli = BtCli::parse();
    match &cli.command {
        Some(Commands::Devices { long_output }) => {
            devicelist(*long_output);
        }
        Some(Commands::Connect { name }) => {
            if let Some(address) = name_to_addess(name) {
                let _ = Command::new("bluetoothctl").arg("connect").arg(address).status();
        }
        }
        None => devicelist(false),
    }
}

fn devicelist(long_output: bool) {
    let bluetoothctl_output : Output = Command::new("bluetoothctl").arg("devices").output().expect("failed to execute process");
    let output_str = String::from_utf8(bluetoothctl_output.stdout).expect("Invalid UTF-8 in output");
    for line in output_str.lines() {
        let mut split = line.splitn(3, ' ');
        // First is always "Device" and unnecessary
        split.next();
        let address = split.next().unwrap_or_else(|| "Address Not Known");
        let device_name = split.next().unwrap_or_else(|| "");
        let device_name_text = match check_if_address_connected(address) {
            true => device_name.bold().green(),
            false => device_name.into()
        };
        if long_output {
            println!("{} {device_name_text}", address.bright_black());
        } else {
            print!("{device_name_text}  ");
        }
    }
    // Newline
    println!("");
}

fn name_to_addess(name: &str) -> Option<String> {
    let bluetoothctl_output : Output = Command::new("bluetoothctl").arg("devices").output().expect("failed to get device list");
    let output_str = String::from_utf8(bluetoothctl_output.stdout).expect("Invalid UTF-8 in output");
    for line in output_str.lines() {
        let mut split = line.splitn(3, ' ');
        // First is always "Device" and unnecessary
        split.next();
        let address = split.next().unwrap_or_else(|| "");
        if let Some(device_name) = split.next() {
            if device_name == name {
                return Some(address.to_string());
            }
        }
    }
    return None;
}

fn check_if_address_connected(address: &str) -> bool {
    let bluetoothctl_output : Output = Command::new("bluetoothctl").arg("info").arg(address).output().expect("failed to get device info");
    match String::from_utf8(bluetoothctl_output.stdout) {
        Ok(str) => {
            return str.contains("Connected: yes");
        },
        Err(..) => {
            return false;
        }
    };
}

