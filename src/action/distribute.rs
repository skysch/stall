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


use anyhow::Error;
use log::*;

use std::path::PathBuf;


/// Executes the 'stall distribute' command.
pub fn distribute(
	from: PathBuf,
	_common: CommonOptions,
	config: Config) 
	-> Result<(), Error>
{
	for target in config.targets {
		info!("Distribute {:?}", target);
	}
	info!(".. from stall {:?}", from);

	Ok(())
}