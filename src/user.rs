use crate::{errors::FydeError, AddressList, GovernanceModuleContract, LiquidVaultContract, ERC20};
use ethers::{
    contract::Multicall,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::{collections::HashMap, sync::Arc};

pub type ERC20s = Vec<ERC20<Provider<Http>>>;

pub struct User {
    address: Address,
    provider: Arc<Provider<Http>>,
    multicall: Multicall<Provider<Http>>,
    governance_module: GovernanceModuleContract<Provider<Http>>,
    liquid_vault: LiquidVaultContract<Provider<Http>>,
    address_list: AddressList,
}

pub struct GovernanceData {
    pub st_trsy_balance: HashMap<Address, U256>,
    pub current_governance_rights: HashMap<Address, U256>,
    pub total_voting_rights: HashMap<Address, U256>,
}

impl User {
    pub async fn new(
        provider: Arc<Provider<Http>>,
        address_list: AddressList,
        user_address: Address,
    ) -> Self {
        Self {
            address: user_address,
            provider: provider.clone(),
            multicall: Multicall::new(provider.clone(), None)
                .await
                .expect("Failed to create Multicall"),
            governance_module: GovernanceModuleContract::new(
                address_list.governance_module,
                provider.clone(),
            ),
            liquid_vault: LiquidVaultContract::new(address_list.liquid_vault, provider.clone()),
            address_list,
        }
    }

    pub async fn get_trsy_balance(&self) -> Result<U256, FydeError> {
        Ok(self.liquid_vault.balance_of(self.address).call().await?)
    }

    pub async fn get_allowances(
        &self,
        assets: &[Address],
    ) -> Result<HashMap<Address, U256>, FydeError> {
        let mut multicall = self.multicall.clone();
        multicall.clear_calls();

        let erc20s: ERC20s = assets
            .iter()
            .map(|&addr| ERC20::new(addr, self.provider.clone()))
            .collect();

        let relayer_address = self.address_list.relayer;

        for erc20 in erc20s.iter() {
            multicall.add_call(erc20.allowance(self.address, relayer_address), false);
        }
        let results: Vec<U256> = self.multicall.call_array().await?;
        multicall.clear_calls();

        let mut allowances: HashMap<Address, U256> = HashMap::new();
        for (i, asset) in assets.iter().enumerate() {
            allowances.insert(*asset, results[i]);
        }

        Ok(allowances)
    }

    pub async fn get_balances(
        &self,
        assets: &[Address],
    ) -> Result<HashMap<Address, U256>, FydeError> {
        let mut multicall = self.multicall.clone();
        multicall.clear_calls();

        let erc20s: ERC20s = assets
            .iter()
            .map(|&addr| ERC20::new(addr, self.provider.clone()))
            .collect();

        for erc20 in erc20s.iter() {
            multicall.add_call(erc20.balance_of(self.address), false);
        }
        let results: Vec<U256> = self.multicall.call_array().await?;
        multicall.clear_calls();

        let mut balances: HashMap<Address, U256> = HashMap::new();
        for (i, asset) in assets.iter().enumerate() {
            balances.insert(*asset, results[i]);
        }

        Ok(balances)
    }

    pub async fn get_proxy_address(&self) -> Result<Option<Address>, FydeError> {
        let proxy_address: Option<Address> = match self
            .governance_module
            .user_to_proxy(self.address)
            .call()
            .await?
        {
            address if address == Address::zero() => None,
            address => Some(address),
        };
        Ok(proxy_address)
    }

    pub async fn get_governance_data(
        &self,
        assets_in_gov: &[Address],
    ) -> Result<GovernanceData, FydeError> {
        let mut multicall = self.multicall.clone();
        multicall.clear_calls();
        for asset in assets_in_gov {
            multicall.add_call(
                self.governance_module.strsy_balance(self.address, *asset),
                false,
            );
            multicall.add_call(
                self.governance_module.proxy_balance(self.address, *asset),
                false,
            );
            multicall.add_call(
                self.governance_module
                    .get_user_gt_allowance(self.address, *asset),
                false,
            );
        }
        let res: Vec<U256> = self.multicall.call_array().await?;
        multicall.clear_calls();

        let mut st_trsy_balance: HashMap<Address, U256> = HashMap::new();
        let mut current_governance_rights: HashMap<Address, U256> = HashMap::new();
        let mut total_voting_rights: HashMap<Address, U256> = HashMap::new();

        for (i, asset) in assets_in_gov.iter().enumerate() {
            st_trsy_balance.insert(*asset, res[i * 3]);
            current_governance_rights.insert(*asset, res[i * 3 + 1]);
            total_voting_rights.insert(*asset, res[i * 3 + 2]);
        }

        Ok(GovernanceData {
            st_trsy_balance,
            current_governance_rights,
            total_voting_rights,
        })
    }

    pub async fn get_governance_data_when_multicall_fail(
        &self,
        assets_in_gov: &[Address],
    ) -> Result<GovernanceData, FydeError> {
        let mut multicall = self.multicall.clone();
        multicall.clear_calls();
        for asset in assets_in_gov {
            multicall.add_call(
                self.governance_module.strsy_balance(self.address, *asset),
                false,
            );
            multicall.add_call(
                self.governance_module.proxy_balance(self.address, *asset),
                false,
            );
            multicall.add_call(
                self.governance_module.proxy_balance(self.address, *asset),
                false,
            );
        }
        let res: Vec<U256> = self.multicall.call_array().await?;
        multicall.clear_calls();

        let mut st_trsy_balance: HashMap<Address, U256> = HashMap::new();
        let mut current_governance_rights: HashMap<Address, U256> = HashMap::new();
        let mut total_voting_rights: HashMap<Address, U256> = HashMap::new();

        for (i, asset) in assets_in_gov.iter().enumerate() {
            st_trsy_balance.insert(*asset, res[i * 3]);
            current_governance_rights.insert(*asset, res[i * 3 + 1]);
            total_voting_rights.insert(*asset, res[i * 3 + 2]);
        }

        Ok(GovernanceData {
            st_trsy_balance,
            current_governance_rights,
            total_voting_rights,
        })
    }
}
