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
/// This will 
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
pub fn add<'i, I>(
    stall_dir: &Path,
    stall: &mut Stall,
    files: I,
    rename: Option<&Path>,
    into: Option<&Path>,
    collect: bool,
    common: &CommonOptions)
    -> Result<(), Error>
    where I: IntoIterator<Item=&'i Path>
{
    let _span = span!(Level::INFO, "add").entered();

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

        stall.insert(local, remote.to_owned());

        if collect {
            let mut out = std::io::stdout();

            stall.entry_remote(remote)
                .expect("get added entry for collect")
                .collect(&mut out, stall_dir, false, false, common)?;
        }
    }

    Ok(())
}
