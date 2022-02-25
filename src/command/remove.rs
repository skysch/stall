////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Add a file to a stall.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::CommonOptions;
use crate::Stall;

// External library imports.
use anyhow::Error;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// remove
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-rm' command.
///
/// Adds entries into a stall file.
///
/// ### Parameters
///
/// + `stall`: The loaded `Stall` data.
/// + `files`: An iterator over the [`Path`]s of the files to remove.
/// + `delete_stall_dir`: The stall directory to delete from, or None if no
/// delete should occur.
/// + `remote_naming`: Lookup stall entries using the remote name instead of the
/// local name.
/// + `dry_run`: Do not modify any files.
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
pub fn remove<'i, I>(
    stall: &mut Stall,
    files: I,
    delete_stall_dir: Option<&Path>,
    remote_naming: bool,
    dry_run: bool,
    common: &CommonOptions)
    -> Result<(), Error>
    where I: IntoIterator<Item=&'i Path>
{
    let _span = span!(Level::INFO, "add").entered();
    if dry_run && common.quiet { return Ok(()); }

    for file in files.into_iter() {
        event!(Level::DEBUG, "Remove entry with path: {:?}", file);

        if dry_run {
            println!("remove stall entry with {} path {}",
                if remote_naming { "remote" } else { "local" },
                file.display());
            return Ok(())
        }

        let removed = if remote_naming {
            stall.remove_remote(file)
        } else {
            stall.remove_local(file)
        };

        if let (Some((local, _)), Some(stall_dir)) = (removed, delete_stall_dir)
        {
            let path = stall_dir.to_owned().join(local);
            if let Err(e) = std::fs::remove_file(path) {

                event!(Level::WARN, "{}", e);
                if common.promote_warnings_to_errors {
                    return Err(e.into());
                }
            }
        }
    }

    Ok(())
}
