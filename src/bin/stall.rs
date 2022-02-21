////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Application entry point.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use stall::application::Config;
use stall::application::TraceGuard;
use stall::action;
use stall::CommandOptions;

// External library imports.
use anyhow::Context;
use anyhow::Error;
use clap::Parser;
use tracing::event;
use tracing::Level;
use tracing::span;



////////////////////////////////////////////////////////////////////////////////
// main
////////////////////////////////////////////////////////////////////////////////
/// The application entry point.
pub fn main() {
    // The worker_guard holds the worker thread handle for the nonblocking
    // trace writer. It should be held until all tracing is complete, as any
    // trace spans or events after it is dropped will be ignored.
    let mut trace_guard = TraceGuard::default();

    if let Err(err) = main_facade(&mut trace_guard) {
        // Trace errors without coloring.
        colored::control::set_override(false);
        event!(Level::ERROR, "{:?}", err);

        // Print errors to stderr and exit with error code.
        colored::control::unset_override();
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
}

////////////////////////////////////////////////////////////////////////////////
// main_facade
////////////////////////////////////////////////////////////////////////////////
/// The application facade for propagating user errors.
pub fn main_facade(trace_guard: &mut TraceGuard) -> Result<(), Error> {
    // Parse command line options.
    let command = CommandOptions::try_parse()?;
    let common = command.common();

    // Find the path for the config file.
    // We do this up front because current_dir might fail due to access
    // problems, and we only want to error out if we really need to use it.
    let stall_dir = command.stall_dir()?;
    let config_path = match &common.use_config {
        Some(path) => path.clone(),
        None       => stall_dir.join(Config::DEFAULT_CONFIG_PATH),
    };

    // Load the config file.
    let mut config_load_status = Ok(());
    let config = Config::read_from_path(&config_path)
        .with_context(|| format!("Unable to load config file: {:?}", 
            config_path))
        .unwrap_or_else(|e| {
            // Store the error for output until after the logger is configured.
            config_load_status = Err(e);
            Config::new().with_load_path(&config_path)
        });

    // Initialize the global tracing subscriber.
    let base_level = match (common.verbose, common.quiet, common.trace) {
        (_, _, true) => Level::TRACE,
        (_, true, _) => Level::WARN,
        (true, _, _) => Level::INFO,
        _            => Level::WARN,
    };
    *trace_guard = config.trace_config.init_global_default(base_level)?;
    let _span = span!(Level::INFO, "main").entered();


    // Print version information.
    event!(Level::INFO, "Atma version: {}", env!("CARGO_PKG_VERSION"));
    #[cfg(feature = "png")]
    event!(Level::INFO, "PNG support enabled.");
    #[cfg(feature = "termsize")]
    event!(Level::INFO, "Terminal size detection support enabled.");
    let rustc_meta = rustc_version_runtime::version_meta();
    event!(Level::DEBUG, "Rustc version: {} {:?}", rustc_meta.semver, rustc_meta.channel);
    if let Some(hash) = rustc_meta.commit_hash {
        event!(Level::DEBUG, "Rustc git commit: {}", hash);
    }
    event!(Level::DEBUG, "{:#?}", common);
    event!(Level::DEBUG, "{:#?}", command);
    event!(Level::DEBUG, "{:#?}", config);

    // Dispatch to appropriate commands.
    use CommandOptions::*;
    match command {
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

