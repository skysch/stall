////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface options.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::application::ConfigFormat;

// External library imports.
use clap::Parser;
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// CommonOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line options shared between subcommands.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Parser)]
pub struct CommonOptions {
    /// The stall file to use.
    #[clap(
        short = 'u',
        long = "use-config",
        parse(from_os_str))]
    pub use_config: Option<PathBuf>,

    /// The format of the stall file.
    #[clap(
        short = 'c',
        long = "config-format",
        default_value = "list",
        arg_enum)]
    pub config_format: ConfigFormat,

    /// Print copy operations instead of running them.
    #[clap(
        short = 'n',
        long = "dry-run")]
    pub dry_run: bool,
    
    /// Shorten filenames by omitting path prefixes.
    #[clap(
        short = 's',
        long = "short-names")]
    pub short_names: bool,

    /// Force copy even if files are unmodified.
    #[clap(
        short = 'f',
        long = "force")]
    pub force: bool,
    
    /// Promote file access warnings into errors.
    #[clap(
        short = 'e',
        long = "error")]
    pub promote_warnings_to_errors: bool,
    
    /// Provides more detailed messages.
    #[clap(
        short = 'v',
        long = "verbose",
        group = "verbosity")]
    pub verbose: bool,

    /// Silences all program output.
    #[clap(
        short = 'q',
        long = "quiet",
        alias = "silent",
        group = "verbosity")]
    pub quiet: bool,

    /// Print trace messages.
    #[clap(
        long = "ztrace",
        hide(true))]
    pub trace: bool,
}


////////////////////////////////////////////////////////////////////////////////
// CommandOptions
////////////////////////////////////////////////////////////////////////////////
/// Command line subcommand options.
#[allow(missing_docs)]
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Parser)]
#[clap(name = "stall")]
pub enum CommandOptions {
    /// Copies files into the stall directory.
    Collect {
        /// The stall directory to copy into. Default is the current directory.
        #[clap(
            long = "into",
            parse(from_os_str))]
        into: Option<PathBuf>,

        #[clap(flatten)]
        common: CommonOptions,
    },

    /// Copies files from the stall directory to their sources.
    Distribute {
        /// The stall directory to copy from. Default is the current directory.
        #[clap(
            long = "from",
            parse(from_os_str))]
        from: Option<PathBuf>,

        #[clap(flatten)]
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
