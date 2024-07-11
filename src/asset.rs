use async_trait::async_trait;
use ethers::{
    providers::{Http, Provider},
    types::{Address, U256},
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    errors::FydeError, utils::ToF32, AddressList, Chain, GovernanceModuleContract,
    LiquidVaultContract, RelayerContract, TaxModuleContract, ERC20,
};

pub struct Asset {
    asset_address: Address,
    contract: ERC20<Provider<Http>>,
    liquid_vault: LiquidVaultContract<Provider<Http>>,
    tax_module: TaxModuleContract<Provider<Http>>,
    governance_module: GovernanceModuleContract<Provider<Http>>,
    relayer: RelayerContract<Provider<Http>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AssetAccounting {
    pub token_in_protocol: f32,
    pub token_in_standard_pool: f32,
    pub token_in_governance_pool: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct TargetConcentrations {
    pub target_concentration_deposit: f32,
    pub target_concentration_withdraw: f32,
    pub target_concentration_fyde: f32,
}

#[derive(Debug, Serialize, Clone)]
pub enum WeightStatus {
    Overweight,
    Underweight,
    InRange,
    Undertemined,
}

impl std::fmt::Display for WeightStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightStatus::Overweight => write!(f, "Overweight"),
            WeightStatus::Underweight => write!(f, "Underweight"),
            WeightStatus::InRange => write!(f, "InRange"),
            WeightStatus::Undertemined => write!(f, "Undetermined"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct UniswapInfo {
    pub uniswap_pool: Address,
    pub decimals: u8,
    pub quote_token: Address,
    pub quote_token_decimals: u8,
}

// We make trait here in case we want to implement trait for Vec<Asset> in the future
#[async_trait]
pub trait AssetTrait {
    async fn get_symbol(&self) -> Result<String, FydeError>;
    async fn get_decimals(&self) -> Result<u8, FydeError>;
    async fn get_address(&self) -> Result<Address, FydeError>;
    async fn get_asset_aum(&self) -> Result<f32, FydeError>;
    async fn get_oracle_price(&self, decimals: u8) -> Result<f32, FydeError>;
    async fn get_uniswap_info(&self) -> Result<UniswapInfo, FydeError>;
    async fn get_asset_accounting(&self, decimals: u8) -> Result<AssetAccounting, FydeError>;
    async fn get_asset_target_concentrations(&self) -> Result<TargetConcentrations, FydeError>;
    async fn get_current_concentration(&self, asset_aum: f32, tvl: f32) -> Result<f32, FydeError>;
    async fn get_weight_status(
        &self,
        target_concentrations: &TargetConcentrations,
        current_concentration: f32,
    ) -> Result<WeightStatus, FydeError>;
    async fn get_liquidity_profile(
        &self,
        asset_accounting: AssetAccounting,
    ) -> Result<f32, FydeError>;
    async fn get_is_allowed_on_governance(&self) -> Result<bool, FydeError>;
    async fn get_st_address(&self) -> Result<Address, FydeError>;
    async fn get_is_quarantined(&self) -> Result<bool, FydeError>;
}

impl Asset {
    pub fn new(address: Address, provider: Arc<Provider<Http>>, chain: Chain) -> Self {
        let address_list: AddressList = AddressList::new(&chain);

        Self {
            asset_address: address.clone(),
            contract: ERC20::new(address, provider.clone()),
            liquid_vault: LiquidVaultContract::new(address_list.liquid_vault, provider.clone()),
            tax_module: TaxModuleContract::new(address_list.tax_module, provider.clone()),
            governance_module: GovernanceModuleContract::new(
                address_list.governance_module,
                provider.clone(),
            ),
            relayer: RelayerContract::new(address_list.relayer, provider.clone()),
        }
    }
}

#[async_trait]
impl AssetTrait for Asset {
    async fn get_symbol(&self) -> Result<String, FydeError> {
        let mkr_address: Address = String::from("0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2")
            .parse()
            .expect("Failed to create address");
        let symbol = match self.asset_address == mkr_address {
            true => String::from("MKR"),
            false => self.contract.symbol().call().await?,
        };
        Ok(symbol)
    }

    async fn get_decimals(&self) -> Result<u8, FydeError> {
        Ok(self.contract.decimals().call().await?)
    }

    async fn get_address(&self) -> Result<Address, FydeError> {
        Ok(self.contract.address())
    }

    async fn get_asset_aum(&self) -> Result<f32, FydeError> {
        let asset_aum = self
            .liquid_vault
            .get_asset_aum(self.asset_address)
            .call()
            .await?;
        Ok(asset_aum.to_f32(18.0))
    }

    async fn get_oracle_price(&self, decimals: u8) -> Result<f32, FydeError> {
        let amount: U256 = U256::from(10).pow(U256::from(decimals));
        let oracle_price = self
            .liquid_vault
            .get_quote(self.asset_address, amount)
            .call()
            .await?;
        Ok(oracle_price.to_f32(decimals as f32))
    }

    async fn get_uniswap_info(&self) -> Result<UniswapInfo, FydeError> {
        let asset_info = self
            .liquid_vault
            .asset_info(self.asset_address)
            .call()
            .await?;

        let uniswap_pool = asset_info.1;
        let decimals = asset_info.3;
        let quote_token = asset_info.5;
        let quote_token_decimals = asset_info.4;
        Ok(UniswapInfo {
            uniswap_pool,
            decimals,
            quote_token,
            quote_token_decimals,
        })
    }

    async fn get_asset_accounting(&self, decimals: u8) -> Result<AssetAccounting, FydeError> {
        let token_in_protocol = self
            .liquid_vault
            .total_asset_accounting(self.asset_address)
            .call()
            .await?;
        let token_in_standard_pool = self
            .liquid_vault
            .standard_asset_accounting(self.asset_address)
            .call()
            .await?;
        let token_in_governance_pool = self
            .liquid_vault
            .proxy_asset_accounting(self.asset_address)
            .call()
            .await?;
        Ok(AssetAccounting {
            token_in_protocol: token_in_protocol.to_f32(decimals as f32),
            token_in_standard_pool: token_in_standard_pool.to_f32(decimals as f32),
            token_in_governance_pool: token_in_governance_pool.to_f32(decimals as f32),
        })
    }

    async fn get_asset_target_concentrations(&self) -> Result<TargetConcentrations, FydeError> {
        let concentrations_from_tax: (u128, u128) = self
            .tax_module
            .tax_params(self.asset_address)
            .call()
            .await?;
        let fyde_concentration: (u128, Address, i128, u8, u8, Address, bool) = self
            .liquid_vault
            .asset_info(self.asset_address)
            .call()
            .await?;
        Ok(TargetConcentrations {
            target_concentration_deposit: U256::from(concentrations_from_tax.0).to_f32(18.0),
            target_concentration_withdraw: U256::from(concentrations_from_tax.1).to_f32(18.0),
            target_concentration_fyde: U256::from(fyde_concentration.0).to_f32(18.0),
        })
    }

    async fn get_current_concentration(&self, asset_aum: f32, tvl: f32) -> Result<f32, FydeError> {
        Ok(100.0 * asset_aum / tvl)
    }

    async fn get_weight_status(
        &self,
        target_concentrations: &TargetConcentrations,
        current_concentration: f32,
    ) -> Result<WeightStatus, FydeError> {
        let weight_status = match current_concentration {
            _ if current_concentration > target_concentrations.target_concentration_deposit => {
                WeightStatus::Overweight
            }
            _ if current_concentration < target_concentrations.target_concentration_withdraw => {
                WeightStatus::Underweight
            }
            _ if current_concentration > target_concentrations.target_concentration_withdraw
                && current_concentration < target_concentrations.target_concentration_deposit =>
            {
                WeightStatus::InRange
            }
            _ => WeightStatus::Undertemined,
        };

        Ok(weight_status)
    }

    async fn get_liquidity_profile(
        &self,
        asset_accounting: AssetAccounting,
    ) -> Result<f32, FydeError> {
        if asset_accounting.token_in_protocol == 0.0 {
            return Ok(0.0);
        }
        Ok(100.0 * asset_accounting.token_in_standard_pool / asset_accounting.token_in_protocol)
    }

    async fn get_is_allowed_on_governance(&self) -> Result<bool, FydeError> {
        let is_allowed = self
            .governance_module
            .is_on_governance_whitelist(self.asset_address)
            .call()
            .await?;
        Ok(is_allowed)
    }

    async fn get_st_address(&self) -> Result<Address, FydeError> {
        let st_address = self
            .governance_module
            .asset_to_strsy(self.asset_address)
            .call()
            .await?;
        Ok(st_address)
    }

    async fn get_is_quarantined(&self) -> Result<bool, FydeError> {
        let is_quarantined = self
            .relayer
            .is_quarantined(self.asset_address)
            .call()
            .await?;
        Ok(is_quarantined)
    }
}
