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
use crate::action::copy_file;
use crate::action::CopyMethod;

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
    info!("{}", "    STATE ACTION FILE".bright_white().bold());
    

    for source in &config.files[..] {
        debug!("Processing source file: {:?}", source);
        let file_name = source.file_name().ok_or(InvalidFile)?;
        let target = into.join(file_name);

        match (source.exists(), target.exists()) {
            // Both files exist, compare modify dates.
            (true,  true) => {
                let source_last_modified = source.metadata()
                    .with_context(|| "load source metadata")?
                    .modified()
                    .with_context(|| "load source modified time")?;
                let target_last_modified = target.metadata()
                    .with_context(|| "load target metadata")?
                    .modified()
                    .with_context(|| "load target modified time")?;

                if source_last_modified > target_last_modified {
                    info!("    {}{} {}",
                        "newer ".bright_green(),
                        "copy  ".bright_green(),
                        source.display());

                } else if common.force {
                    info!("    {}{} {}",
                        "force ".bright_white(),
                        "copy  ".bright_green(),
                        source.display());

                } else {
                    info!("    {}{} {}",
                        "older ".bright_yellow(),
                        "skip  ".bright_white(),
                        source.display());
                    continue;
                }
            },

            // Source exists, but not target.
            (true, false) => info!("    {}{} {}",
                "found ".bright_green(),
                "copy  ".bright_green(),
                source.display()),

            // Source does not exist.
            (false, _) => if common.promote_warnings_to_errors {
                info!("    {}{} {}",
                    "error ".bright_red(),
                    "stop  ".bright_red(),
                    source.display());
                return Err(MissingFile { path: source.clone() }.into());
            } else {
                info!("    {}{} {}",
                    "error ".bright_red(),
                    "skip  ".bright_white(),
                    source.display());
                continue;
            },
        }

        // If we got this far, we're collecting this file.
        let copy_method = match common.dry_run {
            true  => CopyMethod::None,
            false => CopyMethod::Subprocess,
        };
        copy_file(source, &target, copy_method)?;
    }

    Ok(())
}
