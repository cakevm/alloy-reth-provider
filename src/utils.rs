#[cfg(not(feature = "optimism"))]
use alloy_network::ReceiptResponse;
#[cfg(not(feature = "optimism"))]
use alloy_primitives::Log;
use alloy_rpc_types_eth::Block as RpcBlock;
#[cfg(not(feature = "optimism"))]
use alloy_rpc_types_eth::TransactionReceipt;
#[cfg(feature = "optimism")]
use op_alloy_rpc_types::OpTransactionReceipt;
use reth_ethereum_primitives::{Receipt, TransactionSigned};
use reth_primitives::{Block, BlockBody};
use reth_primitives_traits::{RecoveredBlock, SealedBlock, SealedHeader};

pub fn rpc_block_to_recovered_block(block: RpcBlock) -> eyre::Result<RecoveredBlock<Block>> {
    let block = block.map_transactions(|tx| tx.into_inner().into());
    let block_body = BlockBody::<TransactionSigned> {
        transactions: block.transactions.into_transactions().collect(),
        ommers: vec![],
        withdrawals: block.withdrawals,
    };
    let sealed_header = SealedHeader::new(block.header.inner, block.header.hash);
    let sealed_block = SealedBlock::from_sealed_parts(sealed_header, block_body);
    let recovered_block = RecoveredBlock::try_recover_sealed(sealed_block)?;
    Ok(recovered_block)
}

#[cfg(not(feature = "optimism"))]
pub fn rpc_receipt_to_receipt(receipt: TransactionReceipt) -> Receipt {
    Receipt {
        tx_type: receipt.transaction_type(),
        success: receipt.status(),
        cumulative_gas_used: receipt.cumulative_gas_used(),
        logs: receipt.logs().iter().map(|log| Log { address: log.address(), data: log.data().clone() }).collect(),
    }
}
#[cfg(feature = "optimism")]
pub fn rpc_receipt_to_receipt(_receipt: OpTransactionReceipt) -> Receipt {
    todo!()
}
