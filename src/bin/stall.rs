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
use stall::application::Prefs;
use stall::Stall;
use stall::application::TraceGuard;
use stall::CommandOptions;

// External library imports.
use anyhow::Context;
use anyhow::Error;
use anyhow::anyhow;
use clap::Parser;
use clap::ErrorKind;
use clap::CommandFactory as _;
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

		let exit_code = match err.downcast::<clap::Error>()
			.map(|e| e.kind())
		{
			Ok(ErrorKind::DisplayHelp)    |
			Ok(ErrorKind::DisplayVersion) => 0,
			_ => 1,
		};

		std::process::exit(exit_code);
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
	let cur_dir = std::env::current_dir()?;
	let config_path = match &common.config {
		Some(path) => path.clone(),
		None       => cur_dir.join(Config::DEFAULT_CONFIG_PATH),
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
	let rustc_meta = rustc_version_runtime::version_meta();
	event!(Level::DEBUG, "Rustc version: {} {:?}",
		rustc_meta.semver,
		rustc_meta.channel);
	if let Some(hash) = rustc_meta.commit_hash {
		event!(Level::DEBUG, "Rustc git commit: {}", hash);
	}
	event!(Level::DEBUG, "{:#?}", common);
	event!(Level::DEBUG, "{:#?}", command);
	event!(Level::DEBUG, "{:#?}", config);

	// Find the path for the prefs file.
	let prefs_path = match &common.prefs {
		Some(path) => path.clone(),
		None       => cur_dir.join(&config.prefs_path),
	};

	// Load the prefs file.
	let prefs = match Prefs::read_from_path(&prefs_path) {
		Err(e) if common.prefs.is_some() => {
			// Path is user-specified, so it is an error to now load it.
			return Err(Error::from(e)).with_context(|| format!(
				"Unable to load preferences file: {:?}", 
				prefs_path));
		},
		Err(_) => {
			// Path is default, so it is ok to use default prefs.
			event!(Level::DEBUG, "Using default prefs.");
			Prefs::new().with_load_path(prefs_path)
		},
		Ok(prefs) => {
			event!(Level::TRACE, "{:#?}", prefs); 
			prefs
		},
	};
	event!(Level::DEBUG, "{:#?}", prefs);

	// Find the paths for the stall directory and stall file.
	let (stall_dir, stall_path) = match command.stall() {
		Some(path) if path.is_file() && command.is_init() => {
			return Err(anyhow!("file already exists: {}", path.display()));
		},

		Some(path) if path.is_file() => ( 
			path.parent()
				.ok_or_else(|| anyhow!(
					"unable to determine stall parent directory: {}",
					path.display()))?
				.to_path_buf(),
			path.to_path_buf(),
		),

		Some(path) => (
			path.to_path_buf(),
			path.join(Config::DEFAULT_STALL_PATH),
		),

		None => (
			cur_dir.clone(),
			cur_dir.join(Config::DEFAULT_STALL_PATH),
		),
	};

	// Load/create the stall file.
	let mut stall_data = match Stall::read_from_path(&stall_path) {
		Err(e) if !command.is_init() => {
			return Err(Error::from(e)).with_context(|| format!(
				"Unable to load stall file: {:?}", 
				stall_path));
		},
		Err(_) => {
			// Path is default, so it is ok to use default stall.
			event!(Level::DEBUG, "Creating stall file with path {:?}",
				stall_path);
			Stall::new(stall_path)
		},

		Ok(stall_data) => {
			event!(Level::TRACE, "{:#?}", stall_data); 
			stall_data
		},
	};
	event!(Level::DEBUG, "{:#?}", stall_data);
	
	// Dispatch to appropriate commands.
	use CommandOptions::*;
	let res = match command {
		Init { common, dry_run, .. } => stall::init(
			stall_dir.as_path(),
			&mut stall_data,
			dry_run,
			&common),
		
		Status { common, .. } => stall::status(
			stall_dir.as_path(),
			&stall_data,
			&common),

		Add { common, files, rename, into, collect, dry_run, .. } => {
			// Emit error if using --rename with multiple files.
			if files.len() > 1 && rename.is_some() {
				// TODO: Figure out how to produce better error output.
				CommandOptions::command()
					.find_subcommand("add")
					.unwrap()
					.clone()
					.error(
						clap::ErrorKind::ArgumentConflict,
                    	"--rename option is not supported when multiple FILES \
                    		are provided")
                	.exit()
			}

			stall::add(
				&mut stall_data,
				files.iter().map(|f| f.as_path()),
				rename.as_ref().map(|p| p.as_path()),
				into.as_ref().map(|p| p.as_path()),
				if collect { Some(stall_dir.as_path()) } else { None },
				dry_run,
				&common)
		},

		Remove { common, files, delete, remote_naming, dry_run, .. } => {
			stall::remove(
				&mut stall_data,
				files.iter().map(|f| f.as_path()),
				if delete { Some(stall_dir.as_path()) } else { None },
				remote_naming,
				dry_run,
				&common)
		},

		Move { common, from, to, move_file, force, dry_run, .. } => {
			stall::rename(
				&mut stall_data,
				from.as_path(),
				to.as_path(),
				if move_file { Some(stall_dir.as_path()) } else { None },
				force,
				dry_run,
				&common)
		},

		Collect { common, files, force, dry_run, .. } => stall::collect(
			stall_dir.as_path(),
			&stall_data,
			files.iter().map(|f| f.as_path()),
			force,
			dry_run,
			&common),

		Distribute { common, files, force, dry_run, .. } => stall::distribute(
			stall_dir.as_path(),
			&stall_data,
			files.iter().map(|f| f.as_path()),
			force,
			dry_run,
			&common),
	};

	// Save the stall data if any changes occurred.
	// TODO: Should the stall be saved if an error occurs above?
	if stall_data.modified() {
		if stall_data.write_to_load_path()? {
			event!(Level::INFO, "Stall saved.");
		}
	}

	return res
}

