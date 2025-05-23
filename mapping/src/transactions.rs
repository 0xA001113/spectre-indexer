use bigdecimal::ToPrimitive;
use spectre_rpc_core::RpcTransaction;

use spectre_database::models::address_transaction::AddressTransaction as SqlAddressTransaction;
use spectre_database::models::block_transaction::BlockTransaction as SqlBlockTransaction;
use spectre_database::models::script_transaction::ScriptTransaction as SqlScriptTransaction;
use spectre_database::models::transaction::Transaction as SqlTransaction;
use spectre_database::models::transaction_input::TransactionInput as SqlTransactionInput;
use spectre_database::models::transaction_output::TransactionOutput as SqlTransactionOutput;

pub fn map_transaction(
    subnetwork_key: i32,
    transaction: &RpcTransaction,
    include_subnetwork_id: bool,
    include_hash: bool,
    include_mass: bool,
    include_payload: bool,
    include_block_time: bool,
) -> SqlTransaction {
    let verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    SqlTransaction {
        transaction_id: verbose_data.transaction_id.into(),
        subnetwork_id: include_subnetwork_id.then_some(subnetwork_key),
        hash: include_hash.then_some(verbose_data.hash.into()),
        mass: (include_mass && verbose_data.compute_mass != 0).then_some(verbose_data.compute_mass.to_i32().unwrap()),
        payload: (include_payload && !transaction.payload.is_empty()).then_some(transaction.payload.to_owned()),
        block_time: include_block_time.then_some(verbose_data.block_time.to_i64().unwrap()),
    }
}

pub fn map_block_transaction(transaction: &RpcTransaction) -> SqlBlockTransaction {
    let verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    SqlBlockTransaction { block_hash: verbose_data.block_hash.into(), transaction_id: verbose_data.transaction_id.into() }
}

pub fn map_transaction_inputs(
    transaction: &RpcTransaction,
    include_previous_outpoint: bool,
    include_signature_script: bool,
    include_sig_op_count: bool,
    include_block_time: bool,
) -> Vec<SqlTransactionInput> {
    let tx_verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    transaction
        .inputs
        .iter()
        .enumerate()
        .map(|(i, input)| SqlTransactionInput {
            transaction_id: tx_verbose_data.transaction_id.into(),
            index: i.to_i16().unwrap(),
            previous_outpoint_hash: include_previous_outpoint.then_some(input.previous_outpoint.transaction_id.into()),
            previous_outpoint_index: include_previous_outpoint.then_some(input.previous_outpoint.index.to_i16().unwrap()),
            signature_script: include_signature_script.then_some(input.signature_script.clone()),
            sig_op_count: include_sig_op_count.then_some(input.sig_op_count as i16),
            block_time: include_block_time.then_some(tx_verbose_data.block_time.to_i64().unwrap()),
            previous_outpoint_script: None,
            previous_outpoint_amount: None,
        })
        .collect::<Vec<SqlTransactionInput>>()
}

pub fn map_transaction_outputs(
    transaction: &RpcTransaction,
    include_amount: bool,
    include_script_public_key: bool,
    include_script_public_key_address: bool,
    include_block_time: bool,
) -> Vec<SqlTransactionOutput> {
    let tx_verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    transaction
        .outputs
        .iter()
        .enumerate()
        .map(|(i, output)| {
            let verbose_data = output.verbose_data.as_ref().expect("Transaction output verbose_data is missing");
            SqlTransactionOutput {
                transaction_id: tx_verbose_data.transaction_id.into(),
                index: i.to_i16().expect("Tx output index is too large for i16"),
                amount: include_amount.then_some(output.value.to_i64().expect("Tx output amount is too large for i64")),
                script_public_key: include_script_public_key.then_some(output.script_public_key.script().to_vec()),
                script_public_key_address: include_script_public_key_address
                    .then_some(verbose_data.script_public_key_address.payload_to_string()),
                block_time: include_block_time.then_some(tx_verbose_data.block_time.to_i64().unwrap()),
            }
        })
        .collect::<Vec<SqlTransactionOutput>>()
}

pub fn map_transaction_outputs_address(transaction: &RpcTransaction) -> Vec<SqlAddressTransaction> {
    let tx_verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    transaction
        .outputs
        .iter()
        .map(|output| {
            let verbose_data = output.verbose_data.as_ref().expect("Transaction output verbose_data is missing");
            SqlAddressTransaction {
                address: verbose_data.script_public_key_address.payload_to_string(),
                transaction_id: tx_verbose_data.transaction_id.into(),
                block_time: tx_verbose_data.block_time.to_i64().unwrap(),
            }
        })
        .collect::<Vec<SqlAddressTransaction>>()
}

pub fn map_transaction_outputs_script(transaction: &RpcTransaction) -> Vec<SqlScriptTransaction> {
    let tx_verbose_data = transaction.verbose_data.as_ref().expect("Transaction verbose_data is missing");
    transaction
        .outputs
        .iter()
        .map(|output| SqlScriptTransaction {
            script_public_key: output.script_public_key.script().to_vec(),
            transaction_id: tx_verbose_data.transaction_id.into(),
            block_time: tx_verbose_data.block_time.to_i64().unwrap(),
        })
        .collect::<Vec<SqlScriptTransaction>>()
}
