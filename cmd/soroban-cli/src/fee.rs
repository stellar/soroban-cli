use clap::arg;

use soroban_env_host::xdr;
use soroban_rpc::Assembled;

use crate::commands::{txn_result::TxnResult, HEADING_RPC};

#[derive(Debug, clap::Args, Clone)]
#[group(skip)]
pub struct Args {
    /// fee amount for transaction, in stroops. 1 stroop = 0.0000001 xlm
    #[arg(long, default_value = "100", env = "SOROBAN_FEE", help_heading = HEADING_RPC)]
    pub fee: u32,
    /// Output the cost execution to stderr
    #[arg(long = "cost", help_heading = HEADING_RPC)]
    pub cost: bool,
    /// Number of instructions to simulate
    #[arg(long, help_heading = HEADING_RPC)]
    pub instructions: Option<u32>,
    /// Build the transaction only write the base64 xdr to stdout
    #[arg(long, help_heading = HEADING_RPC)]
    pub build_only: bool,
    /// Simulation the transaction only write the base64 xdr to stdout
    #[arg(long, help_heading = HEADING_RPC, conflicts_with = "build_only")]
    pub sim_only: bool,
}

impl Args {
    pub fn apply_to_assembled_txn(
        &self,
        txn: Assembled,
    ) -> Result<TxnResult<Assembled>, xdr::Error> {
        let simulated_txn = if let Some(instructions) = self.instructions {
            txn.set_max_instructions(instructions)
        } else {
            add_padding_to_instructions(txn)
        };
        if self.sim_only {
            // TODO: Move into callers.
            Ok(TxnResult::Txn(simulated_txn.transaction().clone()))
        } else {
            Ok(TxnResult::Res(simulated_txn))
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
