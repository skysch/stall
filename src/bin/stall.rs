////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use stall::action;
use stall::CommandOptions;
use stall::Config;
use stall::DEFAULT_CONFIG_PATH;
use stall::error::Context;
use stall::error::Error;
use stall::logger::Logger;

// External library imports.
use structopt::StructOpt;
use log::*;
pub use log::LevelFilter;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() -> Result<(), Error> {
    // Parse command line options.
    let opts = CommandOptions::from_args();

    // Find the path for the config file.
    // We do this up front because current_dir might fail due to access
    // problems, and we only want to error out if we really need to use it.
    let stall_dir = opts.stall_dir()?;
    let config_path = match &opts.common().use_config {
        Some(path) => path.clone(),
        None       => stall_dir.join(DEFAULT_CONFIG_PATH),
    };

    // Load the config file.
    let mut config = Config::from_path(&config_path)
        .with_context(|| format!("Unable to load config file: {:?}",
            config_path))?;
    config.normalize_paths(&stall_dir);

    // Setup and start the global logger.
    let mut logger =  Logger::from_config(config.logger_config.clone());
    for (context, level) in &config.log_levels {
        logger = logger.level_for(context.clone(), *level);
    }
    let common = opts.common();
    match (common.verbose, common.quiet, common.trace) {
        (_, _, true) => logger.level_for("stall", LevelFilter::Trace).start(),
        (_, true, _) => (),
        (true, _, _) => logger.level_for("stall", LevelFilter::Debug).start(),
        _            => logger.level_for("stall", LevelFilter::Info).start(),
    }

    // Print version information.
    debug!("Stall version: {}", env!("CARGO_PKG_VERSION"));
    let rustc_meta = rustc_version_runtime::version_meta();
    trace!("Rustc version: {} {:?}", rustc_meta.semver, rustc_meta.channel);
    if let Some(hash) = rustc_meta.commit_hash {
        trace!("Rustc git commit: {}", hash);
    }
    trace!("Options: {:?}", opts);
    trace!("Config: {:?}", config); 

    // Dispatch to appropriate commands.
    use CommandOptions::*;
    match opts {
        Collect { common, .. } => action::collect(
            stall_dir,
            config.files.iter().map(|p| &**p),
            common),

        Distribute { common, .. } => action::distribute(
            stall_dir,
            config.files.iter().map(|p| &**p),
            common),
    }
}
