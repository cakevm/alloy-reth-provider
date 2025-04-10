use crate::AlloyRethStateProvider;
use alloy_consensus::constants::KECCAK_EMPTY;
use alloy_network::Network;
use alloy_primitives::Address;
use alloy_provider::Provider;
use reth_errors::{ProviderError, ProviderResult};
use reth_primitives::{Account, Bytecode};
use reth_provider::errors::any::AnyError;
use reth_provider::AccountReader;
use revm_database::DatabaseRef;

impl<N, P> AccountReader for AlloyRethStateProvider<N, P>
where
    N: Network,
    P: Provider<N> + Clone,
{
    fn basic_account(&self, address: &Address) -> ProviderResult<Option<Account>> {
        match self.alloy_db.basic_ref(*address) {
            Ok(Some(account)) => {
                let bytecode_hash = match account.code {
                    Some(code) => {
                        if account.code_hash == KECCAK_EMPTY {
                            None
                        } else {
                            self.bytecode.write().insert(account.code_hash, Bytecode::new_raw(code.bytes()));
                            Some(account.code_hash)
                        }
                    }
                    None => None,
                };

                Ok(Some(Account { nonce: account.nonce, balance: account.balance, bytecode_hash }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ProviderError::Other(AnyError::new(e))),
        }
    }
}
