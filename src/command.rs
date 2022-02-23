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
mod add;
mod collect;
mod distribute;
mod init;
mod remove;
mod rename;
mod status;

// Exports.
pub use add::*;
pub use collect::*;
pub use distribute::*;
pub use init::*;
pub use remove::*;
pub use rename::*;
pub use status::*;


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
#[derive(Parser)]
#[clap(name = "stall")]
pub struct CommonOptions {
	/// The application configuration file to load.
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

	/// Print intended operations instead of running them.
	#[clap(long = "dry-run")]
	pub dry_run: bool,
	
	/// Shorten filenames by omitting path prefixes.
	#[clap(
		short = 'o',
		long = "short-names")]
	pub short_names: bool,
	
	/// Promote any warnings into errors and abort.
	#[clap(long = "error")]
	pub promote_warnings_to_errors: bool,


	/// When to color output.
	#[clap(
		long = "color",
		default_value = "auto",
		arg_enum)]
	pub color: ColorOption,
	
	/// Provide more detailed messages.
	#[clap(
		short = 'v',
		long = "verbose",
		group = "verbosity")]
	pub verbose: bool,

	/// Silence all non-error program output.
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
#[derive(Parser)]
#[clap(name = "stall")]
#[clap(author, version, about, long_about = None)]
pub enum CommandOptions {
	/// Intitialize a stall directory by generating a stall file.
	Init {
		#[clap(flatten)]
		common: CommonOptions,

		// TODO: Set rename policy
	},

	/// Print the status of stalled files.
	Status {
		#[clap(flatten)]
		common: CommonOptions,

		// TODO: Sort entries.
	},

	/// Add a file to a stall.
	Add {
		#[clap(flatten)]
		common: CommonOptions,

		#[clap(parse(from_os_str))]
		file: PathBuf,

		// TODO: Overwrite if exists?
		// TODO: Immediate collect?
		// TODO: Add rename?
		// TODO: Rename if exists?
		// TODO: multiple?
	},

	/// Remove a file from a stall.
	Remove {
		#[clap(flatten)]
		common: CommonOptions,

		#[clap(parse(from_os_str))]
		file: PathBuf,

		// TODO: Delete local copy?
		// TODO: match local name?
		// TODO: multiple?
	},

	/// Rename a file in a stall.
	Move {
		#[clap(flatten)]
		common: CommonOptions,

		#[clap(parse(from_os_str))]
		from: PathBuf,

		#[clap(parse(from_os_str))]
		to: PathBuf,

		// TODO: Overwrite if exists?
	},

	/// Copy files into the stall directory from their remote locations.
	Collect {
		#[clap(flatten)]
		common: CommonOptions,

		/// Force copy even if files are unmodified.
		#[clap(
			short = 'f',
			long = "force")]
		force: bool,
	},

	/// Copi files from the stall directory to their remote locations.
	Distribute {
		#[clap(flatten)]
		common: CommonOptions,

		/// Force copy even if files are unmodified.
		#[clap(
			short = 'f',
			long = "force")]
		force: bool,
	},
}

impl CommandOptions {
	/// Returns the `CommonOptions`.
	pub fn common(&self) -> &CommonOptions {
		use CommandOptions::*;
		match self {
			Init { common, .. }       |
			Status { common, .. }     |
			Add { common, .. }        |
			Remove { common, .. }     |
			Move { common, .. }       |
			Collect { common, .. }    |
			Distribute { common, .. } => common,
		}
	}
}



////////////////////////////////////////////////////////////////////////////////
// ColorOption
////////////////////////////////////////////////////////////////////////////////
/// Options for handling missing files.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(clap::ArgEnum)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}

impl ColorOption {
	/// Returns true if colored output should be used.
	pub fn enabled(&self) -> bool {
		match self {
			ColorOption::Auto => {
				// Defer to `colored` for enviroment vars and TTY detection.
				colored::control::SHOULD_COLORIZE.should_colorize()
			},
			ColorOption::Always => true,
			ColorOption::Never => false,
		}
	}
}

impl std::str::FromStr for ColorOption {
    type Err = ColorOptionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("auto") {
            Ok(ColorOption::Auto)
        } else if s.eq_ignore_ascii_case("always") {
            Ok(ColorOption::Always)
        } else if s.eq_ignore_ascii_case("never") {
            Ok(ColorOption::Never)
        } else {
            Err(ColorOptionParseError)
        }
    }
}

/// An error indicating a failure to parse a [`ColorOption`].
///
/// [`ColorOption`]: ColorOption 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorOptionParseError;

impl std::error::Error for ColorOptionParseError {}

impl std::fmt::Display for ColorOptionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failure to parse ColorOption")
    }
}
