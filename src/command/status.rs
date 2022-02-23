////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Print the stall status.
////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////
// collect
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall collect' command.
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
// Release checklist:
// [0.1.0] Documentation accuracy check.
// [0.1.0] Documentation links test.
// [0.1.0] Style check.
//
pub fn status<P>(
	stall_dir: P,
	data: &Stall,
	common: CommonOptions) 
	-> Result<(), Error>
	where 
		P: AsRef<Path>,
{
	let _span = span!(Level::INFO, "status").entered();

	let into = into.as_ref();
	if !common.quiet {
		if data.is_empty() {
			println!("No files to distribute. Use `add` command to place files \
				in the stall.");
			return Ok(());
		}
	} else {
		// Nothing to do if asking for status with --quiet.
		return Ok(());
	}

	for entry in data.entries() {

	}

	Ok(())
}
