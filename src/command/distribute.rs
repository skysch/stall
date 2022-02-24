////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Distribute files from a stall.
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
use std::path::PathBuf;
use std::io::Write as _;


////////////////////////////////////////////////////////////////////////////////
// distribute
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-distribute' command.
///
/// This will iterate over each file, checking if it is more recent than its
/// counterpart in the stall directory by comparing their modification times.
/// If the file is older than the one in the stall directory, it will be 
/// overwritten by the one in the stall directory.
///
/// ### Command line options
///
/// The `--force` option will cause the overwrite to occur even if the file
/// is newer than the one in the stall directory.
///
/// The `--error` option will cause the function to return with an error if any
/// of the distributed files cannot be opened or read. Further files will not be
/// processed.
///
/// The `--dry-run` option will prevent any file copying, but all of the normal
/// checks and outputs will be emitted.
///
/// The `--verbose`, `--quiet`, `--xtrace`, and `--short-names` options will
/// change which outputs are produced.
///
/// ### Parameters
/// + `from`: The 'stall directory' to distribute from. Takes a generic argument
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
pub fn distribute<P>(
	stall_dir: P,
	stall: &Stall,
	files: &[PathBuf],
	force: bool,
	dry_run: bool,
	common: CommonOptions) 
	-> Result<(), Error>
	where 
		P: AsRef<Path>,
{
	let _span = span!(Level::INFO, "distribute").entered();

	if stall.is_empty() {
		if !common.quiet {
			println!("No files in stall. Use `add` command to place files \
			in the stall.");
		}
		// Nothing to do if there's no data.
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

	// Process each entry table.
	Entry::write_status_action_header(&mut out, &common)?;


	let entries = if files.is_empty() {
		Either::Left(stall.entries())
	} else {
		let selected = files
			.iter()
			.map(|f| stall
				.entry_local(f.as_path())
				.ok_or_else(|| anyhow!("unrecognized stall entry: {}",
					f.display())))
			.collect::<Result<Vec<_>, _>>()?;
		Either::Right(selected.into_iter())
	};

	for entry in entries {
		entry.distribute(
			&mut out,
			stall_dir,
			force,
			dry_run,
			&common)?;
	}

	Ok(())
}
