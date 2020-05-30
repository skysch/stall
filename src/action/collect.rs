////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Collect files into a stall.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::CommonOptions;
use crate::Config;
use crate::error::Error;
use crate::error::InvalidFile;
use crate::error::MissingFile;
use crate::error::Context;
use crate::action::Action;
use crate::action::copy_file;
use crate::action::CopyMethod;
use crate::action::print_status_header;
use crate::action::print_status_line;
use crate::action::State;

// External library imports.
use log::*;
use colored::Colorize as _;

// Standard library imports.
use std::path::Path;

////////////////////////////////////////////////////////////////////////////////
// collect
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall collect' command.
pub fn collect<P>(
    into: P,
    common: CommonOptions,
    config: Config) 
    -> Result<(), Error>
    where P: AsRef<Path>
{
    let into = into.as_ref();
    info!("{} {}", 
        "Destination directory:".bright_white(),
        into.display());

    let copy_method = match common.dry_run {
        true  => CopyMethod::None,
        false => CopyMethod::Subprocess,
    };
    debug!("Copy method: {:?}", copy_method);

    print_status_header();

    for source in &config.files[..] {
        debug!("Processing source file: {:?}", source);
        let file_name = source.file_name().ok_or(InvalidFile)?;
        let target = into.join(file_name);

        use State::*;
        use Action::*;
        match (source.exists(), target.exists()) {
            // Both files exist, compare modify dates.
            (true,  true) => {
                let source_last_modified = source.metadata()
                    .with_context(|| "load source metadata")?
                    .modified()
                    .with_context(|| "load source modified time")?;
                trace!("Source last modified: {:?}", source_last_modified);
                let target_last_modified = target.metadata()
                    .with_context(|| "load target metadata")?
                    .modified()
                    .with_context(|| "load target modified time")?;
                trace!("Target last modified: {:?}", source_last_modified);

                if source_last_modified > target_last_modified {
                    print_status_line(Newer, Copy, source, &common);

                } else if common.force {
                    print_status_line(Force, Copy, source, &common);
                    
                } else {
                    print_status_line(Older, Skip, source, &common);
                    continue;
                }
            },

            // Source exists, but not target.
            (true, false) => print_status_line(Found, Copy, source, &common),

            // Source does not exist.
            (false, _) => if common.promote_warnings_to_errors {
                print_status_line(Error, Stop, source, &common);
                return Err(MissingFile { path: source.clone() }.into());
            } else {
                print_status_line(Error, Skip, source, &common);
                continue;
            },
        }

        // If we got this far, we're collecting this file.
        copy_file(source, &target, copy_method)?;
    }

    Ok(())
}
