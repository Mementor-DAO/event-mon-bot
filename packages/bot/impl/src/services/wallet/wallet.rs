use candid::Principal;
use ic_ledger_types::{
    account_balance, AccountBalanceArgs, AccountIdentifier, 
    Memo, Subaccount, Timestamp, Tokens, TransferArgs, 
    DEFAULT_FEE, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID
};

use crate::{storage::user::UserStorage, types::user::UserTransaction};

pub struct WalletService;

impl WalletService {
    fn account_id(
        user_id: Principal
    ) -> AccountIdentifier {
        AccountIdentifier::new(
            &ic_cdk::id(), 
            &Subaccount::from(user_id)
        )
    }
    
    pub fn address_of(
        user_id: Principal
    ) -> String {
        let acc_id = AccountIdentifier::new(
            &ic_cdk::id(), 
            &Subaccount::from(user_id)
        );
        acc_id.to_hex()
    }
    
    pub async fn balance_of(
        user_id: Principal
    ) -> Result<u64, String> {
        let acc_id = Self::account_id(user_id);
        let icp = account_balance(
            MAINNET_LEDGER_CANISTER_ID, 
            AccountBalanceArgs {
                account: acc_id,
            }
        ).await
            .map_err(|e| e.1)?;

        Ok(icp.e8s())
    }

    pub async fn transfer_hex(
        user_id: Principal, 
        to: Option<String>, 
        amount: u64
    ) -> Result<(u64, String), String> {
        let to = if let Some(to) = to {
            AccountIdentifier::from_hex(&to).unwrap()
        }
        else {
            AccountIdentifier::new(&user_id, &DEFAULT_SUBACCOUNT)
        };
        
        Self::transfer(Some(user_id), to, amount).await
    }

    pub async fn transfer(
        from: Option<Principal>, 
        to: AccountIdentifier, 
        amount: u64
    ) -> Result<(u64, String), String> {
        let now = ic_cdk::api::time();
        let block_num = ic_ledger_types::transfer(
            MAINNET_LEDGER_CANISTER_ID, 
            TransferArgs { 
                from_subaccount: if let Some(from) = from {
                    Some(from.into())
                } 
                else {
                    None
                },  
                to, 
                fee: DEFAULT_FEE.into(), 
                created_at_time: Some(Timestamp{timestamp_nanos: now}), 
                memo: Memo(0x4E4E), 
                amount: Tokens::from_e8s(amount),
            }
        ).await
            .map_err(|e| e.1)?
            .map_err(|e| e.to_string())?;

        if let Some(from) = from {
            let mut user = UserStorage::load(&from);
            user.txs.push(UserTransaction::IcpWithdraw { 
                amount,
                to, 
                block_num, 
                timestamp: (now / 1_000_000_000) as _,
            });
            UserStorage::save(from, user);
        }

        Ok((
            block_num,
            to.to_hex()
        ))
    }
}