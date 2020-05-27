////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer.
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

use stall::utility::application_root_dir;

use anyhow::Error;

use structopt::StructOpt;

use std::path::PathBuf;

#[derive(Debug)]
#[derive(StructOpt)]
pub struct CommonOptions {
	#[structopt(short, long)]
	no_run: bool,
	#[structopt(short, long)]
	use_stall_file: Option<PathBuf>,
}


#[derive(Debug)]
#[derive(StructOpt)]
#[structopt(name = "stall")]
enum CommandOptions {
    Collect {
    	#[structopt(short, long)]
    	into: Option<PathBuf>,
    	#[structopt(flatten)]
    	common_options: CommonOptions,
    },

    Distribute {
    	#[structopt(short, long)]
    	from: Option<PathBuf>,
    	#[structopt(flatten)]
    	common_options: CommonOptions,
    },
}


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() -> Result<(), Error> {
	let opts = CommandOptions::from_args();
	println!("application root: {:#?}", application_root_dir());
	println!("{:#?}", opts);

	Ok(())
}
