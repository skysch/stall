////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stall file entry.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::application::LoadStatus;

// External library imports.
use serde::Deserialize;
use serde::Serialize;
use anyhow::Context as _;
use anyhow::Error;
use tracing::event;
use tracing::Level;

// Standard library imports.
use std::convert::TryInto as _;
use std::fs::File;
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
use std::str::FromStr;



////////////////////////////////////////////////////////////////////////////////
// Stall
////////////////////////////////////////////////////////////////////////////////
/// A stall file entry database.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stall {
    /// The stall file's load status.
    #[serde(skip)]
    load_status: LoadStatus,

    entries: Vec<Entry>,
}

impl Default for Stall {
    fn default() -> Self {
        Stall::new()
    }
}

impl Stall {
    /// Constructs a new `Stall` with no entries.
    pub fn new() -> Self {
        Stall {
            load_status: LoadStatus::default(),
            entries: Vec::new(),
        }
    }

    /// Returns `true` if there are no entries in the stall.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over the entries in the stall.
    pub fn entries(&self) -> impl Iterator<Item=&Entry> {
        self.entries.iter()
    }

    // pub fn add_entry(&mut self, entry: Entry) {}
    // pub fn remove_entry_local(&mut self, name: &Path) -> Entry {}
    // pub fn remove_entry_remote(&mut self, name: &Path) -> Entry {}
    // pub fn move_entry(&mut self, name: &Path) {}
    // pub fn entry_mut_local(&mut self, name: &Path) -> &mut Entry {}
    // pub fn entry_mut_remote(&mut self, name: &Path) -> &mut Entry {}


    ////////////////////////////////////////////////////////////////////////////
    // File and serialization methods.
    ////////////////////////////////////////////////////////////////////////////

    /// Returns the given `Stall` with the given load path.
    #[must_use]
    pub fn with_load_path<P>(mut self, path: P) -> Self
        where P: AsRef<Path>
    {
        self.set_load_path(path);
        self
    }

    /// Returns the `Stall`'s load path.
    #[must_use]
    pub fn load_path(&self) -> Option<&Path> {
        self.load_status.load_path()
    }

    /// Sets the `Stall`'s load path.
    pub fn set_load_path<P>(&mut self, path: P)
        where P: AsRef<Path>
    {
        self.load_status.set_load_path(path);
    }

    /// Returns true if the Stall was modified.
    #[must_use]
    pub const fn modified(&self) -> bool {
        self.load_status.modified()
    }

    /// Sets the Stall modification flag.
    pub fn set_modified(&mut self, modified: bool) {
        self.load_status.set_modified(modified);
    }

    /// Constructs a new `Stall` with options read from the given file path.
    pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = File::open(path)
            .with_context(|| format!(
                "Failed to open stall file for reading: {}",
                path.display()))?;
        let mut stall = Self::read_from_file(file)?;
        stall.set_load_path(path);
        Ok(stall)
    }

    /// Open a file at the given path and write the `Stall` into it.
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
                "Failed to create/open stall file for writing: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write stall file")?;
        Ok(())
    }
    
    /// Create a new file at the given path and write the `Stall` into it.
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
                "Failed to create stall file: {}",
                path.display()))?;
        self.write_to_file(file)
            .context("Failed to write stall file")?;
        Ok(())
    }

    /// Write the `Stall` into the file is was loaded from. Returns true if the
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

    /// Write the `Stall` into a new file using the load path. Returns true
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

    /// Constructs a new `Stall` with options parsed from the given file.
    pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
        match Self::parse_ron_from_file(&mut file) {
            Ok(stall) => Ok(stall),
            Err(e)     => {
                event!(Level::DEBUG, "Error in RON, switching to list format.\n\
                    {:?}", e);
                let _ = file.seek(SeekFrom::Start(0))?;
                Self::parse_list_from_file(&mut file)
            },
        }
    }

    /// Parses a `Stall` from a file using the RON format.
    fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
        let len = file.metadata()
            .context("Failed to recover file metadata.")?
            .len();
        let mut buf = Vec::with_capacity(len.try_into()?);
        let _ = file.read_to_end(&mut buf)
            .context("Failed to read stall file")?;

        Self::parse_ron_from_bytes(&buf[..])
    }


    /// Parses a `Stall` from a file using a newline-delimited file list
    /// format.
    fn parse_list_from_file(file: &mut File) -> Result<Self, Error> {
        let mut stall = Stall::default();
        let buf_reader = BufReader::new(file);
        for line in buf_reader.lines() {
            let line = line
                .with_context(|| "Failed to read stall file")?;
            
            // Skip empty lines.
            let line = line.trim();
            if line.is_empty() { continue }

            // Skip comment lines.
            if line.starts_with("//") { continue }
            if line.starts_with("#") { continue }

            let path: PathBuf = line.into();
            stall.entries.push(Entry::from_remote(path));
        }

        Ok(stall) 
    }

    /// Parses a `Stall` from a buffer using the RON format.
    fn parse_ron_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        use ron::de::Deserializer;
        let mut d = Deserializer::from_bytes(bytes)
            .context("Failed deserializing RON file")?;
        let stall = Self::deserialize(&mut d)
            .context("Failed parsing RON file")?;
        d.end()
            .context("Failed parsing RON file")?;

        Ok(stall) 
    }

    /// Write the `Stall` into the given file.
    pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
        self.generate_ron_into_file(&mut file)
    }

    /// Parses a `Stall` from a file using the RON format.
    fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
        tracing::debug!("Serializing & writing Stall file.");
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

}


////////////////////////////////////////////////////////////////////////////////
// Entry
////////////////////////////////////////////////////////////////////////////////
/// A stall file entry
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Entry {
    remote: PathBuf,
    local: PathBuf,
}


impl Entry {
    pub fn new(remote: PathBuf, local: PathBuf) -> Self {
        Entry {
            remote,
            local,
        }
    }

    pub fn from_remote(remote: PathBuf) -> Self {
        Entry {
            local: remote.clone(),
            remote,
        }
    }

    /// Returns the stall entry's associated remote path.
    pub fn remote_path(&self) -> &Path {
        self.remote.as_path()
    }

    /// Returns the stall entry's local path within the stall.
    pub fn local_path(&self) -> &Path {
        self.local.as_path()
    }
}
