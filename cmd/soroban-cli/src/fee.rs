use clap::arg;

use soroban_env_host::xdr;
use soroban_rpc::Assembled;

use crate::commands::HEADING_NETWORK;

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub struct Args {
    /// fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm
    #[arg(long, default_value = "100", env = "STELLAR_FEE", help_heading = HEADING_NETWORK)]
    pub fee: u32,
    /// Output the cost execution to stderr
    #[arg(long = "cost", help_heading = HEADING_NETWORK)]
    pub cost: bool,
    /// Number of instructions to simulate
    #[arg(long, help_heading = HEADING_NETWORK)]
    pub instructions: Option<u32>,
    /// Build the transaction only write the base64 xdr to stdout
    #[arg(long, help_heading = HEADING_NETWORK)]
    pub build_only: bool,
    /// Simulation the transaction only write the base64 xdr to stdout
    #[arg(long, help_heading = HEADING_NETWORK, conflicts_with = "build_only")]
    pub sim_only: bool,
}

impl Args {
    pub fn apply_to_assembled_txn(&self, txn: Assembled) -> Assembled {
        if let Some(instructions) = self.instructions {
            txn.set_max_instructions(instructions)
        } else {
            add_padding_to_instructions(txn)
        }
    }
}

pub fn add_padding_to_instructions(txn: Assembled) -> Assembled {
    let xdr::TransactionExt::V1(xdr::SorobanTransactionData {
        resources: xdr::SorobanResources { instructions, .. },
        ..
    }) = txn.transaction().ext
    else {
        return txn;
    };
    // Start with 150%
    let instructions = (instructions.checked_mul(150 / 100)).unwrap_or(instructions);
    txn.set_max_instructions(instructions)
}

impl Default for Args {
    fn default() -> Self {
        Self {
            fee: 100,
            cost: false,
            instructions: None,
            build_only: false,
            sim_only: false,
        }
    }
}
