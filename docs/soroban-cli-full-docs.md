# Command-Line Help for `soroban`

This document contains the help content for the `soroban` command-line program.

**Command Overview:**

* [`soroban`↴](#soroban)
* [`soroban completion`↴](#soroban-completion)
* [`soroban config`↴](#soroban-config)
* [`soroban config network`↴](#soroban-config-network)
* [`soroban config network add`↴](#soroban-config-network-add)
* [`soroban config network rm`↴](#soroban-config-network-rm)
* [`soroban config network ls`↴](#soroban-config-network-ls)
* [`soroban config network start`↴](#soroban-config-network-start)
* [`soroban config network stop`↴](#soroban-config-network-stop)
* [`soroban config identity`↴](#soroban-config-identity)
* [`soroban config identity add`↴](#soroban-config-identity-add)
* [`soroban config identity address`↴](#soroban-config-identity-address)
* [`soroban config identity fund`↴](#soroban-config-identity-fund)
* [`soroban config identity generate`↴](#soroban-config-identity-generate)
* [`soroban config identity ls`↴](#soroban-config-identity-ls)
* [`soroban config identity rm`↴](#soroban-config-identity-rm)
* [`soroban config identity show`↴](#soroban-config-identity-show)
* [`soroban contract`↴](#soroban-contract)
* [`soroban contract asset`↴](#soroban-contract-asset)
* [`soroban contract asset id`↴](#soroban-contract-asset-id)
* [`soroban contract asset deploy`↴](#soroban-contract-asset-deploy)
* [`soroban contract bindings`↴](#soroban-contract-bindings)
* [`soroban contract bindings json`↴](#soroban-contract-bindings-json)
* [`soroban contract bindings rust`↴](#soroban-contract-bindings-rust)
* [`soroban contract bindings typescript`↴](#soroban-contract-bindings-typescript)
* [`soroban contract build`↴](#soroban-contract-build)
* [`soroban contract extend`↴](#soroban-contract-extend)
* [`soroban contract deploy`↴](#soroban-contract-deploy)
* [`soroban contract fetch`↴](#soroban-contract-fetch)
* [`soroban contract id`↴](#soroban-contract-id)
* [`soroban contract id asset`↴](#soroban-contract-id-asset)
* [`soroban contract id wasm`↴](#soroban-contract-id-wasm)
* [`soroban contract init`↴](#soroban-contract-init)
* [`soroban contract inspect`↴](#soroban-contract-inspect)
* [`soroban contract install`↴](#soroban-contract-install)
* [`soroban contract invoke`↴](#soroban-contract-invoke)
* [`soroban contract optimize`↴](#soroban-contract-optimize)
* [`soroban contract read`↴](#soroban-contract-read)
* [`soroban contract restore`↴](#soroban-contract-restore)
* [`soroban events`↴](#soroban-events)
* [`soroban keys`↴](#soroban-keys)
* [`soroban keys add`↴](#soroban-keys-add)
* [`soroban keys address`↴](#soroban-keys-address)
* [`soroban keys fund`↴](#soroban-keys-fund)
* [`soroban keys generate`↴](#soroban-keys-generate)
* [`soroban keys ls`↴](#soroban-keys-ls)
* [`soroban keys rm`↴](#soroban-keys-rm)
* [`soroban keys show`↴](#soroban-keys-show)
* [`soroban lab`↴](#soroban-lab)
* [`soroban lab token`↴](#soroban-lab-token)
* [`soroban lab token wrap`↴](#soroban-lab-token-wrap)
* [`soroban lab token id`↴](#soroban-lab-token-id)
* [`soroban lab xdr`↴](#soroban-lab-xdr)
* [`soroban lab xdr types`↴](#soroban-lab-xdr-types)
* [`soroban lab xdr types list`↴](#soroban-lab-xdr-types-list)
* [`soroban lab xdr guess`↴](#soroban-lab-xdr-guess)
* [`soroban lab xdr decode`↴](#soroban-lab-xdr-decode)
* [`soroban lab xdr encode`↴](#soroban-lab-xdr-encode)
* [`soroban lab xdr version`↴](#soroban-lab-xdr-version)
* [`soroban network`↴](#soroban-network)
* [`soroban network add`↴](#soroban-network-add)
* [`soroban network rm`↴](#soroban-network-rm)
* [`soroban network ls`↴](#soroban-network-ls)
* [`soroban network start`↴](#soroban-network-start)
* [`soroban network stop`↴](#soroban-network-stop)
* [`soroban version`↴](#soroban-version)

## `soroban`

Build, deploy, & interact with contracts; set identities to sign with; configure networks; generate keys; and more.

Intro: https://soroban.stellar.org/docs
CLI Reference: https://github.com/stellar/soroban-cli/tree/main/docs/soroban-cli-full-docs.md

The easiest way to get started is to generate a new identity:

    soroban config identity generate alice

You can use identities with the `--source` flag in other commands later.

Commands that relate to smart contract interactions are organized under the `contract` subcommand. List them:

    soroban contract --help

A Soroban contract has its interface schema types embedded in the binary that gets deployed on-chain, making it possible to dynamically generate a custom CLI for each. `soroban contract invoke` makes use of this:

    soroban contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- --help

Anything after the `--` double dash (the "slop") is parsed as arguments to the contract-specific CLI, generated on-the-fly from the embedded schema. For the hello world example, with a function called `hello` that takes one string argument `to`, here's how you invoke it:

    soroban contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- hello --to world

Full CLI reference: https://github.com/stellar/soroban-tools/tree/main/docs/soroban-cli-full-docs.md

**Usage:** `soroban [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `completion` — Print shell completion code for the specified shell
* `config` — Deprecated, use `soroban keys` and `soroban network` instead
* `contract` — Tools for smart contract developers
* `events` — Watch the network for contract events
* `keys` — Create and manage identities including keys and addresses
* `lab` — Experiment with early features and expert tools
* `network` — Start and configure networks
* `version` — Print version information

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-f`, `--filter-logs <FILTER_LOGS>` — Filter logs output. To turn on "soroban_cli::log::footprint=debug" or off "=off". Can also use env var `RUST_LOG`
* `-q`, `--quiet` — Do not write logs to stderr including `INFO`

  Possible values: `true`, `false`

* `-v`, `--verbose` — Log DEBUG events

  Possible values: `true`, `false`

* `--very-verbose` — Log DEBUG and TRACE events

  Possible values: `true`, `false`

* `--list` — List installed plugins. E.g. `soroban-hello`

  Possible values: `true`, `false`




## `soroban completion`

Print shell completion code for the specified shell

Ensure the completion package for your shell is installed,
e.g., bash-completion for bash.

To enable autocomplete in the current bash shell, run:
  source <(soroban completion --shell bash)

To enable autocomplete permanently, run:
  echo "source <(soroban completion --shell bash)" >> ~/.bashrc

**Usage:** `soroban completion --shell <SHELL>`

###### **Options:**

* `--shell <SHELL>` — The shell type

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`




## `soroban config`

Deprecated, use `soroban keys` and `soroban network` instead

**Usage:** `soroban config <COMMAND>`

###### **Subcommands:**

* `network` — Configure different networks. Depraecated, use `soroban network` instead
* `identity` — Identity management. Deprecated, use `soroban keys` instead



## `soroban config network`

Configure different networks. Depraecated, use `soroban network` instead

**Usage:** `soroban config network <COMMAND>`

###### **Subcommands:**

* `add` — Add a new network
* `rm` — Remove a network
* `ls` — List networks
* `start` — Start network
* `stop` — Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it



## `soroban config network add`

Add a new network

**Usage:** `soroban config network add [OPTIONS] --rpc-url <RPC_URL> --network-passphrase <NETWORK_PASSPHRASE> <NAME>`

###### **Arguments:**

* `<NAME>` — Name of network

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config network rm`

Remove a network

**Usage:** `soroban config network rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Network to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config network ls`

List networks

**Usage:** `soroban config network ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long` — Get more info about the networks

  Possible values: `true`, `false`




## `soroban config network start`

Start network

Start a container running a Stellar node, RPC, API, and friendbot (faucet).

soroban network start <NETWORK> [OPTIONS]

By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command: docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc

**Usage:** `soroban config network start [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to start

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock
* `-l`, `--limits <LIMITS>` — Optional argument to specify the limits for the local network only
* `-p`, `--ports-mapping <PORTS_MAPPING>` — Argument to specify the HOST_PORT:CONTAINER_PORT mapping

  Default value: `8000:8000`
* `-t`, `--image-tag-override <IMAGE_TAG_OVERRIDE>` — Optional argument to override the default docker image tag for the given network
* `-v`, `--protocol-version <PROTOCOL_VERSION>` — Optional argument to specify the protocol version for the local network only



## `soroban config network stop`

Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it

**Usage:** `soroban config network stop [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to stop

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock



## `soroban config identity`

Identity management. Deprecated, use `soroban keys` instead

**Usage:** `soroban config identity <COMMAND>`

###### **Subcommands:**

* `add` — Add a new identity (keypair, ledger, macOS keychain)
* `address` — Given an identity return its address (public key)
* `fund` — Fund an identity on a test network
* `generate` — Generate a new identity with a seed phrase, currently 12 words
* `ls` — List identities
* `rm` — Remove an identity
* `show` — Given an identity return its private key



## `soroban config identity add`

Add a new identity (keypair, ledger, macOS keychain)

**Usage:** `soroban config identity add [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity

###### **Options:**

* `--secret-key` — Add using secret_key Can provide with SOROBAN_SECRET_KEY

  Possible values: `true`, `false`

* `--seed-phrase` — Add using 12 word seed phrase to generate secret_key

  Possible values: `true`, `false`

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config identity address`

Given an identity return its address (public key)

**Usage:** `soroban config identity address [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default test identity used if not provided

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config identity fund`

Fund an identity on a test network

**Usage:** `soroban config identity fund [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default test identity used if not provided

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config identity generate`

Generate a new identity with a seed phrase, currently 12 words

**Usage:** `soroban config identity generate [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity

###### **Options:**

* `--no-fund` — Do not fund address

  Possible values: `true`, `false`

* `--seed <SEED>` — Optional seed to use when generating seed phrase. Random otherwise
* `-s`, `--as-secret` — Output the generated identity as a secret key

  Possible values: `true`, `false`

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--hd-path <HD_PATH>` — When generating a secret key, which hd_path should be used from the original seed_phrase
* `-d`, `--default-seed` — Generate the default seed phrase. Useful for testing. Equivalent to --seed 0000000000000000

  Possible values: `true`, `false`

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `soroban config identity ls`

List identities

**Usage:** `soroban config identity ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long`

  Possible values: `true`, `false`




## `soroban config identity rm`

Remove an identity

**Usage:** `soroban config identity rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Identity to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban config identity show`

Given an identity return its private key

**Usage:** `soroban config identity show [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default is test identity

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract`

Tools for smart contract developers

**Usage:** `soroban contract <COMMAND>`

###### **Subcommands:**

* `asset` — Utilities to deploy a Stellar Asset Contract or get its id
* `bindings` — Generate code client bindings for a contract
* `build` — Build a contract from source
* `extend` — Extend the time to live ledger of a contract-data ledger entry
* `deploy` — Deploy a wasm contract
* `fetch` — Fetch a contract's Wasm binary
* `id` — Generate the contract id for a given contract or asset
* `init` — Initialize a Soroban project with an example contract
* `inspect` — Inspect a WASM file listing contract functions, meta, etc
* `install` — Install a WASM file to the ledger without creating a contract instance
* `invoke` — Invoke a contract function
* `optimize` — Optimize a WASM file
* `read` — Print the current value of a contract-data ledger entry
* `restore` — Restore an evicted value for a contract-data legder entry



## `soroban contract asset`

Utilities to deploy a Stellar Asset Contract or get its id

**Usage:** `soroban contract asset <COMMAND>`

###### **Subcommands:**

* `id` — Get Id of builtin Soroban Asset Contract. Deprecated, use `soroban contract id asset` instead
* `deploy` — Deploy builtin Soroban Asset Contract



## `soroban contract asset id`

Get Id of builtin Soroban Asset Contract. Deprecated, use `soroban contract id asset` instead

**Usage:** `soroban contract asset id [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract asset deploy`

Deploy builtin Soroban Asset Contract

**Usage:** `soroban contract asset deploy [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`




## `soroban contract bindings`

Generate code client bindings for a contract

**Usage:** `soroban contract bindings <COMMAND>`

###### **Subcommands:**

* `json` — Generate Json Bindings
* `rust` — Generate Rust bindings
* `typescript` — Generate a TypeScript / JavaScript package



## `soroban contract bindings json`

Generate Json Bindings

**Usage:** `soroban contract bindings json --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary



## `soroban contract bindings rust`

Generate Rust bindings

**Usage:** `soroban contract bindings rust --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary



## `soroban contract bindings typescript`

Generate a TypeScript / JavaScript package

**Usage:** `soroban contract bindings typescript [OPTIONS] --output-dir <OUTPUT_DIR> --contract-id <CONTRACT_ID>`

###### **Options:**

* `--wasm <WASM>` — Path to optional wasm binary
* `--output-dir <OUTPUT_DIR>` — Where to place generated project
* `--overwrite` — Whether to overwrite output directory if it already exists

  Possible values: `true`, `false`

* `--contract-id <CONTRACT_ID>` — The contract ID/address on the network
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `soroban contract build`

Build a contract from source

Builds all crates that are referenced by the cargo manifest (Cargo.toml) that have cdylib as their crate-type. Crates are built for the wasm32 target. Unless configured otherwise, crates are built with their default features and with their release profile.

To view the commands that will be executed, without executing them, use the --print-commands-only option.

**Usage:** `soroban contract build [OPTIONS]`

###### **Options:**

* `--manifest-path <MANIFEST_PATH>` — Path to Cargo.toml

  Default value: `Cargo.toml`
* `--package <PACKAGE>` — Package to build
* `--profile <PROFILE>` — Build with the specified profile

  Default value: `release`
* `--features <FEATURES>` — Build with the list of features activated, space or comma separated
* `--all-features` — Build with the all features activated

  Possible values: `true`, `false`

* `--no-default-features` — Build with the default feature not activated

  Possible values: `true`, `false`

* `--out-dir <OUT_DIR>` — Directory to copy wasm files to
* `--print-commands-only` — Print commands to build without executing them

  Possible values: `true`, `false`




## `soroban contract extend`

Extend the time to live ledger of a contract-data ledger entry.

If no keys are specified the contract itself is extended.

**Usage:** `soroban contract extend [OPTIONS] --ledgers-to-extend <LEDGERS_TO_EXTEND> --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--ledgers-to-extend <LEDGERS_TO_EXTEND>` — Number of ledgers to extend the entries
* `--ttl-ledger-only` — Only print the new Time To Live ledger

  Possible values: `true`, `false`

* `--id <CONTRACT_ID>` — Contract ID to which owns the data entries. If no keys provided the Contract's instance will be extended
* `--key <KEY>` — Storage key (symbols only)
* `--key-xdr <KEY_XDR>` — Storage key (base64-encoded XDR)
* `--wasm <WASM>` — Path to Wasm file of contract code to extend
* `--wasm-hash <WASM_HASH>` — Path to Wasm file of contract code to extend
* `--durability <DURABILITY>` — Storage entry durability

  Default value: `persistent`

  Possible values:
  - `persistent`:
    Persistent
  - `temporary`:
    Temporary

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`




## `soroban contract deploy`

Deploy a wasm contract

**Usage:** `soroban contract deploy [OPTIONS] --source-account <SOURCE_ACCOUNT> <--wasm <WASM>|--wasm-hash <WASM_HASH>>`

###### **Options:**

* `--wasm <WASM>` — WASM file to deploy
* `--wasm-hash <WASM_HASH>` — Hash of the already installed/deployed WASM file
* `--salt <SALT>` — Custom salt 32-byte salt for the token id
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`

* `-i`, `--ignore-checks` — Whether to ignore safety checks when deploying contracts

  Default value: `false`

  Possible values: `true`, `false`




## `soroban contract fetch`

Fetch a contract's Wasm binary

**Usage:** `soroban contract fetch [OPTIONS] --id <CONTRACT_ID>`

###### **Options:**

* `--id <CONTRACT_ID>` — Contract ID to fetch
* `-o`, `--out-file <OUT_FILE>` — Where to write output otherwise stdout is used
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `soroban contract id`

Generate the contract id for a given contract or asset

**Usage:** `soroban contract id <COMMAND>`

###### **Subcommands:**

* `asset` — Deploy builtin Soroban Asset Contract
* `wasm` — Deploy normal Wasm Contract



## `soroban contract id asset`

Deploy builtin Soroban Asset Contract

**Usage:** `soroban contract id asset [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract id wasm`

Deploy normal Wasm Contract

**Usage:** `soroban contract id wasm [OPTIONS] --salt <SALT> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--salt <SALT>` — ID of the Soroban contract
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract init`

Initialize a Soroban project with an example contract

**Usage:** `soroban contract init [OPTIONS] <PROJECT_PATH>`

###### **Arguments:**

* `<PROJECT_PATH>`

###### **Options:**

* `-w`, `--with-example <WITH_EXAMPLE>`

  Possible values: `account`, `alloc`, `atomic_multiswap`, `atomic_swap`, `auth`, `cross_contract`, `custom_types`, `deep_contract_auth`, `deployer`, `errors`, `eth_abi`, `events`, `fuzzing`, `increment`, `liquidity_pool`, `logging`, `mint-lock`, `simple_account`, `single_offer`, `timelock`, `token`, `upgradeable_contract`, `workspace`

* `-f`, `--frontend-template <FRONTEND_TEMPLATE>`

  Default value: ``



## `soroban contract inspect`

Inspect a WASM file listing contract functions, meta, etc

**Usage:** `soroban contract inspect [OPTIONS] --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary
* `--output <OUTPUT>` — Output just XDR in base64

  Default value: `docs`

  Possible values:
  - `xdr-base64`:
    XDR of array of contract spec entries
  - `xdr-base64-array`:
    Array of xdr of contract spec entries
  - `docs`:
    Pretty print of contract spec entries

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract install`

Install a WASM file to the ledger without creating a contract instance

**Usage:** `soroban contract install [OPTIONS] --source-account <SOURCE_ACCOUNT> --wasm <WASM>`

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`

* `--wasm <WASM>` — Path to wasm binary
* `-i`, `--ignore-checks` — Whether to ignore safety checks when deploying contracts

  Default value: `false`

  Possible values: `true`, `false`




## `soroban contract invoke`

Invoke a contract function

Generates an "implicit CLI" for the specified contract on-the-fly using the contract's schema, which gets embedded into every Soroban contract. The "slop" in this command, everything after the `--`, gets passed to this implicit CLI. Get in-depth help for a given contract:

soroban contract invoke ... -- --help

**Usage:** `soroban contract invoke [OPTIONS] --id <CONTRACT_ID> --source-account <SOURCE_ACCOUNT> [-- <CONTRACT_FN_AND_ARGS>...]`

###### **Arguments:**

* `<CONTRACT_FN_AND_ARGS>` — Function name as subcommand, then arguments for that function as `--arg-name value`

###### **Options:**

* `--id <CONTRACT_ID>` — Contract ID to invoke
* `--is-view` — View the result simulating and do not sign and submit transaction

  Possible values: `true`, `false`

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`




## `soroban contract optimize`

Optimize a WASM file

**Usage:** `soroban contract optimize [OPTIONS] --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary
* `--wasm-out <WASM_OUT>` — Path to write the optimized WASM file to (defaults to same location as --wasm with .optimized.wasm suffix)



## `soroban contract read`

Print the current value of a contract-data ledger entry

**Usage:** `soroban contract read [OPTIONS] --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--output <OUTPUT>` — Type of output to generate

  Default value: `string`

  Possible values:
  - `string`:
    String
  - `json`:
    Json
  - `xdr`:
    XDR

* `--id <CONTRACT_ID>` — Contract ID to which owns the data entries. If no keys provided the Contract's instance will be extended
* `--key <KEY>` — Storage key (symbols only)
* `--key-xdr <KEY_XDR>` — Storage key (base64-encoded XDR)
* `--wasm <WASM>` — Path to Wasm file of contract code to extend
* `--wasm-hash <WASM_HASH>` — Path to Wasm file of contract code to extend
* `--durability <DURABILITY>` — Storage entry durability

  Default value: `persistent`

  Possible values:
  - `persistent`:
    Persistent
  - `temporary`:
    Temporary

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban contract restore`

Restore an evicted value for a contract-data legder entry.

If no keys are specificed the contract itself is restored.

**Usage:** `soroban contract restore [OPTIONS] --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--id <CONTRACT_ID>` — Contract ID to which owns the data entries. If no keys provided the Contract's instance will be extended
* `--key <KEY>` — Storage key (symbols only)
* `--key-xdr <KEY_XDR>` — Storage key (base64-encoded XDR)
* `--wasm <WASM>` — Path to Wasm file of contract code to extend
* `--wasm-hash <WASM_HASH>` — Path to Wasm file of contract code to extend
* `--durability <DURABILITY>` — Storage entry durability

  Default value: `persistent`

  Possible values:
  - `persistent`:
    Persistent
  - `temporary`:
    Temporary

* `--ledgers-to-extend <LEDGERS_TO_EXTEND>` — Number of ledgers to extend the entry
* `--ttl-ledger-only` — Only print the new Time To Live ledger

  Possible values: `true`, `false`

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`




## `soroban events`

Watch the network for contract events

**Usage:** `soroban events [OPTIONS]`

###### **Options:**

* `--start-ledger <START_LEDGER>` — The first ledger sequence number in the range to pull events https://developers.stellar.org/docs/encyclopedia/ledger-headers#ledger-sequence
* `--cursor <CURSOR>` — The cursor corresponding to the start of the event range
* `--output <OUTPUT>` — Output formatting options for event stream

  Default value: `pretty`

  Possible values:
  - `pretty`:
    Colorful, human-oriented console output
  - `plain`:
    Human-oriented console output without colors
  - `json`:
    JSONified console output

* `-c`, `--count <COUNT>` — The maximum number of events to display (defer to the server-defined limit)

  Default value: `10`
* `--id <CONTRACT_IDS>` — A set of (up to 5) contract IDs to filter events on. This parameter can be passed multiple times, e.g. `--id C123.. --id C456..`, or passed with multiple parameters, e.g. `--id C123 C456`
* `--topic <TOPIC_FILTERS>` — A set of (up to 4) topic filters to filter event topics on. A single topic filter can contain 1-4 different segment filters, separated by commas, with an asterisk (* character) indicating a wildcard segment
* `--type <EVENT_TYPE>` — Specifies which type of contract events to display

  Default value: `all`

  Possible values: `all`, `contract`, `system`

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `soroban keys`

Create and manage identities including keys and addresses

**Usage:** `soroban keys <COMMAND>`

###### **Subcommands:**

* `add` — Add a new identity (keypair, ledger, macOS keychain)
* `address` — Given an identity return its address (public key)
* `fund` — Fund an identity on a test network
* `generate` — Generate a new identity with a seed phrase, currently 12 words
* `ls` — List identities
* `rm` — Remove an identity
* `show` — Given an identity return its private key



## `soroban keys add`

Add a new identity (keypair, ledger, macOS keychain)

**Usage:** `soroban keys add [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity

###### **Options:**

* `--secret-key` — Add using secret_key Can provide with SOROBAN_SECRET_KEY

  Possible values: `true`, `false`

* `--seed-phrase` — Add using 12 word seed phrase to generate secret_key

  Possible values: `true`, `false`

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban keys address`

Given an identity return its address (public key)

**Usage:** `soroban keys address [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default test identity used if not provided

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban keys fund`

Fund an identity on a test network

**Usage:** `soroban keys fund [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default test identity used if not provided

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban keys generate`

Generate a new identity with a seed phrase, currently 12 words

**Usage:** `soroban keys generate [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity

###### **Options:**

* `--no-fund` — Do not fund address

  Possible values: `true`, `false`

* `--seed <SEED>` — Optional seed to use when generating seed phrase. Random otherwise
* `-s`, `--as-secret` — Output the generated identity as a secret key

  Possible values: `true`, `false`

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--hd-path <HD_PATH>` — When generating a secret key, which hd_path should be used from the original seed_phrase
* `-d`, `--default-seed` — Generate the default seed phrase. Useful for testing. Equivalent to --seed 0000000000000000

  Possible values: `true`, `false`

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `soroban keys ls`

List identities

**Usage:** `soroban keys ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long`

  Possible values: `true`, `false`




## `soroban keys rm`

Remove an identity

**Usage:** `soroban keys rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Identity to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban keys show`

Given an identity return its private key

**Usage:** `soroban keys show [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default is test identity

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban lab`

Experiment with early features and expert tools

**Usage:** `soroban lab <COMMAND>`

###### **Subcommands:**

* `token` — Wrap, create, and manage token contracts
* `xdr` — Decode xdr



## `soroban lab token`

Wrap, create, and manage token contracts

**Usage:** `soroban lab token <COMMAND>`

###### **Subcommands:**

* `wrap` — Deploy a token contract to wrap an existing Stellar classic asset for smart contract usage Deprecated, use `soroban contract deploy asset` instead
* `id` — Compute the expected contract id for the given asset Deprecated, use `soroban contract id asset` instead



## `soroban lab token wrap`

Deploy a token contract to wrap an existing Stellar classic asset for smart contract usage Deprecated, use `soroban contract deploy asset` instead

**Usage:** `soroban lab token wrap [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulation the transaction only write the base64 to stdout

  Possible values: `true`, `false`




## `soroban lab token id`

Compute the expected contract id for the given asset Deprecated, use `soroban contract id asset` instead

**Usage:** `soroban lab token id [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…"). Default: `identity generate --default-seed`
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban lab xdr`

Decode xdr

**Usage:** `soroban lab xdr [CHANNEL] <COMMAND>`

###### **Subcommands:**

* `types` — View information about types
* `guess` — Guess the XDR type
* `decode` — Decode XDR
* `encode` — Encode XDR
* `version` — Print version information

###### **Arguments:**

* `<CHANNEL>` — Channel of XDR to operate on

  Default value: `+curr`

  Possible values: `+curr`, `+next`




## `soroban lab xdr types`

View information about types

**Usage:** `soroban lab xdr types <COMMAND>`

###### **Subcommands:**

* `list` — 



## `soroban lab xdr types list`

**Usage:** `soroban lab xdr types list [OPTIONS]`

###### **Options:**

* `--output <OUTPUT>`

  Default value: `plain`

  Possible values: `plain`, `json`, `json-formatted`




## `soroban lab xdr guess`

Guess the XDR type

**Usage:** `soroban lab xdr guess [OPTIONS] [FILE]`

###### **Arguments:**

* `<FILE>` — File to decode, or stdin if omitted

###### **Options:**

* `--input <INPUT>`

  Default value: `single-base64`

  Possible values: `single`, `single-base64`, `stream`, `stream-base64`, `stream-framed`

* `--output <OUTPUT>`

  Default value: `list`

  Possible values: `list`

* `--certainty <CERTAINTY>` — Certainty as an arbitrary value

  Default value: `2`



## `soroban lab xdr decode`

Decode XDR

**Usage:** `soroban lab xdr decode [OPTIONS] --type <TYPE> [FILES]...`

###### **Arguments:**

* `<FILES>` — Files to decode, or stdin if omitted

###### **Options:**

* `--type <TYPE>` — XDR type to decode
* `--input <INPUT>`

  Default value: `stream-base64`

  Possible values: `single`, `single-base64`, `stream`, `stream-base64`, `stream-framed`

* `--output <OUTPUT>`

  Default value: `json`

  Possible values: `json`, `json-formatted`




## `soroban lab xdr encode`

Encode XDR

**Usage:** `soroban lab xdr encode [OPTIONS] --type <TYPE> [FILES]...`

###### **Arguments:**

* `<FILES>` — Files to encode, or stdin if omitted

###### **Options:**

* `--type <TYPE>` — XDR type to encode
* `--input <INPUT>`

  Default value: `json`

  Possible values: `json`

* `--output <OUTPUT>`

  Default value: `single-base64`

  Possible values: `single`, `single-base64`




## `soroban lab xdr version`

Print version information

**Usage:** `soroban lab xdr version`



## `soroban network`

Start and configure networks

**Usage:** `soroban network <COMMAND>`

###### **Subcommands:**

* `add` — Add a new network
* `rm` — Remove a network
* `ls` — List networks
* `start` — Start network
* `stop` — Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it



## `soroban network add`

Add a new network

**Usage:** `soroban network add [OPTIONS] --rpc-url <RPC_URL> --network-passphrase <NETWORK_PASSPHRASE> <NAME>`

###### **Arguments:**

* `<NAME>` — Name of network

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban network rm`

Remove a network

**Usage:** `soroban network rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Network to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `soroban network ls`

List networks

**Usage:** `soroban network ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long` — Get more info about the networks

  Possible values: `true`, `false`




## `soroban network start`

Start network

Start a container running a Stellar node, RPC, API, and friendbot (faucet).

soroban network start <NETWORK> [OPTIONS]

By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command: docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc

**Usage:** `soroban network start [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to start

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock
* `-l`, `--limits <LIMITS>` — Optional argument to specify the limits for the local network only
* `-p`, `--ports-mapping <PORTS_MAPPING>` — Argument to specify the HOST_PORT:CONTAINER_PORT mapping

  Default value: `8000:8000`
* `-t`, `--image-tag-override <IMAGE_TAG_OVERRIDE>` — Optional argument to override the default docker image tag for the given network
* `-v`, `--protocol-version <PROTOCOL_VERSION>` — Optional argument to specify the protocol version for the local network only



## `soroban network stop`

Stop a network started with `network start`. For example, if you ran `soroban network start local`, you can use `soroban network stop local` to stop it

**Usage:** `soroban network stop [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to stop

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock



## `soroban version`

Print version information

**Usage:** `soroban version`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
