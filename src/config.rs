////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! The application configuration, or 'stall file.'
////////////////////////////////////////////////////////////////////////////////
#![warn(missing_docs)]

// Local imports.
use crate::logger::LevelFilter;
use crate::logger::LoggerConfig;
use crate::logger::StdoutLogOutput;
use crate::error::Error;
use crate::error::Context;

// External library imports.
use serde::Deserialize;
use serde::Serialize;

use log::*;

// Standard library imports.
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::PathBuf;
use std::borrow::Cow;
use std::collections::BTreeMap;


////////////////////////////////////////////////////////////////////////////////
// DEFAULT_CONFIG_PATH
////////////////////////////////////////////////////////////////////////////////
/// The default path to look for the [`Config`] file, relative to the app root.
///
/// [`Config`]: struct.Config.html
pub const DEFAULT_CONFIG_PATH: &'static str = ".stall";

////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration data (stall file). Configures the logger and
/// defines files.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The logger configuration.
    #[serde(default = "Config::default_logger_config")]
    pub logger_config: LoggerConfig,

    /// Module specific log levels.
    #[serde(default = "Config::default_log_levels")]
    pub log_levels: BTreeMap<Cow<'static, str>, LevelFilter>,

    /// The list of files to apply stall commands to.
    pub files: Vec<Box<Path>>,
}


impl Config {
    /// Constructs a new `Config` with the default options.
    pub fn new() -> Self {
        Config::default()
    }

    /// Constructs a new `Config` with options read from the given file path.
    pub fn from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let file = File::open(path)
            .with_context(|| "Failed to open config file.")?;
        Config::from_file(file)
    }

    /// Constructs a new `Config` with options parsed from the given file.
    fn from_file(mut file: File) -> Result<Self, Error>  {
        match Config::parse_ron_file(&mut file) {
            Ok(config) => Ok(config),
            Err(e)     => {
                debug!("Error in RON, switching to list format.\n{:?}", e);
                file.seek(SeekFrom::Start(0))?;
                Config::parse_list_file(&mut file)
            },
        }
    }

    /// Parses a `Config` from a file using the RON format.
    fn parse_ron_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .with_context(|| "Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len as usize);
        file.read_to_end(&mut buf)
            .with_context(|| "Failed to read config file")?;

        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(&buf)
            .with_context(|| "Failed deserializing RON file")?;
        let config = Config::deserialize(&mut d)
            .with_context(|| "Failed parsing Ron file")?;
        d.end()
            .with_context(|| "Failed parsing Ron file")?;

        Ok(config) 
    }
    
    /// Parses a `Config` from a file using a newline-delimited file list
    /// format.
    fn parse_list_file(file: &mut File) -> Result<Self, Error> {
        let mut config = Config::default();
        let buf_reader = BufReader::new(file);
        for line in buf_reader.lines() {
            let line = line
                .with_context(|| "Failed to read config file")?;
            
            // Skip empty lines.
            let line = line.trim();
            if line.is_empty() { continue }

            // Skip comment lines.
            if line.starts_with("//") { continue }
            if line.starts_with("#") { continue }

            let path: PathBuf = line.into();
            config.files.push(path.into());
        }

        Ok(config) 
    }

    /// Normalizes paths in the config by expanding them relative to the given
    /// root path.
    pub fn normalize_paths(&mut self, base: &PathBuf) {
        match self.logger_config.log_path {
            Some(ref log_path) if log_path.is_relative() => {
                let log_path = base.clone().join(log_path);
                // Relative log file paths are relative to base.
                self.logger_config.log_path = Some(log_path);
            },
            _ => (),
        }
    }

    /// Returns the default [`LoggerConfig`].
    ///
    /// [`LoggerConfig`]: ../logger/struct.LoggerConfig.html
    #[inline(always)]
    fn default_logger_config() -> LoggerConfig {
        LoggerConfig {
            stdout_log_output: StdoutLogOutput::Colored,
            .. Default::default()
        }
    }

    /// Returns the default log levels for modules.
    #[inline(always)]
    fn default_log_levels() -> BTreeMap<Cow<'static, str>, LevelFilter> {
        Default::default()
    }

}

impl Default for Config {
    fn default() -> Self {
        Config {
            logger_config: Config::default_logger_config(),
            log_levels: Config::default_log_levels(),
            files: Vec::new(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\n\tlogger_config/stdout_log_output: {:?}",
            self.logger_config.stdout_log_output)?;
        writeln!(fmt, "\tlogger_config/level_filter: {:?}",
            self.logger_config.level_filter)?;
        writeln!(fmt, "\tfiles: {:?}", self.files)
    }
}
