use crate::{
    errors::FydeError, AddressList, Chain, LiquidVaultContract, LiquidVaultContractEvents,
    StakingTRSY,
};
use ethers::{
    contract::Multicall,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::sync::Arc;

pub struct LiquidVault {
    contract: LiquidVaultContract<Provider<Http>>,
    staking_trsy: StakingTRSY<Provider<Http>>,
    multicall: Multicall<Provider<Http>>,
    address: Address,
}

impl LiquidVault {
    pub async fn new(provider: Arc<Provider<Http>>, chain: Chain) -> Self {
        let address_list: AddressList = AddressList::new(&chain);
        let contract = LiquidVaultContract::new(address_list.liquid_vault, provider.clone());
        let staking_trsy = StakingTRSY::new(address_list.staking_trsy, provider.clone());
        let multicall = Multicall::new(provider, None)
            .await
            .expect("Failed to create Multicall");

        Self {
            contract,
            staking_trsy,
            multicall,
            address: address_list.liquid_vault,
        }
    }

    pub async fn get_tvl(&self) -> Result<U256, FydeError> {
        Ok(self.contract.compute_protocol_aum().call().await?)
    }

    pub async fn get_trsy_supply(&self) -> Result<U256, FydeError> {
        Ok(self.contract.total_supply().call().await?)
    }

    pub async fn get_trsy_staked(&self) -> Result<U256, FydeError> {
        Ok(self.staking_trsy.total_supply().call().await?)
    }

    pub async fn get_trsy_value(&self) -> Result<U256, FydeError> {
        let tvl = self.contract.compute_protocol_aum().call().await?;
        let trsy_supply = self.contract.total_supply().call().await?;
        Ok(tvl / trsy_supply)
    }

    pub async fn get_total_fees(&self) -> Result<U256, FydeError> {
        let events = self
            .contract
            .events()
            .from_block(0)
            .query()
            .await
            .expect("Failed to get events");

        let mut tax = U256::from(0);
        for event in events.iter() {
            if let LiquidVaultContractEvents::TransferFilter(ev) = event {
                if ev.from
                    == "0x0000000000000000000000000000000000000000"
                        .parse()
                        .expect("Failed to parse zero address")
                    && ev.to == self.address
                {
                    tax += ev.amount;
                }
            }
        }
        Ok(tax)
    }

    pub async fn get_management_fees(&self) -> Result<U256, FydeError> {
        let events = self
            .contract
            .events()
            .from_block(0)
            .query()
            .await
            .expect("Failed to get events");

        let mut fees = U256::from(0);

        for event in events.iter() {
            if let LiquidVaultContractEvents::ManagementFeeCollectedFilter(ev) = event {
                fees += ev.fee_to_mint;
            }
        }

        Ok(fees)
    }

    pub async fn get_tax_fees(&self) -> Result<U256, FydeError> {
        let tax_fees = self.get_total_fees().await? - self.get_management_fees().await?;
        Ok(tax_fees)
    }

    pub async fn get_burned_trsy_by_swap(&self) -> Result<U256, FydeError> {
        let events = self
            .contract
            .events()
            .from_block(0)
            .query()
            .await
            .expect("Failed to get events");
        let mut burned = U256::from(0);
        for event in events.iter() {
            if let LiquidVaultContractEvents::TransferFilter(ev) = event {
                if ev.to
                    == "0x0000000000000000000000000000000000000000"
                        .parse()
                        .expect("Failed to parse zero address")
                    && ev.from == self.address
                {
                    burned += ev.amount;
                }
            }
        }
        Ok(burned)
    }

    pub async fn get_assets_list(&mut self) -> Result<Vec<Address>, FydeError> {
        let n_assets = self
            .contract
            .get_assets_list_length()
            .call()
            .await?
            .as_u128() as usize;

        self.multicall.clear_calls();
        for n in 0..n_assets {
            self.multicall
                .add_call(self.contract.assets_list(n.into()), false);
        }
        let assets_list: Vec<Address> = self.multicall.call_array().await?;

        Ok(assets_list)
    }
}
