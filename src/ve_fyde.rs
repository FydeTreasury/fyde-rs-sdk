use crate::{
    errors::FydeError, AddressList, Chain, Checkpoint, VoteEscrowContract, VoteEscrowContractEvents,
};
use ethers::{
    contract::Multicall,
    providers::{Http, Provider},
    types::{Address, U256},
};
use serde::Serialize;
use std::{sync::Arc, vec};

pub struct VeFyde {
    vote_escrow: VoteEscrowContract<Provider<Http>>,
    multicall: Multicall<Provider<Http>>,
}

#[derive(Serialize, Default, Debug)]
pub struct VeFydeUser {
    pub ve_fyde_balance: u128,
    pub fyde_locked: u128,
    pub last_locking_date: u64,
    pub history_length: u64,
    pub lock_duration_in_days: u64,
    pub unlock_date: u64,
    pub ve_balance_chart: Option<VeBalanceChart>,
}

#[derive(Serialize, Default, Debug)]
pub struct VeBalanceChart {
    /// Timestamps
    pub ts: Vec<u64>,
    /// Ve balances decaying over time
    pub ve_balance: Vec<f32>,
}

impl VeBalanceChart {
    pub fn get_decay_graph(last_locking_date: u64, expiry: u64, checkpoint: Checkpoint) -> Self {
        let range = expiry - last_locking_date;
        let mut ve_balance_chart = VeBalanceChart {
            ts: vec![],
            ve_balance: vec![],
        };
        let seconds_in_a_day = 86_400_u64;

        // Loop through each day until the expiry
        for i in (0..range).step_by(seconds_in_a_day as usize) {
            let ts = last_locking_date + i;
            let ve_balance = checkpoint.value.bias - checkpoint.value.slope * ts as u128;
            ve_balance_chart.ts.push(ts);
            ve_balance_chart
                .ve_balance
                .push(ve_balance as f32 / 1e18_f32);
        }

        // Ensure the last data point is exactly at expiry if not already included
        if ve_balance_chart.ts.last().copied() != Some(expiry) {
            let ve_balance = checkpoint.value.bias - checkpoint.value.slope * expiry as u128;
            ve_balance_chart.ts.push(expiry);
            ve_balance_chart
                .ve_balance
                .push(ve_balance as f32 / 1e18_f32);
        }

        ve_balance_chart
    }
}

impl VeFyde {
    pub async fn new(provider: Arc<Provider<Http>>, chain: Chain) -> Self {
        let address_list: AddressList = AddressList::new(&chain);
        let vote_escrow = VoteEscrowContract::new(address_list.vote_escrow, provider.clone());
        let multicall = Multicall::new(provider.clone(), None)
            .await
            .expect("Failed to create Multicall");
        Self {
            vote_escrow,
            multicall,
        }
    }

    pub async fn get_ve_fyde_holders_list(&self) -> Result<Vec<Address>, FydeError> {
        let events = self
            .vote_escrow
            .events()
            .from_block(20231776)
            .query()
            .await?;

        let mut holders: Vec<Address> = vec![];
        for event in events {
            match event {
                VoteEscrowContractEvents::UpdateLockFilter(ev) => {
                    if holders.contains(&ev.user) {
                        continue;
                    }
                    holders.push(ev.user);
                }
                _ => {}
            }
        }
        Ok(holders)
    }

    pub async fn get_ve_fyde_balance(&self, user: Address) -> Result<u128, FydeError> {
        let balance = self.vote_escrow.balance_of(user).call().await?;
        Ok(balance)
    }

    pub async fn get_ve_fyde_data(
        &self,
        user: Address,
        draw_vefyde_chart: bool,
    ) -> Result<VeFydeUser, FydeError> {
        let mut multicall = self.multicall.clone();
        multicall.clear_calls();

        multicall.add_call(self.vote_escrow.balance_of(user), false);
        multicall.add_call(self.vote_escrow.position_data(user), false);
        multicall.add_call(self.vote_escrow.get_user_history_length(user), false);
        let results: (u128, (u128, u128), U256) = multicall.call().await?;
        let ve_fyde_balance = results.0;
        let (fyde_locked, expiry) = results.1;
        let history_length = results.2;

        if history_length == U256::zero() || expiry == 0 {
            return Ok(VeFydeUser::default());
        }

        let checkpoint: Checkpoint = self
            .vote_escrow
            .get_user_history_at(user, history_length - 1)
            .call()
            .await?;

        let last_locking_date = checkpoint.timestamp as u64;
        let lock_duration = expiry as u64 - last_locking_date;

        let ve_balance_chart = if draw_vefyde_chart {
            Some(VeBalanceChart::get_decay_graph(
                last_locking_date,
                expiry as u64,
                checkpoint,
            ))
        } else {
            None
        };

        let ve_fyde_user = VeFydeUser {
            ve_fyde_balance,
            fyde_locked,
            last_locking_date,
            history_length: history_length.as_u64(),
            lock_duration_in_days: lock_duration,
            unlock_date: expiry as u64,
            ve_balance_chart,
        };

        Ok(ve_fyde_user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ve_fyde() -> Result<(), FydeError> {
        let provider = Arc::new(
            Provider::<Http>::try_from(
                "https://eth-mainnet.g.alchemy.com/v2/6scwdLmXmD0Ifv_8TgZaNA5Y7MzBhoZP",
            )
            .expect("Failed to create provider"),
        );
        let chain = Chain::Mainnet;
        let ve_fyde = VeFyde::new(provider.clone(), chain).await;
        let holders = ve_fyde.get_ve_fyde_holders_list().await?;

        for holder in holders {
            println!("holder: {:?}", holder);
            let ve_fyde_user = ve_fyde.get_ve_fyde_data(holder, false).await?;
            println!("{:?}", ve_fyde_user);
        }

        Ok(())
    }
}
