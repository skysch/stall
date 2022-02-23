////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licensed using the MIT or Apache 2 license.
// See license-mit.md and license-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! File source and modification tracking.
////////////////////////////////////////////////////////////////////////////////


// Standard library imports.
use std::path::Path;
use std::path::PathBuf;



////////////////////////////////////////////////////////////////////////////////
// LoadStatus
////////////////////////////////////////////////////////////////////////////////
/// Structure for tracking a file's load status.
#[derive(Debug, Clone)]
pub struct LoadStatus {
	/// The path the data was initially loaded from.
	load_path: Option<PathBuf>,
	/// Whether the data has been modified since last save.
	modified: bool,
}

impl Default for LoadStatus {
	fn default() -> Self {
		Self::new()   
	}
}

impl LoadStatus {
	/// Constructs a new `LoadStatus`.
	#[must_use]
	pub const fn new() -> Self {
		Self {
			load_path: None,
			modified: false,
		}
	}

	/// Returns the `LoadStatus` with the given load path.
	#[must_use]
	pub fn with_load_path<P>(mut self, path: P) -> Self
		where P: AsRef<Path>
	{
		self.set_load_path(path);
		self
	}

	/// Returns the `LoadStatus` with the given data modification flag.
	#[must_use]
	pub fn with_modified(mut self, modified: bool) -> Self
	{
		self.set_modified(modified);
		self
	}

	/// Returns the `LoadStatus`'s load path.
	#[must_use]
	pub fn load_path(&self) -> Option<&Path> {
		self.load_path.as_ref().map(AsRef::as_ref)
	}

	/// Sets the `LoadStatus`'s load path.
	pub fn set_load_path<P>(&mut self, path: P)
		where P: AsRef<Path>
	{
		self.load_path = Some(path.as_ref().to_owned());
	}

	/// Clears the `LoadStatus`'s load path.
	pub fn clear_load_path<P>(&mut self) {
		self.load_path = None;
	}

	/// Returns true if the data was modified.
	#[must_use]
	pub const fn modified(&self) -> bool {
		self.modified
	}

	/// Sets the data modification flag.
	pub fn set_modified(&mut self, modified: bool) {
		self.modified = modified;
	}
}
