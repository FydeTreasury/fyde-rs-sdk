use std::{collections::HashMap, sync::Arc, vec};

use ethers::{
    prelude::LogMeta,
    providers::{Http, Middleware, Provider},
    types::{Address, H256, U256},
};
use serde::Serialize;

use crate::{
    errors::FydeError, AddressList, Chain, LiquidVaultContract, LiquidVaultContractEvents,
    RelayerContract, RelayerContractEvents,
};

pub struct ProtocolHistory {
    client: Arc<Provider<Http>>,
    liquid_vault: LiquidVaultContract<Provider<Http>>,
    relayer: RelayerContract<Provider<Http>>,
}

#[derive(Debug)]
struct RequestData {
    tx_hash: H256,
    block_number: u32,
    timestamp: u64,
    request_id: u32,
    requestor: Address,
    asset_in: Vec<Address>,
    asset_out: Vec<Address>,
    amount_in: Vec<U256>,
    amount_out: Vec<U256>,
    keep_gov_rights: bool,
}

struct MetaFromBlock {
    tx_hash: H256,
    block_number: u32,
    timestamp: u64,
    from: Address,
}

impl From<(crate::relayer_contract::DepositFilter, MetaFromBlock)> for RequestData {
    fn from(event: (crate::relayer_contract::DepositFilter, MetaFromBlock)) -> Self {
        Self {
            tx_hash: event.1.tx_hash,
            block_number: event.1.block_number,
            timestamp: event.1.timestamp,
            request_id: event.0.request_id,
            requestor: event.1.from,
            asset_in: event.0.request.asset_in,
            asset_out: event.0.request.asset_out,
            amount_in: event.0.request.amount_in,
            amount_out: event.0.request.amount_out,
            keep_gov_rights: event.0.request.keep_gov_rights,
        }
    }
}

impl From<(crate::relayer_contract::WithdrawFilter, MetaFromBlock)> for RequestData {
    fn from(event: (crate::relayer_contract::WithdrawFilter, MetaFromBlock)) -> Self {
        Self {
            tx_hash: event.1.tx_hash,
            block_number: event.1.block_number,
            timestamp: event.1.timestamp,
            request_id: event.0.request_id,
            requestor: event.1.from,
            asset_in: event.0.request.asset_in,
            asset_out: event.0.request.asset_out,
            amount_in: event.0.request.amount_in,
            amount_out: event.0.request.amount_out,
            keep_gov_rights: event.0.request.keep_gov_rights,
        }
    }
}

impl From<(crate::relayer_contract::SwapFilter, MetaFromBlock)> for RequestData {
    fn from(event: (crate::relayer_contract::SwapFilter, MetaFromBlock)) -> Self {
        Self {
            tx_hash: event.1.tx_hash,
            block_number: event.1.block_number,
            timestamp: event.1.timestamp,
            request_id: event.0.request_id,
            requestor: event.1.from,
            asset_in: event.0.request.asset_in,
            asset_out: event.0.request.asset_out,
            amount_in: event.0.request.amount_in,
            amount_out: event.0.request.amount_out,
            keep_gov_rights: event.0.request.keep_gov_rights,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Deposit {
    pub tx_hash: H256,
    pub block_number: u32,
    pub timestamp: u64,
    pub request_id: u32,
    pub user: Address,
    pub asset_in: Vec<Address>,
    pub amount_in: Vec<U256>,
    pub keep_gov_rights: bool,
    pub minted_at_trsy_price: U256,
    pub usd_value_deposited: U256,
    pub trsy_minted: U256,
}

impl From<(RequestData, crate::liquid_vault_contract::DepositFilter)> for Deposit {
    fn from(data: (RequestData, crate::liquid_vault_contract::DepositFilter)) -> Self {
        if data.0.request_id != data.1.request_id {
            panic!("Request id mismatch");
        }
        Self {
            tx_hash: data.0.tx_hash,
            block_number: data.0.block_number,
            timestamp: data.0.timestamp,
            request_id: data.0.request_id,
            user: data.0.requestor,
            asset_in: data.0.asset_in,
            amount_in: data.0.amount_in,
            keep_gov_rights: data.0.keep_gov_rights,
            minted_at_trsy_price: data.1.trsy_price,
            usd_value_deposited: data.1.usd_deposit_value,
            trsy_minted: data.1.trsy_minted,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Withdraw {
    pub tx_hash: H256,
    pub block_number: u32,
    pub timestamp: u64,
    pub request_id: u32,
    pub user: Address,
    pub asset_out: Vec<Address>,
    pub amount_out: Vec<U256>,
    pub burned_at_trsy_price: U256,
    pub usd_value_withdrawn: U256,
    pub trsy_burned: U256,
}

impl From<(RequestData, crate::liquid_vault_contract::WithdrawFilter)> for Withdraw {
    fn from(data: (RequestData, crate::liquid_vault_contract::WithdrawFilter)) -> Self {
        if data.0.request_id != data.1.request_id {
            panic!("Request id mismatch");
        }
        Self {
            tx_hash: data.0.tx_hash,
            block_number: data.0.block_number,
            timestamp: data.0.timestamp,
            request_id: data.0.request_id,
            user: data.0.requestor,
            asset_out: data.0.asset_out,
            amount_out: data.0.amount_out,
            burned_at_trsy_price: data.1.trsy_price,
            usd_value_withdrawn: data.1.usd_withdraw_value,
            trsy_burned: data.1.trsy_burned,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Swap {
    pub tx_hash: H256,
    pub block_number: u32,
    pub timestamp: u64,
    pub request_id: u32,
    pub user: Address,
    pub asset_in: Address,
    pub asset_out: Address,
    pub amount_in: U256,
    pub amount_out: U256,
}

impl From<(RequestData, crate::liquid_vault_contract::SwapFilter)> for Swap {
    fn from(data: (RequestData, crate::liquid_vault_contract::SwapFilter)) -> Self {
        if data.0.request_id != data.1.request_id {
            panic!("Request id mismatch");
        }
        Self {
            tx_hash: data.0.tx_hash,
            block_number: data.0.block_number,
            timestamp: data.0.timestamp,
            request_id: data.0.request_id,
            user: data.0.requestor,
            asset_in: data.0.asset_in[0],
            asset_out: data.0.asset_out[0],
            amount_in: data.0.amount_in[0],
            amount_out: data.1.amount_out,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum UserAction {
    Deposit {
        tx_hash: H256,
        block_number: u32,
        timestamp: u64,
        request_id: u32,
        user: Address,
        asset_in: Vec<Address>,
        amount_in: Vec<U256>,
        keep_gov_rights: bool,
        minted_at_trsy_price: U256,
        usd_value_deposited: U256,
        trsy_minted: U256,
    },
    Withdraw {
        tx_hash: H256,
        block_number: u32,
        timestamp: u64,
        request_id: u32,
        user: Address,
        asset_out: Vec<Address>,
        amount_out: Vec<U256>,
        burned_at_trsy_price: U256,
        usd_value_withdrawn: U256,
        trsy_burned: U256,
    },
    Swap {
        tx_hash: H256,
        block_number: u32,
        timestamp: u64,
        request_id: u32,
        user: Address,
        asset_in: Address,
        asset_out: Address,
        amount_in: U256,
        amount_out: U256,
    },
}

#[derive(Clone)]
enum FydeEvents {
    Deposit(crate::liquid_vault_contract::DepositFilter),
    Withdraw(crate::liquid_vault_contract::WithdrawFilter),
    Swap(crate::liquid_vault_contract::SwapFilter),
}

impl ProtocolHistory {
    pub fn new(client: Arc<Provider<Http>>, chain: Chain) -> Self {
        let address_list: AddressList = AddressList::new(&chain);

        Self {
            client: client.clone(),
            liquid_vault: LiquidVaultContract::new(address_list.liquid_vault, client.clone()),
            relayer: RelayerContract::new(address_list.relayer, client),
        }
    }

    pub async fn get_data(&self, from_block: u64) -> Result<Vec<UserAction>, FydeError> {
        let events: Vec<(RelayerContractEvents, LogMeta)> = self
            .relayer
            .events()
            .from_block(from_block)
            .query_with_meta()
            .await?;

        let mut requests_data = vec![];

        for event in events {
            match event {
                (RelayerContractEvents::DepositFilter(ev), meta) => {
                    let tx = meta.transaction_hash;
                    let tx_data = self.client.get_transaction(tx).await?.unwrap();
                    let block = self
                        .client
                        .get_block(tx_data.block_number.unwrap())
                        .await?
                        .unwrap();
                    let block_meta = MetaFromBlock {
                        tx_hash: tx,
                        block_number: block.number.unwrap().as_u32(),
                        timestamp: block.timestamp.as_u64(),
                        from: tx_data.from,
                    };
                    requests_data.push(RequestData::from((ev, block_meta)))
                }
                (RelayerContractEvents::WithdrawFilter(ev), meta) => {
                    let tx = meta.transaction_hash;
                    let tx_data = self.client.get_transaction(tx).await?.unwrap();
                    let block = self
                        .client
                        .get_block(tx_data.block_number.unwrap())
                        .await?
                        .unwrap();
                    let block_meta = MetaFromBlock {
                        tx_hash: tx,
                        block_number: block.number.unwrap().as_u32(),
                        timestamp: block.timestamp.as_u64(),
                        from: tx_data.from,
                    };
                    requests_data.push(RequestData::from((ev, block_meta)))
                }
                (RelayerContractEvents::SwapFilter(ev), meta) => {
                    let tx = meta.transaction_hash;
                    let tx_data = self.client.get_transaction(tx).await?.unwrap();
                    let block = self
                        .client
                        .get_block(tx_data.block_number.unwrap())
                        .await?
                        .unwrap();
                    let block_meta = MetaFromBlock {
                        tx_hash: tx,
                        block_number: block.number.unwrap().as_u32(),
                        timestamp: block.timestamp.as_u64(),
                        from: tx_data.from,
                    };
                    requests_data.push(RequestData::from((ev, block_meta)))
                }
                _ => {}
            }
        }

        let events = self
            .liquid_vault
            .events()
            .from_block(from_block)
            .query()
            .await?;
        let mut fyde_events = HashMap::new();
        for event in events {
            match event {
                LiquidVaultContractEvents::DepositFilter(ev) => {
                    fyde_events.insert(ev.request_id, FydeEvents::Deposit(ev));
                }
                LiquidVaultContractEvents::WithdrawFilter(ev) => {
                    fyde_events.insert(ev.request_id, FydeEvents::Withdraw(ev));
                }
                LiquidVaultContractEvents::SwapFilter(ev) => {
                    fyde_events.insert(ev.request_id, FydeEvents::Swap(ev));
                }
                _ => {}
            }
        }

        let mut user_actions = vec![];
        for request in requests_data {
            let fyde_event = fyde_events.get(&request.request_id).unwrap().clone();
            match fyde_event {
                FydeEvents::Deposit(ev) => user_actions.push(UserAction::Deposit {
                    tx_hash: request.tx_hash,
                    block_number: request.block_number,
                    timestamp: request.timestamp,
                    request_id: request.request_id,
                    user: request.requestor,
                    asset_in: request.asset_in,
                    amount_in: request.amount_in,
                    keep_gov_rights: request.keep_gov_rights,
                    minted_at_trsy_price: ev.trsy_price,
                    usd_value_deposited: ev.usd_deposit_value,
                    trsy_minted: ev.trsy_minted,
                }),
                FydeEvents::Withdraw(ev) => user_actions.push(UserAction::Withdraw {
                    tx_hash: request.tx_hash,
                    block_number: request.block_number,
                    timestamp: request.timestamp,
                    request_id: request.request_id,
                    user: request.requestor,
                    asset_out: request.asset_out,
                    amount_out: request.amount_out,
                    burned_at_trsy_price: ev.trsy_price,
                    usd_value_withdrawn: ev.usd_withdraw_value,
                    trsy_burned: ev.trsy_burned,
                }),
                FydeEvents::Swap(ev) => user_actions.push(UserAction::Swap {
                    tx_hash: request.tx_hash,
                    block_number: request.block_number,
                    timestamp: request.timestamp,
                    request_id: request.request_id,
                    user: request.requestor,
                    asset_in: request.asset_in[0],
                    asset_out: request.asset_out[0],
                    amount_in: request.amount_in[0],
                    amount_out: ev.amount_out,
                }),
            }
        }

        Ok(user_actions)
    }
}
