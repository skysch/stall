////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface options.
////////////////////////////////////////////////////////////////////////////////


// External library imports.
use serde::Deserialize;
use serde::Serialize;

use structopt::StructOpt;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
pub struct CommonOptions {
    /// The stall file to use.
    #[structopt(
        short = "u",
        long = "use-config",
        parse(from_os_str))]
    pub use_config: Option<PathBuf>,

    /// The format of the stall file.
    #[structopt(
        short = "c",
        long = "config-format",
        possible_values(&["ron","list"]))]
    pub config_format: Option<String>,

    /// Print copy operations instead of running them.
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
    
    /// Shorten filenames by omitting path prefixes.
    #[structopt(short = "s", long = "short-names")]
    pub short_names: bool,

    /// Force copy even if files are unmodified.
    #[structopt(short = "f", long = "force")]
    pub force: bool,
    
    /// Promote file access warnings into errors.
    #[structopt(short = "e", long = "error")]
    pub promote_warnings_to_errors: bool,
    
    /// Provides more detailed messages.
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,

    /// Silences all program output. This override --verbose if both are provided.
    #[structopt(short = "q", long = "quiet", alias = "silent")]
    pub quiet: bool,

    /// Print trace messages. This override --quiet if both are provided.
    #[structopt(long = "xtrace", hidden(true))]
    pub trace: bool,
}

////////////////////////////////////////////////////////////////////////////////
// CommandOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line subcommand options.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(StructOpt)]
#[structopt(name = "stall")]
pub enum CommandOptions {
    /// Copies files into the stall directory.
    Collect {
        /// The stall directory to copy into. Default is the current directory.
        #[structopt(long = "into", parse(from_os_str))]
        into: Option<PathBuf>,

        #[structopt(flatten)]
        common: CommonOptions,
    },

    /// Copies files from the stall directory to their sources.
    Distribute {
        /// The stall directory to copy from. Default is the current directory.
        #[structopt(long = "from", parse(from_os_str))]
        from: Option<PathBuf>,

        #[structopt(flatten)]
        common: CommonOptions,
    },
}

impl CommandOptions {
    /// Returns the `CommonOptions`.
    pub fn common(&self) -> &CommonOptions {
        use CommandOptions::*;
        match self {
            Collect { common, .. } => common,
            Distribute { common, .. } => common,
        }
    }

    /// Returns the stall directory.
    pub fn stall_dir(&self) -> Result<PathBuf, std::io::Error> {
        use CommandOptions::*;
        match &self {
            Collect { into, .. } => match into {
                Some(path) => Ok(path.clone()),
                None       => std::env::current_dir(),
            },
            Distribute { from, .. } => match from {
                Some(path) => Ok(path.clone()),
                None       => std::env::current_dir(),
            }
        }
    }
}
