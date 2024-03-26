pub(crate) use soroban_rpc::*;
use soroban_sdk::xdr;

pub(crate) fn preview_txn(txn: &xdr::Transaction) -> String {
    let source_account = txn.source_account.to_string();
    let fee = txn.fee;
    let operations = txn
        .operations
        .iter()
        .map(preview_operation)
        .collect::<Vec<_>>()
        .join("\n");
    format!("source_account: {source_account}\nfee: {fee}\noperations:\n{operations}")
}

pub(crate) fn preview_operation(op: &xdr::Operation) -> String {
    use soroban_sdk::xdr::OperationBody;
    let _source_account = op.source_account.as_ref().map(ToString::to_string);

    match &op.body {
        OperationBody::CreateAccount(_) => todo!(),
        OperationBody::Payment(_) => todo!(),
        OperationBody::PathPaymentStrictReceive(_) => todo!(),
        OperationBody::ManageSellOffer(_) => todo!(),
        OperationBody::CreatePassiveSellOffer(_) => todo!(),
        OperationBody::SetOptions(_) => todo!(),
        OperationBody::ChangeTrust(_) => todo!(),
        OperationBody::AllowTrust(_) => todo!(),
        OperationBody::AccountMerge(_) => todo!(),
        OperationBody::Inflation => todo!(),
        OperationBody::ManageData(_) => todo!(),
        OperationBody::BumpSequence(_) => todo!(),
        OperationBody::ManageBuyOffer(_) => todo!(),
        OperationBody::PathPaymentStrictSend(_) => todo!(),
        OperationBody::CreateClaimableBalance(_) => todo!(),
        OperationBody::ClaimClaimableBalance(_) => todo!(),
        OperationBody::BeginSponsoringFutureReserves(_) => todo!(),
        OperationBody::EndSponsoringFutureReserves => todo!(),
        OperationBody::RevokeSponsorship(_) => todo!(),
        OperationBody::Clawback(_) => todo!(),
        OperationBody::ClawbackClaimableBalance(_) => todo!(),
        OperationBody::SetTrustLineFlags(_) => todo!(),
        OperationBody::LiquidityPoolDeposit(_) => todo!(),
        OperationBody::LiquidityPoolWithdraw(_) => todo!(),
        OperationBody::InvokeHostFunction(op) => preview_invoke(op),
        OperationBody::ExtendFootprintTtl(_) => todo!(),
        OperationBody::RestoreFootprint(_) => todo!(),
    }
}

fn preview_invoke(invoke: &xdr::InvokeHostFunctionOp) -> String {
    let host_function = preview_host_function(&invoke.host_function);
    let auth = invoke
        .auth
        .iter()
        .map(preview_auth_entry)
        .collect::<Vec<_>>()
        .join("\n");
    format!("host_function:\n{host_function}\nauth:\n{auth}")
}

fn preview_host_function(host_function: &xdr::HostFunction) -> String {
    match host_function {
        xdr::HostFunction::InvokeContract(args) => preview_invoke_contract_args(args, 1),
        xdr::HostFunction::CreateContract(_) => todo!(),
        xdr::HostFunction::UploadContractWasm(_) => todo!(),
    }
}

fn preview_invoke_contract_args(args: &xdr::InvokeContractArgs, indention: u8) -> String {
    let xdr::InvokeContractArgs {
        contract_address,
        function_name,
        args,
    } = args;
    let contract_id = match contract_address {
        xdr::ScAddress::Account(_) => todo!(),
        xdr::ScAddress::Contract(_) => strkey_from_sc_address(contract_address),
    };
    let function_name = function_name.to_string();
    let args = args.iter().map(|x| format!("{x:?}")).collect::<Vec<_>>();
    let indent = " ".repeat(indention as usize);
    format!("{indent}contract_id: {contract_id}\n{indent}function_name: {function_name}\n{indent}args:\n{indent}{args:?}")
}

fn preview_auth_entry(entry: &xdr::SorobanAuthorizationEntry) -> String {
    let xdr::SorobanAuthorizationEntry {
        credentials,
        root_invocation,
    } = entry;
    let function = preview_authorized_function(root_invocation, 1);
    let account_id = if let xdr::SorobanCredentials::Address(account_id) = &credentials {
        strkey_from_sc_address(&account_id.address)
    } else {
        "SourceAccount".to_string()
    };

    format!("credentials: {account_id}\n root_invocation:\n{function}")
}

fn preview_authorized_function(
    function: &xdr::SorobanAuthorizedInvocation,
    indention: u8,
) -> String {
    let contract_args = match &function.function {
        xdr::SorobanAuthorizedFunction::ContractFn(args) => {
            preview_invoke_contract_args(args, indention)
        }
        xdr::SorobanAuthorizedFunction::CreateContractHostFn(_) => todo!(),
    };
    let mut sub_invocations = String::new();
    let preview_authorized_function = |x| preview_authorized_function(x, indention + 2);
    if !function.sub_invocations.is_empty() {
        sub_invocations = function
            .sub_invocations
            .iter()
            .map(preview_authorized_function)
            .collect::<Vec<_>>()
            .join("\n");
    }
    let indent = " ".repeat(indention as usize);
    format!("{indent}contract_args: {contract_args}\n{indent}sub_invocations:\n{sub_invocations}")
}

fn strkey_from_sc_address(addr: &xdr::ScAddress) -> String {
    match addr {
        xdr::ScAddress::Account(xdr::AccountId(xdr::PublicKey::PublicKeyTypeEd25519(
            xdr::Uint256(bytes),
        ))) => stellar_strkey::ed25519::PublicKey(*bytes).to_string(),
        xdr::ScAddress::Contract(contract) => stellar_strkey::Contract(contract.0).to_string(),
    }
}
