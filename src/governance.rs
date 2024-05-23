use ethers::{
    contract::Multicall,
    providers::{Http, Provider},
    types::{Address, I256},
};
use std::sync::Arc;

use crate::{errors::FydeError, AddressList, GovernanceModuleContract};

pub struct Governance {
    governance_module: GovernanceModuleContract<Provider<Http>>,
    multicall: Multicall<Provider<Http>>,
}

impl Governance {
    pub async fn new(provider: Arc<Provider<Http>>, address_list: AddressList) -> Self {
        let governance_module =
            GovernanceModuleContract::new(address_list.governance_module, provider.clone());
        let multicall = Multicall::new(provider, None)
            .await
            .expect("Failed to create Multicall");
        Self {
            governance_module,
            multicall,
        }
    }

    pub async fn get_list_of_governance_users(&self) -> Result<Vec<Address>, FydeError> {
        Ok(self.governance_module.get_all_gov_users().call().await?)
    }

    pub async fn get_proxy_to_rebalance(
        &mut self,
        asset: Address,
    ) -> Result<Vec<Address>, FydeError> {
        let gov_users = self.get_list_of_governance_users().await?;

        self.multicall.clear_calls();
        for user in &gov_users {
            self.multicall.add_call(
                self.governance_module.get_token_unbalance(*user, asset),
                false,
            );
        }

        let unbalance: Vec<I256> = self.multicall.call_array().await?;
        self.multicall.clear_calls();

        // First, create a vector of tuples containing Address and I256 values
        let mut user_unbalance: Vec<(Address, I256)> = gov_users
            .into_iter()
            .zip(unbalance.into_iter())
            .filter(|&(_, unbalance)| unbalance != I256::zero())
            .collect();

        // Sort the vector of tuples based on I256 values in ascending order
        user_unbalance.sort_by(|a, b| a.1.cmp(&b.1));

        // Extract the sorted gov_users from the sorted vector of tuples
        let sorted_gov_users: Vec<Address> =
            user_unbalance.iter().map(|&(address, _)| address).collect();

        Ok(sorted_gov_users)
    }
}
