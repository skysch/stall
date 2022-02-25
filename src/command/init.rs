////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Initialize a stall.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::CommonOptions;
use crate::Stall;

// External library imports.
use anyhow::Error;
use tracing::Level;
use tracing::span;

// Standard library imports.
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// init
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall-init' command.
///
/// Initializes a stall directory.
///
/// ### Parameters
///
/// + `stall_dir`: The stall directory to initialize.
/// + `stall`: The loaded [`Stall`] data.
/// + `dry_run`: Do not modify any files.
/// + `common`: The [`CommonOptions`] to use for the command.
///
/// ### Errors
/// 
/// Returns an [`Error`] if writing the stall file fails.
/// 
/// [`Stall`]: ../struct.Stall.html
/// [`CommonOptions`]: ../command/struct.CommonOptions.html
/// [`Error`]: ../error/struct.Error.html
/// 
pub fn init(
    _stall_dir: &Path,
    stall: &mut Stall,
    dry_run: bool,
    common: &CommonOptions)
    -> Result<(), Error>
{
    let _span = span!(Level::INFO, "init").entered();
    if dry_run && common.quiet { return Ok(()); }

    let written = if dry_run {
        true
    } else {
        stall.write_to_load_path_if_new()?
    };

    if !common.quiet {
        if written {
            println!("Created new stall file at {}", stall
                .load_path()
                .expect("retrieve stall load path")
                .display());
        } else {
            println!("Stall file already exists at {}", stall
                .load_path()
                .expect("retrieve stall load path")
                .display());
        }
    }

    Ok(())
}
