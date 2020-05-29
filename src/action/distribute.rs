////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Distribute files from a stall.
////////////////////////////////////////////////////////////////////////////////


use crate::CommonOptions;
use crate::Config;
use crate::error::Error;
use crate::error::InvalidFile;
use crate::action::copy_file;
use crate::action::CopyMethod;

use log::*;

use std::path::Path;


/// Executes the 'stall distribute' command.
pub fn distribute<P>(
	from: P,
	common: CommonOptions,
	config: Config) 
	-> Result<(), Error>
	where P: AsRef<Path>
{
	let from = from.as_ref();
    for target in &config.files[..] {
        let file_name = target.file_name().ok_or(InvalidFile)?;

        info!("Collecting {:?}", target);

        let last_modified_file = target.metadata()?.modified()?;
        let source = from.join(file_name);
        
        if !source.exists() {
            info!("File not yet in stall.")
        } else if source.metadata()?.modified()? > last_modified_file {
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
        copy_file(&source, target, copy_method)?;

    }

    Ok(())
}

