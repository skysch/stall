////////////////////////////////////////////////////////////////////////////////
// Sunflower Game Engine
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Crate-wide logging infrastructure.
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// External library imports.
use fern::colors::Color;

use log::*;

use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::env;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

// Exports.
pub use log::LevelFilter;


////////////////////////////////////////////////////////////////////////////////
// LoggerConfig
////////////////////////////////////////////////////////////////////////////////
/// Logger configuration parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Determines how to log to the terminal.
    #[serde(default = "LoggerConfig::default_stdout_log_output")]
    pub stdout_log_output: StdoutLogOutput,
    
    #[serde(default = "LoggerConfig::default_level_filter")]
    /// Sets the level filter for the logger.
    pub level_filter: LevelFilter,
    
    #[serde(default = "LoggerConfig::default_log_path")]
    /// Enables logging to the file at the given path.
    pub log_path: Option<PathBuf>,

    #[serde(default = "LoggerConfig::default_allow_env_override")]
    /// Enables config values to be overriden by environment variables.
    pub allow_env_override: bool,
}

impl LoggerConfig {
    /// Returns the default stdout log ouput setting.
    #[inline(always)]
    fn default_stdout_log_output() -> StdoutLogOutput {
        StdoutLogOutput::Colored
    }

    /// Returns the default level filter.
    #[inline(always)]
    fn default_level_filter() -> LevelFilter {
        LevelFilter::Info
    }

    /// Returns the default log path.
    #[inline(always)]
    fn default_log_path() -> Option<PathBuf> {
        None
    }

    /// Returns the default setting for allowing environment variable overrides.
    #[inline(always)]
    fn default_allow_env_override() -> bool {
        true
    }
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            stdout_log_output: LoggerConfig::default_stdout_log_output(),
            level_filter: LoggerConfig::default_level_filter(),
            log_path: LoggerConfig::default_log_path(),
            allow_env_override: LoggerConfig::default_allow_env_override(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// StdoutLogOutput
////////////////////////////////////////////////////////////////////////////////
/// Options for logging to the terminal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StdoutLogOutput {
    /// Disables logging to the terminal.
    Off,
    /// Enables logging to the terminal without colored output.
    Plain,
    /// Enables logging to the terminal with colored output on supported 
    /// platforms.
    Colored,
}



////////////////////////////////////////////////////////////////////////////////
// Logger
////////////////////////////////////////////////////////////////////////////////
/// Logger interface for creating and setting up the global logger.
#[allow(missing_debug_implementations)]
pub struct Logger {
    /// The logging dispatcher.
    dispatch: fern::Dispatch,
}

impl Logger {
    
    ////////////////////////////////////////////////////////////////////////////
    // Constructors
    ////////////////////////////////////////////////////////////////////////////
    
    /// Constructs a new Logger with the default settings.
    fn new() -> Self {
        let dispatch = fern::Dispatch::new().format(|out, message, record| {
            match record.level() {
                Level::Info => out.finish(*message),
                _ => out.finish(format_args!(
                    "[{level}][{target}] {message}",
                    level = record.level(),
                    target = record.target(),
                    message = message)),
            }
        });

        Self { dispatch }
    }

    /// Constructs a new Logger from a [`LoggerConfig`].
    ///
    /// ### Parameters
    /// + `config`: The logger configuration to use.
    ///
    /// [`LoggerConfig`]: struct.LoggerConfig.html
    pub fn from_config(config: LoggerConfig) -> Self {
        Logger::new().configure(config)
    }

    /// Constructs a new Logger with an output formatter.
    ///
    /// ### Parameters
    /// + `formatter`: The formatter function.
    fn new_with_formatter<F>(formatter: F) -> Self
        where
            F: Fn(
                fern::FormatCallback<'_>,
                &fmt::Arguments<'_>,
                &Record<'_>)
            + Sync + Send + 'static,
    {
        let dispatch = fern::Dispatch::new().format(formatter);
        Self { dispatch }
    }
    
    /// Constructs a new Logger from a [`LoggerConfig`] and an output formatter.
    ///
    /// ### Parameters
    /// + `config`: The logger configuration to use.
    /// + `formatter`: The formatter function.
    ///
    /// [`LoggerConfig`]: struct.LoggerConfig.html
    pub fn from_config_formatter<F>(config: LoggerConfig, formatter: F) -> Self
        where
            F: Fn(
                fern::FormatCallback<'_>,
                &fmt::Arguments<'_>,
                &Record<'_>)
            + Sync + Send + 'static,
    {
        Logger::new_with_formatter(formatter).configure(config)
    }

    ////////////////////////////////////////////////////////////////////////////
    // Methods
    ////////////////////////////////////////////////////////////////////////////

    /// Constructs a new logger based on a [`LoggerConfig`] and the Logger it
    /// will be added to.
    fn configure(mut self, mut config: LoggerConfig) -> Self {
        if config.allow_env_override {
            env_var_override(&mut config);
        }

        self.dispatch = self.dispatch.level(config.level_filter);

        match config.stdout_log_output {
            StdoutLogOutput::Plain => {
                self.dispatch = self.dispatch.chain(io::stdout())
            },

            StdoutLogOutput::Colored => {
                self.dispatch = self
                    .dispatch
                    .chain(colored_stdout(fern::colors::ColoredLevelConfig {
                        error: Color::BrightRed,
                        warn: Color::Yellow,
                        debug: Color::White,
                        info: Color::Green,
                        trace: Color::Cyan,
                    }))
            },
            
            _ => ()
        }

        if let Some(path) = config.log_path {
            if let Ok(log_path) = fern::log_file(path) {
                self.dispatch = self.dispatch.chain(log_path)
            } else {
                eprintln!("Unable to access the log file, as such it will not \
                    be used")
            }
        }

        self
    }

    /// Sets the log level for a module.
    ///
    /// ### Parameters
    /// + `module`: The name of the module.
    /// + `level`: The [`LevelFilter`] to set.
    ///
    /// [`LevelFilter`]: https://docs.rs/log/0.4.10/log/enum.LevelFilter.html
    pub fn level_for<T: Into<std::borrow::Cow<'static, str>>>(
        mut self,
        module: T,
        level: LevelFilter) 
        -> Self
    {
        self.dispatch = self.dispatch.level_for(module, level);
        self
    }

    /// Starts the `Logger`, enabling the use of [`log macros`].
    ///
    /// [`log macros`]: https://docs.rs/log/0.4.10/log/#macros
    pub fn start(self) {
        self.dispatch.apply().unwrap_or_else(|_|
            warn!("Logger already set, SUNFLOWER logger will not be used")
        );
    }
}


////////////////////////////////////////////////////////////////////////////////
// env_var_override
////////////////////////////////////////////////////////////////////////////////
/// Overrides [`LoggerConfig`] settings by reading environment variables.
///
/// ### Parameters
/// + `LoggerConfig`: The logger configuration to override.
///
/// [`LoggerConfig`]: struct.LoggerConfig.html
fn env_var_override(config: &mut LoggerConfig) {
    if let Ok(var) = env::var("SUNFLOWER_LOG_STDOUT") {
        match var.to_lowercase().as_ref() {
            "off" | "no" | "0" 
                => config.stdout_log_output = StdoutLogOutput::Off,
            "plain" | "yes" | "1" 
                => config.stdout_log_output = StdoutLogOutput::Plain,
            "colored" | "2" 
                => config.stdout_log_output = StdoutLogOutput::Colored,
            _ => {}
        }
    }

    if let Ok(var) = env::var("SUNFLOWER_LOG_LEVEL_FILTER") {
        if let Ok(lf) = LevelFilter::from_str(&var) {
            config.level_filter = lf;
        }
    }
    
    if let Ok(path) = env::var("SUNFLOWER_LOG_FILE_PATH") {
        config.log_path = Some(PathBuf::from(path));
    }
}


////////////////////////////////////////////////////////////////////////////////
// colored_stdout
////////////////////////////////////////////////////////////////////////////////
/// Adds color to log output to stdout.
///
/// ### Parameters
/// + `color_config`: The [`ColoredLevelConfig`] specifying output colors.
///
/// [`ColoredLevelConfig`]: https://docs.rs/fern/0.5.9/fern/colors/struct.ColoredLevelConfig.html
fn colored_stdout(color_config: fern::colors::ColoredLevelConfig)
    -> fern::Dispatch
{
    fern::Dispatch::new()
        .chain(io::stdout())
        .format(move |out, message, record| {
            let color = color_config.get_color(&record.level());
            out.finish(format_args!(
                "{color}{message}{color_reset}",
                color = format!("\x1B[{}m", color.to_fg_str()),
                message = message,
                color_reset = "\x1B[0m",
            ))
        })
}
