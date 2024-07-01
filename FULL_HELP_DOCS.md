# Command-Line Help for `stellar`

This document contains the help content for the `stellar` command-line program.

**Command Overview:**

* [`stellar`↴](#stellar)
* [`stellar completion`↴](#stellar-completion)
* [`stellar contract`↴](#stellar-contract)
* [`stellar contract asset`↴](#stellar-contract-asset)
* [`stellar contract asset id`↴](#stellar-contract-asset-id)
* [`stellar contract asset deploy`↴](#stellar-contract-asset-deploy)
* [`stellar contract bindings`↴](#stellar-contract-bindings)
* [`stellar contract bindings json`↴](#stellar-contract-bindings-json)
* [`stellar contract bindings rust`↴](#stellar-contract-bindings-rust)
* [`stellar contract bindings typescript`↴](#stellar-contract-bindings-typescript)
* [`stellar contract build`↴](#stellar-contract-build)
* [`stellar contract extend`↴](#stellar-contract-extend)
* [`stellar contract deploy`↴](#stellar-contract-deploy)
* [`stellar contract fetch`↴](#stellar-contract-fetch)
* [`stellar contract id`↴](#stellar-contract-id)
* [`stellar contract id asset`↴](#stellar-contract-id-asset)
* [`stellar contract id wasm`↴](#stellar-contract-id-wasm)
* [`stellar contract init`↴](#stellar-contract-init)
* [`stellar contract inspect`↴](#stellar-contract-inspect)
* [`stellar contract install`↴](#stellar-contract-install)
* [`stellar contract invoke`↴](#stellar-contract-invoke)
* [`stellar contract optimize`↴](#stellar-contract-optimize)
* [`stellar contract read`↴](#stellar-contract-read)
* [`stellar contract restore`↴](#stellar-contract-restore)
* [`stellar events`↴](#stellar-events)
* [`stellar keys`↴](#stellar-keys)
* [`stellar keys add`↴](#stellar-keys-add)
* [`stellar keys address`↴](#stellar-keys-address)
* [`stellar keys fund`↴](#stellar-keys-fund)
* [`stellar keys generate`↴](#stellar-keys-generate)
* [`stellar keys ls`↴](#stellar-keys-ls)
* [`stellar keys rm`↴](#stellar-keys-rm)
* [`stellar keys show`↴](#stellar-keys-show)
* [`stellar xdr`↴](#stellar-xdr)
* [`stellar xdr types`↴](#stellar-xdr-types)
* [`stellar xdr types list`↴](#stellar-xdr-types-list)
* [`stellar xdr types schema`↴](#stellar-xdr-types-schema)
* [`stellar xdr guess`↴](#stellar-xdr-guess)
* [`stellar xdr decode`↴](#stellar-xdr-decode)
* [`stellar xdr encode`↴](#stellar-xdr-encode)
* [`stellar xdr version`↴](#stellar-xdr-version)
* [`stellar network`↴](#stellar-network)
* [`stellar network add`↴](#stellar-network-add)
* [`stellar network rm`↴](#stellar-network-rm)
* [`stellar network ls`↴](#stellar-network-ls)
* [`stellar network start`↴](#stellar-network-start)
* [`stellar network stop`↴](#stellar-network-stop)
* [`stellar network container`↴](#stellar-network-container)
* [`stellar network container logs`↴](#stellar-network-container-logs)
* [`stellar network container start`↴](#stellar-network-container-start)
* [`stellar network container stop`↴](#stellar-network-container-stop)
* [`stellar version`↴](#stellar-version)
* [`stellar tx`↴](#stellar-tx)
* [`stellar tx simulate`↴](#stellar-tx-simulate)
* [`stellar tx sign`↴](#stellar-tx-sign)
* [`stellar tx send`↴](#stellar-tx-send)
* [`stellar cache`↴](#stellar-cache)
* [`stellar cache clean`↴](#stellar-cache-clean)
* [`stellar cache path`↴](#stellar-cache-path)
* [`stellar cache actionlog`↴](#stellar-cache-actionlog)
* [`stellar cache actionlog ls`↴](#stellar-cache-actionlog-ls)
* [`stellar cache actionlog read`↴](#stellar-cache-actionlog-read)

## `stellar`

Build, deploy, & interact with contracts; set identities to sign with; configure networks; generate keys; and more.

Stellar Docs: https://developers.stellar.org
CLI Full Help Docs: https://github.com/stellar/stellar-cli/tree/main/FULL_HELP_DOCS.md

The easiest way to get started is to generate a new identity:

    stellar keys generate alice

You can use identities with the `--source` flag in other commands later.

Commands that relate to smart contract interactions are organized under the `contract` subcommand. List them:

    stellar contract --help

A Soroban contract has its interface schema types embedded in the binary that gets deployed on-chain, making it possible to dynamically generate a custom CLI for each. The invoke subcommand makes use of this:

    stellar contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- --help

Anything after the `--` double dash (the "slop") is parsed as arguments to the contract-specific CLI, generated on-the-fly from the embedded schema. For the hello world example, with a function called `hello` that takes one string argument `to`, here's how you invoke it:

    stellar contract invoke --id CCR6QKTWZQYW6YUJ7UP7XXZRLWQPFRV6SWBLQS4ZQOSAF4BOUD77OTE2 --source alice --network testnet -- hello --to world


**Usage:** `stellar [OPTIONS] <COMMAND>`

###### **Subcommands:**

* `completion` — Print shell completion code for the specified shell
* `contract` — Tools for smart contract developers
* `events` — Watch the network for contract events
* `keys` — Create and manage identities including keys and addresses
* `xdr` — Decode and encode XDR
* `network` — Start and configure networks
* `version` — Print version information
* `tx` — Sign, Simulate, and Send transactions
* `cache` — Cache for transactions and contract specs

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-f`, `--filter-logs <FILTER_LOGS>` — Filter logs output. To turn on "stellar_cli::log::footprint=debug" or off "=off". Can also use env var `RUST_LOG`
* `-q`, `--quiet` — Do not write logs to stderr including `INFO`

  Possible values: `true`, `false`

* `-v`, `--verbose` — Log DEBUG events

  Possible values: `true`, `false`

* `--very-verbose` — Log DEBUG and TRACE events

  Possible values: `true`, `false`

* `--list` — List installed plugins. E.g. `stellar-hello`

  Possible values: `true`, `false`

* `--no-cache` — Do not cache your simulations and transactions

  Possible values: `true`, `false`




## `stellar completion`

Print shell completion code for the specified shell

Ensure the completion package for your shell is installed,
e.g., bash-completion for bash.

To enable autocomplete in the current bash shell, run:
  source <(stellar completion --shell bash)

To enable autocomplete permanently, run:
  echo "source <(stellar completion --shell bash)" >> ~/.bashrc

**Usage:** `stellar completion --shell <SHELL>`

###### **Options:**

* `--shell <SHELL>` — The shell type

  Possible values: `bash`, `elvish`, `fish`, `powershell`, `zsh`




## `stellar contract`

Tools for smart contract developers

**Usage:** `stellar contract <COMMAND>`

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



## `stellar contract asset`

Utilities to deploy a Stellar Asset Contract or get its id

**Usage:** `stellar contract asset <COMMAND>`

###### **Subcommands:**

* `id` — Get Id of builtin Soroban Asset Contract. Deprecated, use `stellar contract id asset` instead
* `deploy` — Deploy builtin Soroban Asset Contract



## `stellar contract asset id`

Get Id of builtin Soroban Asset Contract. Deprecated, use `stellar contract id asset` instead

**Usage:** `stellar contract asset id [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar contract asset deploy`

Deploy builtin Soroban Asset Contract

**Usage:** `stellar contract asset deploy [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`




## `stellar contract bindings`

Generate code client bindings for a contract

**Usage:** `stellar contract bindings <COMMAND>`

###### **Subcommands:**

* `json` — Generate Json Bindings
* `rust` — Generate Rust bindings
* `typescript` — Generate a TypeScript / JavaScript package



## `stellar contract bindings json`

Generate Json Bindings

**Usage:** `stellar contract bindings json --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary



## `stellar contract bindings rust`

Generate Rust bindings

**Usage:** `stellar contract bindings rust --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary



## `stellar contract bindings typescript`

Generate a TypeScript / JavaScript package

**Usage:** `stellar contract bindings typescript [OPTIONS] --output-dir <OUTPUT_DIR> --contract-id <CONTRACT_ID>`

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



## `stellar contract build`

Build a contract from source

Builds all crates that are referenced by the cargo manifest (Cargo.toml) that have cdylib as their crate-type. Crates are built for the wasm32 target. Unless configured otherwise, crates are built with their default features and with their release profile.

To view the commands that will be executed, without executing them, use the --print-commands-only option.

**Usage:** `stellar contract build [OPTIONS]`

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




## `stellar contract extend`

Extend the time to live ledger of a contract-data ledger entry.

If no keys are specified the contract itself is extended.

**Usage:** `stellar contract extend [OPTIONS] --ledgers-to-extend <LEDGERS_TO_EXTEND> --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

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
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`




## `stellar contract deploy`

Deploy a wasm contract

**Usage:** `stellar contract deploy [OPTIONS] --source-account <SOURCE_ACCOUNT> <--wasm <WASM>|--wasm-hash <WASM_HASH>>`

###### **Options:**

* `--wasm <WASM>` — WASM file to deploy
* `--wasm-hash <WASM_HASH>` — Hash of the already installed/deployed WASM file
* `--salt <SALT>` — Custom salt 32-byte salt for the token id
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `-i`, `--ignore-checks` — Whether to ignore safety checks when deploying contracts

  Default value: `false`

  Possible values: `true`, `false`

* `--alias <ALIAS>` — The alias that will be used to save the contract's id. Whenever used, `--alias` will always overwrite the existing contract id configuration without asking for confirmation



## `stellar contract fetch`

Fetch a contract's Wasm binary

**Usage:** `stellar contract fetch [OPTIONS] --id <CONTRACT_ID>`

###### **Options:**

* `--id <CONTRACT_ID>` — Contract ID to fetch
* `-o`, `--out-file <OUT_FILE>` — Where to write output otherwise stdout is used
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config



## `stellar contract id`

Generate the contract id for a given contract or asset

**Usage:** `stellar contract id <COMMAND>`

###### **Subcommands:**

* `asset` — Deploy builtin Soroban Asset Contract
* `wasm` — Deploy normal Wasm Contract



## `stellar contract id asset`

Deploy builtin Soroban Asset Contract

**Usage:** `stellar contract id asset [OPTIONS] --asset <ASSET> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--asset <ASSET>` — ID of the Stellar classic asset to wrap, e.g. "USDC:G...5"
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar contract id wasm`

Deploy normal Wasm Contract

**Usage:** `stellar contract id wasm [OPTIONS] --salt <SALT> --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--salt <SALT>` — ID of the Soroban contract
* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar contract init`

Initialize a Soroban project with an example contract

**Usage:** `stellar contract init [OPTIONS] <PROJECT_PATH>`

###### **Arguments:**

* `<PROJECT_PATH>`

###### **Options:**

* `-w`, `--with-example <WITH_EXAMPLE>`

  Possible values: `account`, `alloc`, `atomic_multiswap`, `atomic_swap`, `auth`, `cross_contract`, `custom_types`, `deep_contract_auth`, `deployer`, `errors`, `eth_abi`, `events`, `fuzzing`, `increment`, `liquidity_pool`, `logging`, `mint-lock`, `simple_account`, `single_offer`, `timelock`, `token`, `ttl`, `upgradeable_contract`, `workspace`

* `-f`, `--frontend-template <FRONTEND_TEMPLATE>`

  Default value: ``



## `stellar contract inspect`

Inspect a WASM file listing contract functions, meta, etc

**Usage:** `stellar contract inspect [OPTIONS] --wasm <WASM>`

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



## `stellar contract install`

Install a WASM file to the ledger without creating a contract instance

**Usage:** `stellar contract install [OPTIONS] --source-account <SOURCE_ACCOUNT> --wasm <WASM>`

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--wasm <WASM>` — Path to wasm binary
* `-i`, `--ignore-checks` — Whether to ignore safety checks when deploying contracts

  Default value: `false`

  Possible values: `true`, `false`




## `stellar contract invoke`

Invoke a contract function

Generates an "implicit CLI" for the specified contract on-the-fly using the contract's schema, which gets embedded into every Soroban contract. The "slop" in this command, everything after the `--`, gets passed to this implicit CLI. Get in-depth help for a given contract:

stellar contract invoke ... -- --help

**Usage:** `stellar contract invoke [OPTIONS] --id <CONTRACT_ID> --source-account <SOURCE_ACCOUNT> [-- <CONTRACT_FN_AND_ARGS>...]`

###### **Arguments:**

* `<CONTRACT_FN_AND_ARGS>` — Function name as subcommand, then arguments for that function as `--arg-name value`

###### **Options:**

* `--id <CONTRACT_ID>` — Contract ID to invoke
* `--is-view` — View the result simulating and do not sign and submit transaction

  Possible values: `true`, `false`

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`




## `stellar contract optimize`

Optimize a WASM file

**Usage:** `stellar contract optimize [OPTIONS] --wasm <WASM>`

###### **Options:**

* `--wasm <WASM>` — Path to wasm binary
* `--wasm-out <WASM_OUT>` — Path to write the optimized WASM file to (defaults to same location as --wasm with .optimized.wasm suffix)



## `stellar contract read`

Print the current value of a contract-data ledger entry

**Usage:** `stellar contract read [OPTIONS] --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

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
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar contract restore`

Restore an evicted value for a contract-data legder entry.

If no keys are specificed the contract itself is restored.

**Usage:** `stellar contract restore [OPTIONS] --durability <DURABILITY> --source-account <SOURCE_ACCOUNT>`

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
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`

* `--fee <FEE>` — fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm

  Default value: `100`
* `--cost` — Output the cost execution to stderr

  Possible values: `true`, `false`

* `--instructions <INSTRUCTIONS>` — Number of instructions to simulate
* `--build-only` — Build the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`

* `--sim-only` — Simulate the transaction and only write the base64 xdr to stdout

  Possible values: `true`, `false`




## `stellar events`

Watch the network for contract events

**Usage:** `stellar events [OPTIONS]`

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



## `stellar keys`

Create and manage identities including keys and addresses

**Usage:** `stellar keys <COMMAND>`

###### **Subcommands:**

* `add` — Add a new identity (keypair, ledger, macOS keychain)
* `address` — Given an identity return its address (public key)
* `fund` — Fund an identity on a test network
* `generate` — Generate a new identity with a seed phrase, currently 12 words
* `ls` — List identities
* `rm` — Remove an identity
* `show` — Given an identity return its private key



## `stellar keys add`

Add a new identity (keypair, ledger, macOS keychain)

**Usage:** `stellar keys add [OPTIONS] <NAME>`

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



## `stellar keys address`

Given an identity return its address (public key)

**Usage:** `stellar keys address [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default test identity used if not provided

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `stellar keys fund`

Fund an identity on a test network

**Usage:** `stellar keys fund [OPTIONS] <NAME>`

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



## `stellar keys generate`

Generate a new identity with a seed phrase, currently 12 words

**Usage:** `stellar keys generate [OPTIONS] <NAME>`

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



## `stellar keys ls`

List identities

**Usage:** `stellar keys ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long`

  Possible values: `true`, `false`




## `stellar keys rm`

Remove an identity

**Usage:** `stellar keys rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Identity to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `stellar keys show`

Given an identity return its private key

**Usage:** `stellar keys show [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Name of identity to lookup, default is test identity

###### **Options:**

* `--hd-path <HD_PATH>` — If identity is a seed phrase use this hd path, default is 0
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `stellar xdr`

Decode and encode XDR

**Usage:** `stellar xdr [CHANNEL] <COMMAND>`

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




## `stellar xdr types`

View information about types

**Usage:** `stellar xdr types <COMMAND>`

###### **Subcommands:**

* `list` — 
* `schema` — 



## `stellar xdr types list`

**Usage:** `stellar xdr types list [OPTIONS]`

###### **Options:**

* `--output <OUTPUT>`

  Default value: `plain`

  Possible values: `plain`, `json`, `json-formatted`




## `stellar xdr types schema`

**Usage:** `stellar xdr types schema [OPTIONS] --type <TYPE>`

###### **Options:**

* `--type <TYPE>` — XDR type to decode
* `--output <OUTPUT>`

  Default value: `json-schema-draft201909`

  Possible values: `json-schema-draft7`, `json-schema-draft201909`




## `stellar xdr guess`

Guess the XDR type

**Usage:** `stellar xdr guess [OPTIONS] [FILE]`

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



## `stellar xdr decode`

Decode XDR

**Usage:** `stellar xdr decode [OPTIONS] --type <TYPE> [FILES]...`

###### **Arguments:**

* `<FILES>` — Files to decode, or stdin if omitted

###### **Options:**

* `--type <TYPE>` — XDR type to decode
* `--input <INPUT>`

  Default value: `stream-base64`

  Possible values: `single`, `single-base64`, `stream`, `stream-base64`, `stream-framed`

* `--output <OUTPUT>`

  Default value: `json`

  Possible values: `json`, `json-formatted`, `rust-debug`, `rust-debug-formatted`




## `stellar xdr encode`

Encode XDR

**Usage:** `stellar xdr encode [OPTIONS] --type <TYPE> [FILES]...`

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




## `stellar xdr version`

Print version information

**Usage:** `stellar xdr version`



## `stellar network`

Start and configure networks

**Usage:** `stellar network <COMMAND>`

###### **Subcommands:**

* `add` — Add a new network
* `rm` — Remove a network
* `ls` — List networks
* `start` — ⚠️ Deprecated: use `stellar container start` instead
* `stop` — ⚠️ Deprecated: use `stellar container stop` instead
* `container` — Commands to start, stop and get logs for a quickstart container



## `stellar network add`

Add a new network

**Usage:** `stellar network add [OPTIONS] --rpc-url <RPC_URL> --network-passphrase <NETWORK_PASSPHRASE> <NAME>`

###### **Arguments:**

* `<NAME>` — Name of network

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `stellar network rm`

Remove a network

**Usage:** `stellar network rm [OPTIONS] <NAME>`

###### **Arguments:**

* `<NAME>` — Network to remove

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."



## `stellar network ls`

List networks

**Usage:** `stellar network ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long` — Get more info about the networks

  Possible values: `true`, `false`




## `stellar network start`

⚠️ Deprecated: use `stellar container start` instead

Start network

Start a container running a Stellar node, RPC, API, and friendbot (faucet).

stellar network start <NETWORK> [OPTIONS]

By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command: docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc

**Usage:** `stellar network start [OPTIONS] <NETWORK>`

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



## `stellar network stop`

⚠️ Deprecated: use `stellar container stop` instead

Stop a network started with `network start`. For example, if you ran `stellar network start local`, you can use `stellar network stop local` to stop it.

**Usage:** `stellar network stop [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to stop

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock



## `stellar network container`

Commands to start, stop and get logs for a quickstart container

**Usage:** `stellar network container <COMMAND>`

###### **Subcommands:**

* `logs` — Tail logs of a running network container
* `start` — Start network
* `stop` — Stop a network started with `network container start`. For example, if you ran `network container start local`, you can use `network container stop local` to stop it



## `stellar network container logs`

Tail logs of a running network container

**Usage:** `stellar network container logs [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to tail

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock



## `stellar network container start`

Start network

Start a container running a Stellar node, RPC, API, and friendbot (faucet).

stellar network start <NETWORK> [OPTIONS]

By default, when starting a testnet container, without any optional arguments, it will run the equivalent of the following docker command: docker run --rm -p 8000:8000 --name stellar stellar/quickstart:testing --testnet --enable-soroban-rpc

**Usage:** `stellar network container start [OPTIONS] <NETWORK>`

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



## `stellar network container stop`

Stop a network started with `network container start`. For example, if you ran `network container start local`, you can use `network container stop local` to stop it

**Usage:** `stellar network container stop [OPTIONS] <NETWORK>`

###### **Arguments:**

* `<NETWORK>` — Network to stop

  Possible values: `local`, `testnet`, `futurenet`, `pubnet`


###### **Options:**

* `-d`, `--docker-host <DOCKER_HOST>` — Optional argument to override the default docker host. This is useful when you are using a non-standard docker host path for your Docker-compatible container runtime, e.g. Docker Desktop defaults to $HOME/.docker/run/docker.sock instead of /var/run/docker.sock



## `stellar version`

Print version information

**Usage:** `stellar version`



## `stellar tx`

Sign, Simulate, and Send transactions

**Usage:** `stellar tx <COMMAND>`

###### **Subcommands:**

* `simulate` — Simulate a transaction envelope from stdin
* `sign` — Sign a transaction
* `send` — Send a transaction envelope to the network



## `stellar tx simulate`

Simulate a transaction envelope from stdin

**Usage:** `stellar tx simulate [OPTIONS] --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar tx sign`

Sign a transaction

**Usage:** `stellar tx sign [OPTIONS] --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar tx send`

Send a transaction envelope to the network

**Usage:** `stellar tx send [OPTIONS] --source-account <SOURCE_ACCOUNT>`

###### **Options:**

* `--rpc-url <RPC_URL>` — RPC server endpoint
* `--network-passphrase <NETWORK_PASSPHRASE>` — Network passphrase to sign the transaction sent to the rpc server
* `--network <NETWORK>` — Name of network to use from config
* `--source-account <SOURCE_ACCOUNT>` — Account that signs the final transaction. Alias `source`. Can be an identity (--source alice), a secret key (--source SC36…), or a seed phrase (--source "kite urban…")
* `--hd-path <HD_PATH>` — If using a seed phrase, which hierarchical deterministic path to use, e.g. `m/44'/148'/{hd_path}`. Example: `--hd-path 1`. Default: `0`
* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `--check` — Check with user before signature. Eventually this will be replaced with `--yes`, which does the opposite and will force a check without --yes

  Possible values: `true`, `false`




## `stellar cache`

Cache for transactions and contract specs

**Usage:** `stellar cache <COMMAND>`

###### **Subcommands:**

* `clean` — Delete the cache
* `path` — Show the location of the cache
* `actionlog` — Access details about cached actions like transactions, and simulations. (Experimental. May see breaking changes at any time.)



## `stellar cache clean`

Delete the cache

**Usage:** `stellar cache clean`



## `stellar cache path`

Show the location of the cache

**Usage:** `stellar cache path`



## `stellar cache actionlog`

Access details about cached actions like transactions, and simulations. (Experimental. May see breaking changes at any time.)

**Usage:** `stellar cache actionlog <COMMAND>`

###### **Subcommands:**

* `ls` — List cached actions (transactions, simulations)
* `read` — Read cached action



## `stellar cache actionlog ls`

List cached actions (transactions, simulations)

**Usage:** `stellar cache actionlog ls [OPTIONS]`

###### **Options:**

* `--global` — Use global config

  Possible values: `true`, `false`

* `--config-dir <CONFIG_DIR>` — Location of config directory, default is "."
* `-l`, `--long`

  Possible values: `true`, `false`




## `stellar cache actionlog read`

Read cached action

**Usage:** `stellar cache actionlog read --id <ID>`

###### **Options:**

* `--id <ID>` — ID of the cache entry



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
