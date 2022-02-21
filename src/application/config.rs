////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Runtime application configuration.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::application::LoadStatus;
use crate::application::TraceConfig;

// External library imports.
use anyhow::Context as _;
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;
use tracing::event;
use tracing::Level;

// Standard library imports.
use std::convert::TryInto as _;
use std::fs::File;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::SeekFrom;
use std::io::BufRead as _;
use std::io::Seek as _;
use std::io::BufReader;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;
use std::path::PathBuf;


////////////////////////////////////////////////////////////////////////////////
// ConfigFormat
////////////////////////////////////////////////////////////////////////////////
/// The storage format to use for the stall configuration.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[derive(clap::ArgEnum)]
pub enum ConfigFormat {
    /// A stall file serialized as a UTF-8 list of files.
    List,
    /// A stall file serialized as RON.
    Ron,
}

impl FromStr for ConfigFormat {
    type Err = ConfigFormatParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("list") {
            Ok(ConfigFormat::List)
        } else if s.eq_ignore_ascii_case("ron") {
            Ok(ConfigFormat::Ron)
        } else {
            Err(ConfigFormatParseError)
        }
    }
}

/// An error indicating a failure to parse a [`ConfigFormat`].
///
/// [`ConfigFormat`]: ConfigFormat 
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigFormatParseError;

impl std::error::Error for ConfigFormatParseError {}

impl std::fmt::Display for ConfigFormatParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failure to parse ConfigFormat")
    }
}


////////////////////////////////////////////////////////////////////////////////
// Config
////////////////////////////////////////////////////////////////////////////////
/// Application configuration data. Configures the logger, window, renderer,
/// application limits, and behaviors.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The Config file's load status.
    #[serde(skip)]
    load_status: LoadStatus,

    /// The trace configuration.
    #[serde(default = "Config::default_trace_config")]
    pub trace_config: TraceConfig,


    /// The list of files to apply stall commands to.
    pub files: Vec<PathBuf>,
}


impl Default for Config {
    fn default() -> Self {
        Self {
            load_status: LoadStatus::default(),
            trace_config: Self::default_trace_config(),
            files: Vec::new(),
        }
    }
}

impl Config {
    /// The default path to look for the [`Config`] file, relative to the app root.
    ///
    /// [`Config`]: crate::application::Config
    pub const DEFAULT_CONFIG_PATH: &'static str = ".stall";

    /// Constructs a new `Config` with the default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    ////////////////////////////////////////////////////////////////////////////
    // File and serialization methods.
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the given `Config` with the given load path.
    #[must_use]
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.set_load_path(path);
        self
    }

    /// Returns the `Config`'s load path.
    #[must_use]
    pub fn load_path(&self) -> Option<&Path> {
        self.load_status.load_path()
    }

    /// Sets the `Config`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_status.set_load_path(path);
    }

    /// Returns true if the Config was modified.
    #[must_use]
    pub const fn modified(&self) -> bool {
        self.load_status.modified()
    }

    /// Sets the Config modification flag.
    pub fn set_modified(&mut self, modified: bool) {
        self.load_status.set_modified(modified);
    }

    /// Constructs a new `Config` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!(
                "Failed to open config file for reading: {}",
                path.display()))?;
        let mut config = Self::read_from_file(file)?;
        config.set_load_path(path);
        Ok(config)
    }

    /// Open a file at the given path and write the `Config` into it.
    pub fn write_to_path<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .with_context(|| format!(
                "Failed to create/open config file for writing: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write config file")?;
        Ok(())
    }
    
    /// Create a new file at the given path and write the `Config` into it.
    pub fn write_to_path_if_new<P>(&self, path: P) -> Result<(), Error>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create_new(true)
            .open(path)
            .with_context(|| format!(
                "Failed to create config file: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write config file")?;
        Ok(())
    }

    /// Write the `Config` into the file is was loaded from. Returns true if the
    /// data was written.
    pub fn write_to_load_path(&self) -> Result<bool, Error> {
        match self.load_status.load_path() {
            Some(path) => {
                self.write_to_path(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Write the `Config` into a new file using the load path. Returns true
    /// if the data was written.
    pub fn write_to_load_path_if_new(&self) -> Result<bool, Error> {
        match self.load_status.load_path() {
            Some(path) => {
                self.write_to_path_if_new(path)?;
                Ok(true)
            },
            None => Ok(false)    
        }
    }

    /// Constructs a new `Config` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
        match Config::parse_ron_from_file(&mut file) {
            Ok(config) => Ok(config),
            Err(e)     => {
                event!(Level::DEBUG, "Error in RON, switching to list format.\n\
                    {:?}", e);
                let _ = file.seek(SeekFrom::Start(0))?;
                Config::parse_list_from_file(&mut file)
            },
        }
    }

    /// Parses a `Config` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .context("Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len.try_into()?);
        let _ = file.read_to_end(&mut buf)
            .context("Failed to read config file")?;

        Self::parse_ron_from_bytes(&buf[..])
    }


    /// Parses a `Config` from a file using a newline-delimited file list
    /// format.
    fn parse_list_from_file(file: &mut File) -> Result<Self, Error> {
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
            config.files.push(path);
        }

        Ok(config) 
    }

    /// Parses a `Config` from a buffer using the RON format.
    fn parse_ron_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(bytes)
            .context("Failed deserializing RON file")?;
        let config = Self::deserialize(&mut d)
            .context("Failed parsing RON file")?;
        d.end()
            .context("Failed parsing RON file")?;

        Ok(config) 
    }

    /// Write the `Config` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Config` from a file using the RON format.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
        tracing::debug!("Serializing & writing Config file.");
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true)
            .extensions(ron::extensions::Extensions::IMPLICIT_SOME);
        let s = ron::ser::to_string_pretty(&self, pretty)
            .context("Failed to serialize RON file")?;
        let mut writer = BufWriter::new(file);
        writer.write_all(s.as_bytes())
            .context("Failed to write RON file")?;
        writer.flush()
            .context("Failed to flush file buffer")
    }

    ////////////////////////////////////////////////////////////////////////////
    // Default constructors for serde.
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the default [`TraceConfig`].
    ///
    /// [`TraceConfig`]: crate::application::TraceConfig
    fn default_trace_config() -> TraceConfig {
        TraceConfig::default()
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, "\ttrace_config.trace_output_path: {:?}",
            self.trace_config.trace_output_path)?;
        writeln!(fmt, "\ttrace_config.ansi_colors: {:?}",
            self.trace_config.ansi_colors)?;
        writeln!(fmt, "\ttrace_config.output_stdout: {:?}",
            self.trace_config.output_stdout)?;
        writeln!(fmt, "\ttrace_config.filters:")?;
        for filter in &self.trace_config.filters {
            writeln!(fmt, "\t\t{:?}", filter)?;
        }

        Ok(())
    }
}
