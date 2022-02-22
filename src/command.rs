////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command options and dispatch.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// Internal modules.
mod collect;
mod distribute;

// Exports.
pub use collect::*;
pub use distribute::*;


// Internal library imports.
use crate::data::Format;

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
        long = "config",
        parse(from_os_str))]
    pub config: Option<PathBuf>,

    /// The user preferences file to load.
    #[clap(
        long = "prefs",
        parse(from_os_str))]
    pub prefs: Option<PathBuf>,

    /// The stall file to load.
    #[clap(
        short = 's',
        long = "stall",
        parse(from_os_str))]
    pub stall: Option<PathBuf>,

    /// Print copy operations instead of running them.
    #[clap(
        short = 'n',
        long = "dry-run")]
    pub dry_run: bool,
    
    /// Shorten filenames by omitting path prefixes.
    #[clap(
        short = 'm',
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
        #[clap(flatten)]
        common: CommonOptions,
    },

    /// Copies files from the stall directory to their sources.
    Distribute {
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
}
