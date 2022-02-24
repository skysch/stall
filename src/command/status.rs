////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Print the stall status.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::CommonOptions;
use crate::Stall;
use crate::entry::Entry;

// External library imports.
use anyhow::Error;
use tracing::span;
use tracing::Level;
use colored::Colorize as _;

// Standard library imports.
use std::path::Path;
use std::io::Write as _;


////////////////////////////////////////////////////////////////////////////////
// collect
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-status' command.
///

pub fn status<P>(
	stall_dir: P,
	stall: &Stall,
	common: CommonOptions) 
	-> Result<(), Error>
	where 
		P: AsRef<Path>,
{
	let _span = span!(Level::INFO, "status").entered();
	
	if stall.is_empty() || common.quiet {
		if !common.quiet {
			println!("No files in stall. Use `add` command to place files \
				in the stall.");
		}
		// Nothing to do if asking for status with --quiet.
		return Ok(());
	}


	let mut out = std::io::stdout();

	// Setup and print stall directory.
	let stall_dir = stall_dir.as_ref();
	if common.color.enabled() {
		writeln!(&mut out, "{} {}",
			"Stall directory:".bright_white(),
			stall_dir.display())?;
	} else {
		writeln!(&mut out, "{} {}",
			"Stall directory:",
			stall_dir.display())?;
	}

	// Write status table.
	Entry::write_status_header(&mut out, &common)?;
	for entry in stall.entries() {

		let (status_l, status_r) = entry.status(stall_dir);
		entry.write_status(&mut out, status_l, status_r, &common)?;
	}

	Ok(())
}
