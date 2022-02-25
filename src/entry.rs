////////////////////////////////////////////////////////////////////////////////
// Stall configuration management utility
////////////////////////////////////////////////////////////////////////////////
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Stall file entry.
////////////////////////////////////////////////////////////////////////////////

// Internal library imports.
use crate::command::CommonOptions;

// External library imports.
use anyhow::Error;
use anyhow::anyhow;
use colored::Colorize as _;
use fcmp::FileCmp;
use tracing::event;
use tracing::span;
use tracing::Level;

// Standard library imports.
use std::io::Write;
use std::path::Path;


////////////////////////////////////////////////////////////////////////////////
// Entry
////////////////////////////////////////////////////////////////////////////////
/// A stall file entry view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry<'a> {
	/// The local path of a stall entry.
	pub local: &'a Path,
	/// The remote path of a stall entry.
	pub remote: &'a Path,
}


impl<'a> Entry<'a> {
	/// Returns the file statuses for the local and remote entry paths.
	pub fn status(&self, stall_dir: &Path) -> (Status, Status) {
		use Status::*;
		use std::cmp::Ordering::*;

		let mut full_local = stall_dir.to_path_buf();
		full_local.push(self.local);

		let file_cmp_l = FileCmp::try_from(full_local.as_path())
			.map_err(|e| event!(Level::DEBUG, "{e}: {:?}",
				full_local.as_path()));
		let file_cmp_r = FileCmp::try_from(self.remote)
			.map_err(|e| event!(Level::DEBUG, "{e}: {:?}", self.remote));

		event!(Level::TRACE, "LOCAL {:?}", file_cmp_l);
		event!(Level::TRACE, "REMOTE {:?}", file_cmp_r);

		match (file_cmp_l, file_cmp_r) {
			(Err(_), Err(_))                 => (Error, Error),
			(Err(_), Ok(f)) if !f.is_found() => (Error, Absent),
			(Err(_), Ok(_))                  => (Error, Exists),
			(Ok(f), Err(_)) if !f.is_found() => (Absent, Error),
			(Ok(_), Err(_))                  => (Exists, Error),

			(Ok(l), Ok(r)) => match (l.is_found(), r.is_found()) {
				(false, false) => (Absent, Absent),
				(true, false)  => (Exists, Absent),
				(false, true)  => (Absent, Exists),
				(true, true) => match l.partial_cmp(&r, false) {
					Some(Less)    => (Older, Newer),
					Some(Equal)   => (Same,  Same),
					Some(Greater) => (Newer, Older),
					None          => (Error, Error),
				},
			}
		}
	}

	/// Prints the status of the stall entry and copies the remote file into the
	/// stall directory.
	pub fn collect(
		&self,
		out: &mut dyn Write,
		stall_dir: &Path,
		force: bool,
		dry_run: bool,
		common: &CommonOptions)
		-> Result<(), Error>
	{
		use Status::*;

		let (status_l, status_r) = self.status(stall_dir);
		let action = match (&status_l, &status_r) {
			(Absent, Exists) |
			(Older,  Newer)  => Action::Copy,

			(Same,   Same)  if force => Action::Force,
			(Newer,  Older) if force => Action::Force,

			(_, Error) |
			(Error, _) => Action::Stop,

			_ => Action::Skip,
		};

		if !common.quiet {
			self.write_status_action(out, status_l, status_r, action, common)?;
		}
		if common.promote_warnings_to_errors && matches!(action, Action::Stop) {
			return Err(anyhow!("abort collect due to file error"));
		}

		if matches!(action, Action::Force | Action::Copy) {
			let mut full_local = stall_dir.to_path_buf();
			full_local.push(self.local);

			let copy_method = if dry_run {
				CopyMethod::None
			} else {
				CopyMethod::Subprocess
			};

			copy(self.remote, full_local.as_path(), copy_method)?;
		}

		Ok(())
	}

	/// Prints the status of the stall entry and copies the stalled file into
	/// the remote directory.
	pub fn distribute(
		&self,
		out: &mut dyn Write,
		stall_dir: &Path,
		force: bool,
		dry_run: bool,
		common: &CommonOptions)
		-> Result<(), Error>
	{
		use Status::*;

		let (status_l, status_r) = self.status(stall_dir);
		let action = match (&status_l, &status_r) {
			(Exists, Absent) |
			(Newer,  Older)  => Action::Copy,

			(Same,   Same)  if force => Action::Force,
			(Older,  Newer) if force => Action::Force,

			(_, Error) |
			(Error, _) => Action::Stop,

			_ => Action::Skip,
		};

		if !common.quiet {
			self.write_status_action(out, status_l, status_r, action, common)?;
		}
		if common.promote_warnings_to_errors && matches!(action, Action::Stop) {
			return Err(anyhow!("abort collect due to file error"));
		}

		if matches!(action, Action::Force | Action::Copy) {
			let mut full_local = stall_dir.to_path_buf();
			full_local.push(self.local);

			let copy_method = if dry_run {
				CopyMethod::None
			} else {
				CopyMethod::Subprocess
			};

			copy(full_local.as_path(), self.remote, copy_method)?;
		}

		Ok(())

	}

	pub(in crate) fn write_status_header(
		out: &mut dyn Write,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		if common.color.enabled() {
			writeln!(out, "    {:<6} {:<6} {}", 
				"LOCAL".bright_white().bold(),
				"REMOTE".bright_white().bold(),
				"FILE".bright_white().bold())
		} else {
			writeln!(out, "    {:<6} {:<6} {}", 
				"LOCAL",
				"REMOTE",
				"FILE")
		}
	}

	pub(in crate) fn write_status_action_header(
		out: &mut dyn Write,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		if common.color.enabled() {
			writeln!(out, "    {:<6} {:<6} {:<6} {}", 
				"LOCAL".bright_white().bold(),
				"REMOTE".bright_white().bold(),
				"ACTION".bright_white().bold(),
				"FILE".bright_white().bold())
		} else {
			writeln!(out, "    {:<6} {:<6} {:<6} {}", 
				"LOCAL",
				"REMOTE",
				"ACTION",
				"FILE")
		}
	}

	pub(in crate) fn write_status(
		&self,
		out: &mut dyn Write,
		status_l: Status,
		status_r: Status,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		write!(out, "    ")?;
		status_l.write(out, common)?;
		write!(out, " ")?;
		status_r.write(out, common)?;
		write!(out, " ")?;
		self.write_path(out, common)?;
		writeln!(out)
	}

	pub(in crate) fn write_status_action(
		&self,
		out: &mut dyn Write,
		status_l: Status,
		status_r: Status,
		action: Action,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		write!(out, "    ")?;
		status_l.write(out, common)?;
		write!(out, " ")?;
		status_r.write(out, common)?;
		write!(out, " ")?;
		action.write(out, common)?;
		write!(out, " ")?;
		self.write_path(out, common)?;
		writeln!(out)
	}

	fn write_path(
		&self,
		out: &mut dyn Write,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		write!(out, "{}", self.local.display())?;
		
		if common.short_names {
			// Check if file is renamed.
			let remote_name: &Path = self.remote.file_name()
				.expect("get remote file name")
				.as_ref();

			if self.local != remote_name {
				write!(out, " ({})", remote_name.display())?;
			}
		} else {
			write!(out, " ({})", self.remote.display())?;
		}

		Ok(())
	}

}



////////////////////////////////////////////////////////////////////////////////
// Status
////////////////////////////////////////////////////////////////////////////////
/// The entry file status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
	/// The file is unreadable, or its modification time could not be compared.
	Error,
	/// The entry file was found, but its counterpart was not.
	Absent,
	/// The entry file was found, but its counterpart was not.
	Exists,
	/// The entry file is newer than the counterpart.
	Newer,
	/// The entry file is older than the counterpart.
	Older,
	/// The entry file is the same as the counterpart.
	Same,
}

impl Status {
	fn write(
		&self,
		out: &mut dyn Write,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		if common.color.enabled() {
			write!(out, "{:<6}", match self {
				Status::Error  => "error".bright_red(),
				Status::Absent => "absent".bright_yellow(),
				Status::Exists => "exists".bright_green(),
				Status::Newer  => "newer".bright_green(),
				Status::Older  => "older".bright_yellow(),
				Status::Same   => "same".bright_white(),
			})
		} else {
			write!(out, "{:<6}", match self {
				Status::Error  => "error",
				Status::Absent => "absent",
				Status::Exists => "exists",
				Status::Newer  => "newer",
				Status::Older  => "older",
				Status::Same   => "same",
			})
		}
	}
}



////////////////////////////////////////////////////////////////////////////////
// Action
////////////////////////////////////////////////////////////////////////////////
/// An entry action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
	/// The file will be copied.
	Force,
	/// The file will be copied.
	Copy,
	/// The file will be skipped.
	Skip,
	/// The command was stopped.
	Stop,
}

impl Action {
	fn write(
		&self,
		out: &mut dyn Write,
		common: &CommonOptions)
		-> std::io::Result<()>
	{
		if common.quiet { return Ok(()); }

		if common.color.enabled() {
			write!(out, "{:<6}", match self {
				Action::Force => "force".bright_green(),
				Action::Copy  => "copy".bright_green(),
				Action::Skip  => "skip".bright_white(),
				Action::Stop  => "stop".bright_red(),
			})
		} else {
			write!(out, "{:<6}", match self {
				Action::Force => "force",
				Action::Copy  => "copy",
				Action::Skip  => "skip",
				Action::Stop  => "stop",
			})
		}
	}
}



////////////////////////////////////////////////////////////////////////////////
// File copy function.
////////////////////////////////////////////////////////////////////////////////
/// Copies a file from `source` to `target` using the given `CopyMethod`
fn copy(source: &Path, target: &Path, method: CopyMethod)
	-> Result<(), Error>
{
	let _span = span!(Level::DEBUG, "copy").entered();

	use CopyMethod::*;
	match method {
		None => event!(Level::DEBUG, "no-run flag was specified: \
			Not copying data from {:?} to {:?}", source, target),

		Subprocess => {
			let status = if cfg!(target_os = "windows") {
				std::process::Command::new("Xcopy")
					.arg(source)
					.arg(target)
					.args(["/h", "/s", "/e", "/x", "/y", "/i"])
					.status()
			} else {
				// NOTE: -R (recursive dir copy) and -p (preserve attribute
				// such as timestamps) are POSIX requirements.
				std::process::Command::new("cp")
					.args(["-R", "-p"])
					.arg(source)
					.arg(target)
					.status()
			};
			let _ = status.expect("execute copy command");
		},
	}
	Ok(())
}


////////////////////////////////////////////////////////////////////////////////
// CopyMethod
////////////////////////////////////////////////////////////////////////////////
/// The method to use when copying files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CopyMethod {
	/// Do not copy files.
	None,
	/// Copy files using a command in a subprocess.
	Subprocess,
}
