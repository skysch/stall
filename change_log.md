
# Releases

The following is a change log, documenting added, removed, changed, depricated, and fixed features, along with the version number and official release date of that version.

## Unreleased
-------------

Implemented changes not yet published.

### Added
+ N/A

### Fixed
+ Fixed 'STATE' label for the error-skip case on the distribute command.

## Stall 0.1  [2020-00-00]
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
