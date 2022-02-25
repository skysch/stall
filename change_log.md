
# Releases

The following is a change log, documenting added, removed, changed, depricated, and fixed features, along with the version number and official release date of that version.

## Stall 0.2  [2022-02-??] (Unreleased)
----------------------------------------------------

An essential redesign of the stall command, this release supports a number of git-like state management CLI commands to make it easier to update and track the state of the stall. You no longer have to manually edit the .stall file to add/remove files into the stall directory (see stall-add, stall-rm), do dry-runs of collect/distribute to see the state of the stalled files (see stall-status), or ponder at the different relative views of file statuses offered by collect/disribute (the new status lines show both local and remote files side-by-side.)

Stalled files can now be renamed within the stall directory, to ease support for collecting files with similar remote names.

### Added
+ Added `add` subcommand.
+ Added `rm` subcommand.
+ Added `status` subcommand.
+ Added `init` subcommand.
+ (TODO) Added `mv` subcommand.
+ Added support for file renaming.
+ Show both file states for readability.
+ Added a message to indicate no changes when the stall is empty.

### Breaking Changes
+ Updated command line flags.
+ Replaced structopt with clap 3.1.
+ Update application framework; replace logging with tracing.
+ Use fcmp to compare files.


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
