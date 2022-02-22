
# Releases

The following is a change log, documenting added, removed, changed, depricated, and fixed features, along with the version number and official release date of that version.

## Stall 0.2  [2022-02-??] (Unreleased)
----------------------------------------------------

Implemented changes not yet published.

### Added
+ (TODO) Added `add` command.
+ (TODO) Added `rm` command.
+ (TODO) Added `status` command.
+ (TODO) Added `init` command.
+ Added support for file renaming.
+ (TODO) Show both file states for readability.
+ Added a message to indicate no changes when the stall is empty.

### Fixed
+ Fixed 'STATE' label for the error-skip case on the distribute command.

### Changed
+ Updated command line flags.
+ Replaced structopt with clap 3.1.
+ Update application framework; replace logging with tracing.
+ (TODO) Use fcmp to compare files.


## Stall 0.1  [2020-07-01]
----------------------------------------------------

### Added
+ Implemented `collect` and `distribute` subcommands.
+ Implemented `RON` and `list` style config formats.
+ Implemented `--use-config` flag to choose the config file explicitely.
+ Implemented `--config-format` flag to choose the config format explicitely, rather than using the try-parse-and-fallback approach.
+ Implemented `--dry-run` flag for testing.
+ Implemented `--short-names` flag to help output readability for familiar stalls.
+ Implemented `--force` flag to force copy older files.
+ Implemented `--error` flag to exit with and error if a file is missing or unreadable.
+ Implemented `--verbose`, `--quiet`, and `--ztrace` flags to control output verbosity.
