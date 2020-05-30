////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
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
use colored::Colorize as _;

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
    info!("{} {}", 
        "Copy destination:".bright_white(),
        into.display());
    info!("{}", "    STATE ACTION FILE".bright_white());
    

    for source in &config.files[..] {
        debug!("Processing source: {:?}", source);
        let file_name = source.file_name().ok_or(InvalidFile)?;

        let target = into.join(file_name);
        let target_last_modified = target.metadata()?.modified()?;
        

        if !target.exists() {
            info!("    {}{} {}",
                "found ".bright_green(),
                "copy  ".bright_green(),
                source.display());

        } else if source.metadata()?.modified()? > target_last_modified {
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

        // If we got this far, we're collecting this file.
        let copy_method = match common.dry_run {
            true  => CopyMethod::None,
            false => CopyMethod::Subprocess,
        };
        copy_file(source, &target, copy_method)?;
    }

    Ok(())
}
