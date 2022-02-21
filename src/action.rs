////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command implementations.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// Internal modules.
mod collect;
mod distribute;

// Exports.
pub use collect::*;
pub use distribute::*;


// Internal library imports.
use crate::CommonOptions;

// External library imports.
use anyhow::Error;
use tracing::event;
use tracing::Level;
use tracing::span;
use colored::Colorize as _;
use colored::ColoredString;

// Standard library imports.
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Common file copy function.
////////////////////////////////////////////////////////////////////////////////
/// The action taken for a given file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
	/// The file was copied.
	Copy,
	/// The file was skipped.
	Skip,
	/// The command was stopped.
	Stop,
}

impl Action {
	/// Returns a colored string block representation of the Action.
	fn colored_string(&self) -> ColoredString {
		match self {
			Action::Copy => "copy  ".bright_green(),
			Action::Skip => "skip  ".bright_white(),
			Action::Stop => "stop  ".bright_red(),
		}
	}
}

/// The state of the source file relative to the target file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
	/// The file was not available.
	Error,
	/// The file was copied even though it is older than the target.
	Force,
	/// The source file was found, but the target was not.
	Found,
	/// The source file is newer than the target.
	Newer,
	/// The source file is older than the target.
	Older,
}

impl State {
	/// Returns a colored string block representation of the State.
	fn colored_string(&self) -> ColoredString {
		match self {
			State::Error => "error ".bright_red(),
			State::Force => "force ".bright_white(),
			State::Found => "found ".bright_green(),
			State::Newer => "newer ".bright_green(),
			State::Older => "older ".bright_yellow(),
		}
	}
}

/// Prints the status header.
pub fn print_status_header() {
	println!("{}", "    STATE ACTION FILE".bright_white().bold());
}

/// Prints the status line for a file.
pub fn print_status_line(
	state: State,
	action: Action,
	mut path: &Path,
	common: &CommonOptions)
{
	if common.short_names {
		// Fall back to full name if `Path::file_name` method returns `None`.
		// This should never happen, but there's no reason to fail.
		if let Some(name) = path.file_name() {
			path = name.as_ref();
		}
	}

	println!("    {}{} {}", 
		state.colored_string(),
		action.colored_string(),
		path.display());
}


////////////////////////////////////////////////////////////////////////////////
// Common file copy function.
////////////////////////////////////////////////////////////////////////////////
/// Copies a file from `source` to `target` using the given `CopyMethod`
pub fn copy_file(source: &Path, target: &Path, method: CopyMethod)
	-> Result<(), Error>
{
	let _span = span!(Level::DEBUG, "copy_file").entered();

	use CopyMethod::*;
	match method {
		None => event!(Level::DEBUG, "no-run flag was specified: \
            Not copying data from {:?} to {:?}", source, target),

		Subprocess => {
			let status = if cfg!(target_os = "windows") {
			    std::process::Command::new("COPY")
			            .arg(source)
			            .arg(target)
			            .status()
			} else {
			    std::process::Command::new("cp")
			            .arg(source)
			            .arg(target)
			            .status()
			};
			let _ = status.expect("execute copy command");
		},
	}
	Ok(())
}


////////////////////////////////////////////////////////////////////////////////
// CopyMethod
////////////////////////////////////////////////////////////////////////////////
/// The method to use when copying files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMethod {
	/// Do not copy files.
	None,
	/// Copy files using a command in a subprocess.
	Subprocess,
}
