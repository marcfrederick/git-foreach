# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## Version 0.4.1 - 2024-07-23

### Added

- Enabled the creation
  of [GitHub Attestations](https://github.blog/changelog/2024-06-25-artifact-attestations-is-generally-available/) for
  future release artifacts.
- The project now supports both MIT and Apache 2.0 licenses. This means, that users can now choose between the two
  licenses when using the project (previously, only the MIT license was supported).
- Separated the project into separate crates for the binary and library components.
- Integrated [cargo-deny](https://www.github.com/EmbarkStudios/cargo-deny) to enforce strict licensing and security
  policies.

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
