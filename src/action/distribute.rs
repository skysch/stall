////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Distribute files from a stall.
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
// distribute
////////////////////////////////////////////////////////////////////////////////
/// Executes the 'stall distribute' command.
pub fn distribute<P>(
    from: P,
    common: CommonOptions,
    config: Config) 
    -> Result<(), Error>
    where P: AsRef<Path>
{
    let from = from.as_ref();
    info!("{} {}", 
        "Source directory:".bright_white(),
        from.display());
    info!("{}", "    STATE ACTION FILE".bright_white().bold());

    for target in &config.files[..] {
        debug!("Processing target file: {:?}", target);
        let file_name = target.file_name().ok_or(InvalidFile)?;
        let source = from.join(file_name);
        
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
                return Err(MissingFile { path: source.into() }.into());
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
        copy_file(&source, target, copy_method)?;
    }

    Ok(())
}
