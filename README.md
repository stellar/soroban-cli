# Stellar CLI (stellar-cli)

This repo is home to the Stellar CLI, the command-line multi-tool for running and deploying Stellar contracts on the Stellar network.

## Documentation

For installation options see below, for usage instructions [see the full help docs](FULL_HELP_DOCS.md).

## Install
Install the latest version from source:
```
cargo install --locked stellar-cli --features opt
```

Install with `cargo-binstall`:
```
cargo install --locked cargo-binstall
cargo binstall -y stellar-cli
```

Install with Homebrew:

```
brew install stellar/tap/stellar-cli
```

## Install Experimental Features
To use the potentially unreleased bleeding edge CLI functionalities, install from git:
```
cargo install --locked stellar-cli --features opt --git git@github.com:stellar/stellar-cli.git
```

## Setup Autocomplete
```
stellar completion --shell <SHELL>
```
Possible SHELL values are `bash`, `elvish`, `fish`, `powershell`, `zsh`, etc.

To enable autocomplete in the current bash shell, run:
```
source <(stellar completion --shell bash)
```

To enable autocomplete permanently, run:
```
echo "source <(stellar completion --shell bash)" >> ~/.bashrc
```

## Latest Release
For latest releases, see [releases](https://github.com/stellar/stellar-cli/releases).

## Upcoming Features
For upcoming features, please see the [project board](https://github.com/orgs/stellar/projects/50).

## To Contribute
Find issues to contribute to [here](https://github.com/stellar/stellar-cli/contribute) and review [CONTRIBUTING.md](/CONTRIBUTING.md).

Developer Docs: https://developers.stellar.org/docs

Developer Docs CLI Examples: https://developers.stellar.org/docs/smart-contracts/guides/cli
