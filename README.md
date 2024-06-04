# git-foreach

`git-foreach` is a lightweight utility designed to execute a command in each Git repository within a directory and its
subdirectories. It's particularly useful for performing bulk operations, such as `git pull` or `git status`, across
multiple repositories simultaneously.

## Usage

The specified `<command>` will be executed in each Git repository found in the current directory and all its
subdirectories.

```shell
git-foreach <command>
```

## Installation

Pre-built binaries are available for Windows, Linux, and macOS. You can download them from
the [releases page](https://github.com/marcfrederick/git-foreach/releases) and add them to your `$PATH`. The release
page also includes instructions for installation using [Homebrew](https://brew.sh), `curl`, or PowerShell.

## Building from Source

To build the utility from source, ensure you have [Rust](https://www.rust-lang.org) installed on your system. Then,
execute the following commands:

```shell
git clone git@github.com:marcfrederick/git-foreach.git
cd git-foreach
cargo build --release
```

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests on
the [GitHub repository](https://github.com/marcfrederick/git-foreach).

## License

This project is licensed under the MIT License. See
the [LICENSE](https://github.com/marcfrederick/git-foreach/blob/main/LICENSE) file for details.
