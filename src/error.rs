////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Error types.
////////////////////////////////////////////////////////////////////////////////


pub use anyhow::Error;
pub use anyhow::Context;

#[derive(Debug, Clone)]
pub struct InvalidFile;

impl std::error::Error for InvalidFile {}

impl std::fmt::Display for InvalidFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>)
		-> Result<(), std::fmt::Error> 
	{
		write!(f, "Invalid file.")
	}
}
