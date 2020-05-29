////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Collect files into a stall.
////////////////////////////////////////////////////////////////////////////////


use crate::CommonOptions;
use crate::Config;
use crate::error::Error;
use crate::error::InvalidFile;
use crate::action::copy_file;
use crate::action::CopyMethod;

use log::*;

use std::path::Path;

/// Executes the 'stall collect' command.
pub fn collect<P>(
    into: P,
    common: CommonOptions,
    config: Config) 
    -> Result<(), Error>
    where P: AsRef<Path>
{
    let into = into.as_ref();
    for source in &config.files[..] {
        let file_name = source.file_name().ok_or(InvalidFile)?;

        info!("Collecting {:?}", source);

        let last_modified_file = source.metadata()?.modified()?;
        let target = into.join(file_name);
        
        if !target.exists() {
            info!("File not yet in stall.")
        } else if target.metadata()?.modified()? > last_modified_file {
            info!("File is newer than version in stall.")
        } else if !common.force {
            info!("File is older than version in stall. Skipping collection.");
            continue;
        } else {
            info!("File is older than version in stall. Forcing collection.");
        }

        // If we got this far, we're collecting this file.
        let copy_method = match common.no_run {
            true  => CopyMethod::None,
            false => CopyMethod::Subprocess,
        };
        copy_file(source, &target, copy_method)?;

    }

    Ok(())
}
