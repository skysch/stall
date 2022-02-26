////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Rename a file within a stall.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::CommonOptions;
use crate::Stall;

// External library imports.
use anyhow::anyhow;
use anyhow::Error;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
// rename
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-mv' command.
///
/// This will rename a file in the stall
///
/// ### Parameters
///
/// + `stall`: The loaded [`Stall`] data.
/// + `from`: The current name of the stalled file.
/// + `to`: The new name of the stalled file.
/// + `move_stall_dir`: The stall directory to move the files within, or `None`
/// if no move should occur.
/// + `force`: Force overwrite if the new file already exists.
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
pub fn rename(
    stall: &mut Stall,
    from: &Path,
    to: &Path,
    move_stall_dir: Option<&Path>,
    force: bool,
    dry_run: bool,
    common: &CommonOptions)
    -> Result<(), Error>
{
    let _span = span!(Level::INFO, "rename").entered();
    if dry_run && common.quiet { return Ok(()); }

    if stall.is_empty() {
        if !common.quiet {
            println!("No files in stall. Use `add` command to place files \
            in the stall.");
        }
        // Nothing to do if there's no data.
        return Ok(());
    }

    if stall.entry_local(to).is_some() {
        // This move will overwrite an existing file.
        if !force {
            return Err(anyhow!("stall file already exists: {:?}\nUse --force \
                option to overwrite it.", to))
        }
    }

    if !dry_run {
        let (_, r) = stall
            .remove_local(from)
            .ok_or_else(|| anyhow!("no stall file found: {:?}",
                from.display()))?;
        stall.insert(to.to_path_buf(), r);

        if let Some(stall_dir) = move_stall_dir {
            let old = stall_dir.join(from);
            let new = stall_dir.join(to);
            let status = std::process::Command::new("mv")
                .args([old, new])
                .arg("-f")
                .status()?;

            if !status.success() && !common.quiet {
                println!("Failed to move files.");
            }
        }
    }

    Ok(())
}
