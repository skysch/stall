////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! User preferences.
////////////////////////////////////////////////////////////////////////////////


// Internal library imports.
use crate::application::LoadStatus;

// External library imports.
use anyhow::Context as _;
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;

// Standard library imports.
use std::convert::TryInto as _;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Read as _;
use std::io::Write as _;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Prefs
////////////////////////////////////////////////////////////////////////////////
/// User preferences.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefs {
	/// The Prefs file's load status.
	#[serde(skip)]
	load_status: LoadStatus,
}

impl Default for Prefs {
	fn default() -> Self {
		Self::new()
	}
}

impl Prefs {
	/// Constructs a new `Prefs` with the default options.
	#[must_use]
	pub fn new() -> Self {
		Self {
			load_status: LoadStatus::default(),
		}
	}

	////////////////////////////////////////////////////////////////////////////
	// File and serialization methods.
	////////////////////////////////////////////////////////////////////////////
	
	/// Returns the given `Prefs` with the given load path.
	#[must_use]
	pub fn with_load_path<P>(mut self, path: P) -> Self
		where P: AsRef<Path>
	{
		self.set_load_path(path);
		self
	}

	/// Returns the `Prefs`'s load path.
	#[must_use]
	pub fn load_path(&self) -> Option<&Path> {
		self.load_status.load_path()
	}

	/// Sets the `Prefs`'s load path.
	pub fn set_load_path<P>(&mut self, path: P)
		where P: AsRef<Path>
	{
		self.load_status.set_load_path(path);
	}

	/// Returns true if the Prefs was modified.
	#[must_use]
	pub const fn modified(&self) -> bool {
		self.load_status.modified()
	}

	/// Sets the Prefs modification flag.
	pub fn set_modified(&mut self, modified: bool) {
		self.load_status.set_modified(modified);
	}

	/// Constructs a new `Prefs` with options read from the given file path.
	#[tracing::instrument(skip_all, err)]
	pub fn read_from_path<P>(path: P) -> Result<Self, Error> 
		where P: AsRef<Path>
	{
		let path = path.as_ref();
		let file = File::open(path)
			.with_context(|| format!(
				"Failed to open prefs file for reading: {}",
				path.display()))?;
		let mut prefs = Self::read_from_file(file)?;
		prefs.set_load_path(path);
		Ok(prefs)
	}

	/// Open a file at the given path and write the `Prefs` into it.
	#[tracing::instrument(skip_all, err)]
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
				"Failed to create/open prefs file for writing: {}",
				path.display()))?;
		self.write_to_file(file)
			.context("Failed to write prefs file")?;
		Ok(())
	}
	
	/// Create a new file at the given path and write the `Prefs` into it.
	#[tracing::instrument(skip_all, err)]
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
				"Failed to create prefs file: {}",
				path.display()))?;
		self.write_to_file(file)
			.context("Failed to write prefs file")?;
		Ok(())
	}

	/// Write the `Prefs` into the file is was loaded from. Returns true if the
	/// data was written.
	#[tracing::instrument(skip_all, err)]
	pub fn write_to_load_path(&self) -> Result<bool, Error> {
		match self.load_status.load_path() {
			Some(path) => {
				self.write_to_path(path)?;
				Ok(true)
			},
			None => Ok(false)    
		}
	}

	/// Write the `Prefs` into a new file using the load path. Returns true
	/// if the data was written.
	#[tracing::instrument(skip_all, err)]
	pub fn write_to_load_path_if_new(&self) -> Result<bool, Error> {
		match self.load_status.load_path() {
			Some(path) => {
				self.write_to_path_if_new(path)?;
				Ok(true)
			},
			None => Ok(false)    
		}
	}

	/// Constructs a new `Prefs` with options parsed from the given file.
	#[tracing::instrument(skip_all, err)]
	pub fn read_from_file(mut file: File) -> Result<Self, Error>  {
		Self::parse_ron_from_file(&mut file)
	}

	/// Parses a `Prefs` from a file using the RON format.
	#[tracing::instrument(skip_all, err)]
	fn parse_ron_from_file(file: &mut File) -> Result<Self, Error> {
		let len = file.metadata()
			.context("Failed to recover file metadata.")?
			.len();
		let mut buf = Vec::with_capacity(len.try_into()?);
		let _ = file.read_to_end(&mut buf)
			.context("Failed to read prefs file")?;

		Self::parse_ron_from_bytes(&buf[..])
	}

	/// Parses a `Prefs` from a buffer using the RON format.
	#[tracing::instrument(skip_all, err)]
	fn parse_ron_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
		use ron::de::Deserializer;
		let mut d = Deserializer::from_bytes(bytes)
			.context("Failed deserializing RON file")?;
		let prefs = Self::deserialize(&mut d)
			.context("Failed parsing RON file")?;
		d.end()
			.context("Failed parsing RON file")?;

		Ok(prefs)
	}

	/// Write the `Prefs` into the given file.
	#[tracing::instrument(skip_all, err)]
	pub fn write_to_file(&self, mut file: File) -> Result<(), Error> {
		self.generate_ron_into_file(&mut file)
	}

	/// Parses a `Prefs` from a file using the RON format.
	#[tracing::instrument(skip_all, err)]
	fn generate_ron_into_file(&self, file: &mut File) -> Result<(), Error> {
		tracing::debug!("Serializing & writing Prefs file.");
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

}

impl std::fmt::Display for Prefs {
	fn fmt(&self, _fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Ok(())   
	}
}

