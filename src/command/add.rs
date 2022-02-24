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
use anyhow::anyhow;
use anyhow::Error;
use tracing::event;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// add
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-add' command.
///
/// Adds entries into a stall file.
///
/// ### Parameters
///
/// + `stall`: The loaded `Stall` data.
/// + `files`: An iterator over the [`Path`]s of the files to add.
/// + `rename`: The name to use for any local stall path. (If use with multiple
/// files, they will all end up with the same name.)
/// + `into`: A subdirectory within the stall to place the files.
/// + `collect_stall_dir`: The stall directory to collect into, or `None` if no
/// collect should occur.
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
pub fn add<'i, I>(
    stall: &mut Stall,
    files: I,
    rename: Option<&Path>,
    into: Option<&Path>,
    collect_stall_dir: Option<&Path>,
    dry_run: bool,
    common: &CommonOptions)
    -> Result<(), Error>
    where I: IntoIterator<Item=&'i Path>
{
    let _span = span!(Level::INFO, "add").entered();
    if dry_run && common.quiet { return Ok(()); }

    for remote in files.into_iter() {
        event!(Level::DEBUG, "Add entry with remote path: {:?}", remote);

        let mut local = PathBuf::new();

        if let Some(path) = into {
            local.push(path);
        }

        if let Some(f) = rename {
            local.push(f);
        } else if let Some(f) = remote.file_name() {
            local.push(f)
        } else {
            if !common.quiet {
                println!("Invalid remote file name: {}", remote.display());
            }
            event!(Level::WARN, "invalid remote file name: {:?}", remote);
            if common.promote_warnings_to_errors {
                return Err(anyhow!("invalid remote file name: {:?}", remote));
            }
            continue;
        };

        event!(Level::DEBUG, "      ... with local path: {:?}", local);

        if dry_run {
            println!("Insert stall entry {} from {}",
                local.display(),
                remote.display());
            return Ok(())
        }

        stall.insert(local, remote.to_owned());

        if let Some(stall_dir) = collect_stall_dir {
            let mut out = std::io::stdout();

            stall.entry_remote(remote)
                .expect("get added entry for collect")
                .collect(&mut out, stall_dir, false, dry_run, common)?;
        }
    }

    Ok(())
}
