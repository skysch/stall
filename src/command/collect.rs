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
use crate::entry::Stall;
use crate::error::InvalidFile;
use crate::error::MissingFile;
use crate::action::Action;
use crate::action::copy_file;
use crate::action::CopyMethod;
use crate::action::print_status_header;
use crate::action::print_status_line;
use crate::action::State;

// External library imports.
use anyhow::Error;
use anyhow::Context;
use tracing::event;
use tracing::span;
use tracing::Level;
use colored::Colorize as _;

// Standard library imports.
use std::path::Path;

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
pub fn collect<'f, P>(
    into: P,
    data: &Stall,
    force: bool,
    common: CommonOptions) 
    -> Result<(), Error>
    where 
        P: AsRef<Path>,
{
    let _span = span!(Level::INFO, "collect").entered();

    let into = into.as_ref();
    if !common.quiet {
        println!("{} {}", 
            "Destination directory:".bright_white(),
            into.display());
        if data.is_empty() {
            println!("No files to distribute. Use `add` command to place files \
                in the stall.");
            return Ok(());
        }
    }

    let copy_method = match common.dry_run {
        true  => CopyMethod::None,
        false => CopyMethod::Subprocess,
    };
    event!(Level::DEBUG, "Copy method: {:?}", copy_method);

    print_status_header(&common);

    for source in data.entries().map(|e| e.remote_path()) {
        event!(Level::DEBUG, "Processing source file: {:?}", source);
        let file_name = source.file_name().ok_or(InvalidFile)?;
        let target = into.join(file_name);

        use State::*;
        use Action::*;
        match (source.exists(), target.exists()) {
            // Both files exist, compare modify dates.
            (true,  true) => {
                let source_last_modified = source.metadata()
                    .with_context(|| "load source metadata")?
                    .modified()
                    .with_context(|| "load source modified time")?;
                event!(
                    Level::TRACE, 
                    "Source last modified: {:?}",
                    source_last_modified);
                let target_last_modified = target.metadata()
                    .with_context(|| "load target metadata")?
                    .modified()
                    .with_context(|| "load target modified time")?;
                event!(
                    Level::TRACE, 
                    "Target last modified: {:?}",
                    source_last_modified);

                if source_last_modified > target_last_modified {
                    print_status_line(Newer, Copy, source, &common);

                } else if force {
                    print_status_line(Force, Copy, source, &common);

                } else {
                    print_status_line(Older, Skip, source, &common);
                    continue;
                }
            },

            // Source exists, but not target.
            (true, false) => print_status_line(Found, Copy, source, &common),

            // Source does not exist.
            (false, _) => if common.promote_warnings_to_errors {
                print_status_line(Error, Stop, source, &common);
                return Err(MissingFile { path: source.into() }.into());
            } else {
                print_status_line(Error, Skip, source, &common);
                continue;
            },
        }

        // If we got this far, we're collecting this file.
        copy_file(source, &target, copy_method)?;
    }

    Ok(())
}
