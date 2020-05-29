////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command line interface options.
////////////////////////////////////////////////////////////////////////////////


use serde::Deserialize;
use serde::Serialize;

use structopt::StructOpt;

use std::path::PathBuf;



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
    /// Print copy operations instead of running them.
    #[structopt(short = "n", long = "no-run")]
    pub no_run: bool,
    /// Force copy even if files are unmodified.
    #[structopt(short = "f", long = "force")]
    pub force: bool,
    /// Promote file access warnings to errors.
    #[structopt(short = "w", long = "promote_warnings")]
    pub promote_warnings_to_errors: bool,

}


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
    pub fn common(&self) -> &CommonOptions {
        use CommandOptions::*;
        match self {
            Collect { common, .. } => common,
            Distribute { common, .. } => common,
        }
    }
}
