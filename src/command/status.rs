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
use crate::entry::Stall;
use crate::entry::Entry;

// External library imports.
use anyhow::Context;
use anyhow::Error;
use tracing::event;
use tracing::span;
use tracing::Level;
use colored::Colorize as _;

// Standard library imports.
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// collect
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-status' command.
///

pub fn status<P>(
	stall_dir: P,
	data: &Stall,
	common: CommonOptions) 
	-> Result<(), Error>
	where 
		P: AsRef<Path>,
{
	let _span = span!(Level::INFO, "status").entered();

	if !common.quiet {
		if data.is_empty() {
			println!("No files in stall. Use `add` command to place files \
				in the stall.");
			return Ok(());
		}
	} else {
		// Nothing to do if asking for status with --quiet.
		return Ok(());
	}


	let stall_dir = stall_dir.as_ref();
	let mut out = std::io::stdout();

	Entry::write_status_header(&mut out, &common)?;
	for entry in data.entries() {
		entry.write_status(&mut out, stall_dir, &common)?;
	}

	Ok(())
}
