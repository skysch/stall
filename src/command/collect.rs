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
/// This will iterate over each file, checking if it is more recent than its
/// counterpart in the stall directory by comparing their modification times.
/// If the file is newer than the one in the stall directory, it will be copied
/// into the stall directory, overwriting the existing file.
///
/// ### Command line options
///
/// The `--force` option will cause the overwrite to occur even if the file
/// is older than the one in the stall directory.
///
/// The `--error` option will cause the function to return with an error if any
/// of the collected files cannot be opened or read. Further files will not be
/// processed.
///
/// The `--dry-run` option will prevent any file copying, but all of the normal
/// checks and outputs will be emitted.
///
/// The `--verbose`, `--quiet`, `--xtrace`, and `--short-names` options will
/// change which outputs are produced.
///
/// ### Parameters
/// + `into`: The 'stall directory' to collect into. Takes a generic argument
/// that implements [`AsRef`]`<`[`Path`]`>`.
/// + `common`: The [`CommonOptions`] to use for the command.
/// + `files`: An iterator over the [`Path`]s of the files to collect.
///
/// ### Errors
/// 
/// Returns an [`Error`] if both files exist but their metadata can't be read, or if the copy operation fails for some reason.
/// 
/// [`AsRef`]: https://doc.rust-lang.org/stable/std/convert/trait.AsRef.html
/// [`Path`]: https://doc.rust-lang.org/stable/std/path/struct.Path.html
/// [`CommonOptions`]: ../command/struct.CommonOptions.html
/// [`Error`]: ../error/struct.Error.html
/// 
pub fn collect<'i, P, I>(
	stall_dir: P,
	stall: &Stall,
	files: I,
	force: bool,
	dry_run: bool,
	common: CommonOptions) 
	-> Result<(), Error>
	where 
		P: AsRef<Path>,
		I: IntoIterator<Item=&'i Path>
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
