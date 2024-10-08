# BT cli [![Licence](https://img.shields.io/badge/License-GPLv3-red.svg)](LICENSE)

Make bluetooth device management in the terminal easy and intuitive.

#### Usage
- List devices with `bt list` or `bt ls`
- Pair with `bt pair <name>` or `bt p <name>`
- Unpair with `bt unpair <name>` or `bt up <name>`
- Connect with `bt connect <name>` or `bt c <name>`
- Disconnect with `bt disconnect <name>` or `bt dc <name>`
- Show device details with `bt info <name>` or `bt i <name>`

Any command with a `<name>` parameter may use the following arguments:
- `-p --partial-match` matches devices, whose name contains `<name>`. Default behaviour.
- `-P --no-partial-match` matches devices, whose full name matches `<name>`.
- `-r --regex` interprets `<name>` as a regex pattern that must be matched by the device name. (`-p` and `-P` still apply)
- `-R --no-regex` interprets `<name>` as a literal string that must be matched by the device name. Default behaviour.

The following arguments are exclusive to the `list` command:
- `-l --long` for a long listing format
- `-1 --linewise` outputs each device on its own line
- `-a --all` scans for unpaired devices before outputting.

The commands `pair` and `list -a` can specify a timeout (in seconds) for device scanning with `-t <timeout>` or `--timeout <timeout>`.

#### Building
This project can be built with cargo. If you do not have the Rust toolchain installed you can install it from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
```
git clone git@github.com:Profi06/bt-cli.git
cd bt-cli
cargo build --release
```

#### Planned Features
- `bt send <name> <file>` to send files
- `bt recv <name>` to recieve files
- Arguments to turn ANSI color codes off (currently only depends on stdout being a terminal)
- Argument for applying `<name>` filtering to address instead

