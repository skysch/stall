////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer.
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Command implementations.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

mod collect;
mod distribute;

pub use collect::*;
pub use distribute::*;

use crate::error::Error;

use log::*;

use std::path::Path;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

////////////////////////////////////////////////////////////////////////////////
// Common file copy function.
////////////////////////////////////////////////////////////////////////////////
/// Copies a file from `source` to `target` using the given `CopyMethod`
pub fn copy_file(source: &Path, target: &Path, method: CopyMethod)
	-> Result<(), Error>
{
	use CopyMethod::*;
	match method {
		None => info!("no-run flag was specified: \
            Not copying data from {:?} to {:?}", source, target),

		Subprocess => {
			let status = if cfg!(target_os = "windows") {
			    std::process::Command::new("COPY")
			            .arg(source)
			            .arg(target)
			            .status()
			} else {
			    std::process::Command::new("cp")
			            .arg(source)
			            .arg(target)
			            .status()
			};
			status.expect("execute copy command");
		},
		
		Internal => {
            info!("Copying data from {:?} into {:?}", source, target);

            let mut source = OpenOptions::new()
                .read(true)
                .open(source)?;
            let mut target = OpenOptions::new()
                .write(true)
                .create(true)
                .open(target)?;

            // TODO: This might be more efficient with buffers.
            // TODO: It might be safer to read/write in blocks.
            // TODO: If any of these fail, we shouldn't just bail out.
            // TODO: Check common.warn_error.
            let mut buffer: Vec<u8> = Vec::new();
            source.read_to_end(&mut buffer)?;
            target.write_all(&buffer[..])?;
            target.flush()?;
        },
	}
	Ok(())
}


/// The method to use when copying files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyMethod {
	/// Do not copy files.
	None,
	/// Copy files using a command in a subprocess.
	Subprocess,
	/// Copy files using an internal buffer.
	Internal,
}
