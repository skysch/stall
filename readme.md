
# `stall` -- A simple file gathering repository

Stall is a command line utility for managing configuration data within a directory.

Running `stall init` in a directory will create a `.stall` file in that directory which tracks any files added, removed, or moved by `stall add`, `stall rm`, or `stall mv`. 

Tracked files (also 'stalled files') can be easily copied to and from their original locations via the `stall distribute` and `stall collect` commands, respectively. These commands will ensure that copies will only occur if an older version of the file is being overwritten. Use `stall status` to display the status of all files tracked.

Stall can make it easy to group, edit, backup, and apply version control to specific files dispersed among many different directories.


# Installation

There are currently two install options: 

1. [Install cargo](https://crates.io/) and run `cargo install stall`.

2. Build `stall` from source. Clone this repository, install Rust, run `Cargo build --release`, and move the compiled binary into your `$PATH` somewhere.

# Usage

```
USAGE:
    stall <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    add           Add files to a stall
    collect       Copy files into the stall directory from their remote locations
    distribute    Copi files from the stall directory to their remote locations
    help          Print this message or the help of the given subcommand(s)
    init          Intitialize a stall directory by generating a stall file
    mv            Rename a file in a stall. Future collect/distribute actions will use the new
                  name
    rm            Remove files from a stall
    status        Print the status of stalled files
```

# License

Stall is licenced with the [MIT license](/license-mit.md) or the [Apache version 2.0 license](/license-apache.md), at your option.

