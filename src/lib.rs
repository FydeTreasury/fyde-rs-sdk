use ethers::prelude::abigen;
use ethers::types::Address;

pub mod asset;
pub mod errors;
pub mod governance;
pub mod liquid_vault;
pub mod protocol_history;
pub mod user;
pub mod utils;
pub mod ve_fyde;

abigen!(LiquidVaultContract, "./src/abis/LiquidVault.json");
abigen!(TaxModuleContract, "./src/abis/TaxModule.json");
abigen!(GovernanceModuleContract, "./src/abis/GovernanceModule.json");
abigen!(OracleModuleContract, "./src/abis/OracleModule.json");
abigen!(RelayerContract, "./src/abis/RelayerV2.json");
abigen!(StakingTRSY, "./src/abis/StakingTRSY.json");
abigen!(
    RevenueVeFydeDistributorContract,
    "./src/abis/RevenueVeFydeDistributor.json"
);
abigen!(VoteEscrowContract, "./src/abis/VoteEscrow.json");

abigen!(
    ERC20,
    r#"[
        function symbol() external view returns (string)
        function decimals() external view returns(uint8)
        function balanceOf(address) external view returns (uint256)
        function allowance(address,address) external view returns (uint256)
        ]"#,
);
abigen!(Strsy, "./src/abis/Strsy.json");

#[derive(Debug, Clone)]
// List of useful address for the Fyde protocol
pub struct AddressList {
    pub liquid_vault: Address,
    pub relayer: Address,
    pub tax_module: Address,
    pub governance_module: Address,
    pub oracle_module: Address,
    pub staking_trsy: Address,
    pub staking_lrt: Address,
    pub lrt_reward_distribution: Address,
    pub weth: Address,
    pub fyde_token: Address,
    pub vote_escrow: Address,
    pub vefyde_fee_distributor: Address,
    pub strsy: Address,
}

#[derive(Debug, Clone)]
pub enum Chain {
    Mainnet,
    Sepolia,
}

impl std::str::FromStr for Chain {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mainnet" => Ok(Chain::Mainnet),
            "Sepolia" => Ok(Chain::Sepolia),
            _ => Err(()),
        }
    }
}

impl AddressList {
    pub fn new(chain: &Chain) -> Self {
        match chain {
            Chain::Mainnet => Self::mainnet(),
            Chain::Sepolia => Self::sepolia(),
        }
    }

    pub fn mainnet() -> Self {
        Self {
            liquid_vault: String::from("0x87Cc45fFF5c0933bb6aF6bAe7Fc013b7eC7df2Ee")
                .parse()
                .expect("Failed to parse Liquid Vault address"),
            relayer: String::from("0x6830C61dF103946B63C786e63222c59677F32078")
                .parse()
                .expect("Failed to parse Relayer address"),
            governance_module: String::from("0xc6F50903a058f3807111619bD4B24cA64b8239E1")
                .parse()
                .expect("Failed to parse GovernanceModule address"),
            oracle_module: String::from("0xe8e40Fd4DdAb26B44b1fb2D6d73833Cb0A33B736")
                .parse()
                .expect("Failed to parse OracleModule address"),
            staking_trsy: String::from("0x6c7441C76D85d7aB43EacD076D37b0775f5C32f7")
                .parse()
                .expect("Failed to parse stakingTRSY address"),
            tax_module: String::from("0x35afe52bDDEdBc9Bbe53af119568264dA00a70D3")
                .parse()
                .expect("Failed to parse TaxModule address"),
            staking_lrt: String::from("0x3f69F62e25441Cf72E362508f4d6711d53B05341")
                .parse()
                .expect("Failed to parse stakingETH address"),
            lrt_reward_distribution: String::from("0x99628825156746FBabc2819d202eE30ecB3C71a6")
                .parse()
                .expect("Failed to parse RewardDistribution address"),
            weth: String::from("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")
                .parse()
                .expect("Failed to parse TaxModule address"),
            fyde_token: String::from("0xdDE736837C7c275A952a52eE11FAce88adde6711")
                .parse()
                .expect("Failed to parse FydeToken address"),
            vote_escrow: String::from("0x9B369202ff147B54eA7092BC94425C781094DbdE")
                .parse()
                .expect("Failed to parse VoteEscrow address"),
            vefyde_fee_distributor: String::from("0x41B911286E63c508345bA581d75928ecE4A0f543")
                .parse()
                .expect("Failed to parse RevenueVeFydeDistributor address"),
            strsy: String::from("0xE11DF8c0E9B5697bd31515D0Fc5f4C9BD71566B9")
                .parse()
                .expect("Failed to parse sTRSY address"),
        }
    }

    pub fn sepolia() -> Self {
        Self {
            liquid_vault: String::from("0x922B0549983BF40C939c6b66996FF61D140F1455")
                .parse()
                .expect("Failed to parse Liquid Vault address"),
            relayer: String::from("0xF1b793D8A22c2ae7E3ad1Eb578eC7FB6Cdb15eF9")
                .parse()
                .expect("Failed to parse Relayer address"),
            governance_module: String::from("0xe5905bdA71CCaCE9711EA36C2245d8cA231512A9")
                .parse()
                .expect("Failed to parse GovernanceModule address"),
            oracle_module: String::from("0x4bDD437029d9a9095349b41e837CBCA871F84701")
                .parse()
                .expect("Failed to parse OracleModule address"),
            staking_trsy: String::from("0xefC7Cc6d8b7A2e43a1a65A7cED1E53f555466e17")
                .parse()
                .expect("Failed to parse stakingTRSY address"),
            tax_module: String::from("0x18e82DAbB3FFC06F8Bb838e663E88374ba271cEb")
                .parse()
                .expect("Failed to parse TaxModule address"),
            staking_lrt: String::from("0xa98a16c7FBc238e83b7d225287821c3F6E85E0CF")
                .parse()
                .expect("Failed to parse stakingETH address"),
            lrt_reward_distribution: String::from("0x1168890f6Ba96ACc523F0c6d0210Ebb6B40580a9")
                .parse()
                .expect("Failed to parse RewardDistribution address"),
            weth: String::from("0xAEB122a5Ed6cA3Cab6C42d6867AfbFe582db8761")
                .parse()
                .expect("Failed to parse TaxModule address"),
            fyde_token: String::from("0xBF2Dbb0f7C90bAC051dA1E6eF9a836b675B83c02")
                .parse()
                .expect("Failed to parse FydeToken address"),
            vote_escrow: String::from("0x2A26D7f71A0dBB600A317160e42632408D0EE6eb")
                .parse()
                .expect("Failed to parse VoteEscrow address"),
            vefyde_fee_distributor: String::from("0x27b70B78E5a5Bf42E6893fBdbACD048852E87077")
                .parse()
                .expect("Failed to parse RevenueVeFydeDistributor address"),
            strsy: String::from("0xf81203D4da69823609ca74DEd222ce76027a81CB")
                .parse()
                .expect("Failed to parse sTRSY address"),
        }
    }
}
