////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////

use stall::action;
use stall::CommandOptions;
use stall::Config;
use stall::DEFAULT_CONFIG_PATH;
use stall::logger::Logger;

use structopt::StructOpt;
use log::*;

use anyhow::Error;
use anyhow::Context;


////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() -> Result<(), Error> {
    // Parse command line options.
    let opts = CommandOptions::from_args();
    debug!("{:#?}", opts);


    // Find the path for the config file.
    let current_dir = std::env::current_dir()?;
    let config_path = match &opts.common().use_config {
        Some(path) => path.clone(),
        None       => current_dir.join(DEFAULT_CONFIG_PATH),
    };

    // Load the config file.
    let mut config = Config::load_from_file(&config_path)
        .with_context(
            || format!("Unable to load config file: {:?}", config_path))?;
    config.normalize_paths(&current_dir);
    info!("Stall file: {}", config);

    // Setup and start the global logger.
    let mut logger =  Logger::from_config(config.logger_config.clone());
    for (context, level) in &config.log_levels {
        logger = logger.level_for(context.clone(), *level);
    }
    logger.start();

    // Print version information.
    info!("Stall version: {}", env!("CARGO_PKG_VERSION"));
    let rustc_meta = rustc_version_runtime::version_meta();
    info!("Rustc version: {} {:?}", rustc_meta.semver, rustc_meta.channel);
    if let Some(hash) = rustc_meta.commit_hash {
        info!("Rustc git commit: {}", hash);
    }

    // Dispatch to appropriate commands.
    use CommandOptions::*;
    match opts {
        Collect { into, common } => action::collect(
            into.unwrap_or(current_dir),
            common,
            config),

        Distribute { from, common } => action::distribute(
            from.unwrap_or(current_dir),
            common,
            config),
    }
}
