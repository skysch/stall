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
use bimap::BiHashMap;

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

    
    /// The stall file entries. (Left = Local, Right = Remote)
    entries: BiHashMap<PathBuf, PathBuf>,


    // TODO: Auto-rename on add
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
            entries: BiHashMap::new(),
        }
    }

    /// Returns `true` if there are no entries in the stall.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns an iterator over the entries in the stall.
    pub fn entries(&self) -> impl Iterator<Item=Entry> {
        self.entries
            .iter()
            .map(|(l, r)| Entry {
                local: l.as_path(),
                remote: r.as_path(),
            })
    }

    /// Inserts a new stall entry from a list file parse. Doesn't update the
    /// load_status of the Stall.
    fn insert_list_remote(&mut self, remote: PathBuf) {
        // TODO: Generate the local entry properly.
        let _ = self.entries.insert(remote.clone(), remote);
    }

    /// Adds a new entry to the stall with the given local and remote paths.
    pub fn insert(&mut self, local: PathBuf, remote: PathBuf) {
        event!(Level::INFO, "Adding local: {} remote: {}",
            local.display(),
            remote.display());
        self.load_status.set_modified(true);
        let overwrite = self.entries.insert(local, remote);
        event!(Level::DEBUG, "Overwrite: {:?}", overwrite);
    }

    /// Removes an entry from the stall with the given local path, if one
    /// exists.
    pub fn remove_local(&mut self, local: &Path)
        -> Option<(PathBuf, PathBuf)>
    {
        event!(Level::INFO, "Removing local: {}", local.display());
        self.load_status.set_modified(true);
        let removed = self.entries.remove_by_left(local);
        event!(Level::DEBUG, "Removed: {:?}", removed);
        removed
    }

    /// Removes an entry from the stall with the given remote path, if one
    /// exists.
    pub fn remove_remote(&mut self, remote: &Path)
        -> Option<(PathBuf, PathBuf)>
    {
        event!(Level::INFO, "Removing remote: {}", remote.display());
        self.load_status.set_modified(true);
        let removed = self.entries.remove_by_right(remote);
        event!(Level::DEBUG, "Removed: {:?}", removed);
        removed
    }


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
                "Failed to open stall file for writing: {}",
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
        // TODO: Consider returning RON error.
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
            stall.insert_list_remote(path);
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
/// A stall file entry view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
    pub local: &'a Path,
    pub remote: &'a Path,
}
