# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## Version 0.4.0 - 2024-06-19

### Added

- Process directories as they are found, instead of waiting for the search to complete.

## Version 0.3.0 - 2024-06-05

### Added

- Introduced `--dry-run` flag to simulate the execution of the command without actually running it.

### Fixed

- Fixed an issue where the options were logged as part of the command output.
- Fixed an issue where the `--hidden` flag was not working as expected.

## Version 0.2.0 - 2024-06-05

### Added

- Implemented parallel processing of repositories.
- Introduced `--hidden` flag to include hidden directories in the repository search.
- Introduced `--no-ignore` flag to bypass `.gitignore` files when searching for repositories.

## Version 0.1.1 - 2024-06-04

### Added

- Added support for Powershell and Homebrew installers.

## Version 0.1.0 - 2024-06-04

- Initial release of `git-foreach`.
