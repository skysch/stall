
# `stall` -- a simple local configuration management utility

The `stall` application is a simple CLI for managing configuration on a single machine.

Any directory which contains a `.stall` file acts as a central repository for collecting files from across a system. The `stall collect` command will copy all of the specified files into the stall directory. The `stall distribute` command will copy all of the specified files from the stall directory into their source directories.

This allows you to gather all of your files in a central location for editting and version control with a single command, and putting those files into their application-specific locations with a single command.

# Installation

Currently you must build `stall` from source. Clone this repository, install Rust, run `Cargo build --release`, and move the compiled binary into your `$PATH` somewhere.


# License

Stall is licenced with the [MIT license](/license-mit.md) or the [Apache version 2.0 license](/license-apache.md), at your option.

