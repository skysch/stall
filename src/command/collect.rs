////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Collect files into a stall.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::CommonOptions;
use crate::Stall;
use crate::entry::Entry;

// External library imports.
use anyhow::anyhow;
use anyhow::Error;
use colored::Colorize as _;
use either::Either;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::path::Path;
use std::io::Write as _;

////////////////////////////////////////////////////////////////////////////////
// collect
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-collect' command.
///
/// This will iterate over each entry in the [`Stall`], checking if it is more
/// recent than its counterpart in the remote directory by comparing their
/// modification times. If the remote file is newer, it will be copied into the
/// stall directory, overwriting the existing file.
///
/// ### Parameters
///
/// + `stall_dir`: The stall directory to collect into.
/// + `stall`: The loaded `Stall` data.
/// + `files`: An iterator over the [`Path`]s of the files to collect.
/// + `force`: Force overwrites even if the files are current.
/// + `dry_run`: Do not copy any files.
/// + `common`: The [`CommonOptions`] to use for the command.
///
/// ### Errors
/// 
/// Returns an [`Error`] if both files exist but their metadata can't be read,
/// if the copy operation fails, or if any IO errors occur.
/// 
/// [`Path`]: https://doc.rust-lang.org/stable/std/path/struct.Path.html
/// [`Stall`]: ../struct.Stall.html
/// [`CommonOptions`]: ../command/struct.CommonOptions.html
/// [`Error`]: ../error/struct.Error.html
/// 
pub fn collect<'i, I>(
	stall_dir: &Path,
	stall: &Stall,
	files: I,
	force: bool,
	dry_run: bool,
	common: CommonOptions) 
	-> Result<(), Error>
	where I: IntoIterator<Item=&'i Path>
{
	let _span = span!(Level::INFO, "collect").entered();

	if stall.is_empty() {
		if !common.quiet {
			println!("No files in stall. Use `add` command to place files \
			in the stall.");
		}
		// Nothing to do if there's no data.
		return Ok(());
	} 

	// Identify stall files to process.
	let selected = files
		.into_iter()
		.map(|f| stall
			.entry_local(f)
			.ok_or_else(|| anyhow!("unrecognized stall entry: {}",
				f.display())))
		.collect::<Result<Vec<_>, _>>()?;

	let entries = if selected.is_empty() {
		Either::Left(stall.entries())
	} else {
		Either::Right(selected.into_iter())
	};

	let mut out = std::io::stdout();

	// Setup and print stall directory.
	if common.color.enabled() {
		writeln!(&mut out, "{} {}",
			"Stall directory:".bright_white(),
			stall_dir.display())?;
	} else {
		writeln!(&mut out, "{} {}",
			"Stall directory:",
			stall_dir.display())?;
	}

	// Process each entry table.
	Entry::write_status_action_header(&mut out, &common)?;
	for entry in entries {
		entry.collect(
			&mut out,
			stall_dir,
			force,
			dry_run,
			&common)?;
	}

	Ok(())
}
