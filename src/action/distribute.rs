////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
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
use colored::Colorize as _;

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
    info!("{} {}", 
        "Copy source:".bright_white(),
        from.display());
    info!("{}", "    STATE ACTION FILE".bright_white());

    for target in &config.files[..] {
        debug!("Processing target: {:?}", target);
        let file_name = target.file_name().ok_or(InvalidFile)?;

        let source = from.join(file_name);
        let target_last_modified = target.metadata()?.modified()?;
        
        if !source.exists() {
            info!("    {}{} {}",
                "error ".bright_red(),
                "skip  ".bright_white(),
                target.display());
            continue;

        } else if source.metadata()?.modified()? > target_last_modified {
            info!("    {}{} {}",
                "newer ".bright_green(),
                "copy  ".bright_green(),
                target.display());

        } else if !common.force {
            info!("    {}{} {}",
                "force ".bright_white(),
                "copy  ".bright_green(),
                target.display());

        } else {
            info!("    {}{} {}",
                "older ".bright_yellow(),
                "skip  ".bright_white(),
                target.display());
            continue;
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
