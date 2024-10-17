use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::Chain;

#[derive(Serialize, Deserialize, Debug)]
struct AssetFields {
    symbol: String,
    address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub body: String,
    pub choices: Vec<String>,
    pub start: u64,
    pub end: u64,
    pub snapshot: String,
    pub state: String,
    pub scores: Vec<f64>,
    pub scores_total: f64,
    pub scores_updated: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Space {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalsVector {
    pub proposals: Vec<Proposal>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vote {
    pub id: String,
    pub voter: String,
    pub vp: f64,
    pub vp_state: String,
    pub created: u64,
    pub proposal: VoteProposal,
    pub choice: serde_json::Value,
    pub space: Space,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteProposal {
    pub id: String,
    pub choices: Vec<String>,
    pub scores_total: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VotesResponse {
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIResponse {
    pub symbol_mapping: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalsResponse {
    pub proposals: Vec<Proposal>,
    pub choices: Vec<String>,
}

pub struct Snapshot {
    client: Client,
    snapshot_url: SnapshotUrl,
}

struct SnapshotUrl {
    asset_endpoint: String,
    snapshot_graphql: String,
    space_name: String,
}

impl SnapshotUrl {
    pub fn new(chain: Chain) -> Self {
        let asset_endpoint = match chain {
            Chain::Mainnet => "https://api.fyde.fi/api/assets",
            Chain::Sepolia => "https://test.fyde.fi/api/assets",
        };

        let snapshot_graphql = match chain {
            Chain::Mainnet => "https://hub.snapshot.org/graphql",
            Chain::Sepolia => "https://testnet.hub.snapshot.org/graphql",
        };

        let space_name = match chain {
            Chain::Mainnet => "vefyde.eth",
            Chain::Sepolia => "vefyde.eth",
        };

        Self {
            asset_endpoint: asset_endpoint.to_string(),
            snapshot_graphql: snapshot_graphql.to_string(),
            space_name: space_name.to_string(),
        }
    }
}

impl Snapshot {
    pub fn new(chain: Chain) -> Self {
        Self {
            client: Client::new(),
            snapshot_url: SnapshotUrl::new(chain),
        }
    }

    pub async fn fetch_address(&self) -> Result<APIResponse, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(&self.snapshot_url.asset_endpoint)
            .send()
            .await?
            .json::<Vec<AssetFields>>()
            .await?;

        let mut asset_map = std::collections::HashMap::new();

        for asset in response {
            asset_map.insert(asset.symbol, asset.address);
        }

        Ok(APIResponse {
            symbol_mapping: asset_map,
        })
    }

    pub async fn fetch_latest_proposal(
        &self,
        skip_index: usize,
    ) -> Result<ProposalsResponse, Box<dyn std::error::Error>> {
        println!("Fetching latest proposal...");
        let fyde_response = self.fetch_address().await?;

        let query = json!({
            "query": format!(r#"
            {{
                proposals(
                    first: 1,
                    skip: {},
                    where: {{
                        space_in: ["{}"],
                        state: "closed"
                    }},
                    orderBy: "created",
                    orderDirection: desc
                ) {{
                    id
                    title
                    body
                    choices
                    start
                    end
                    snapshot
                    state
                    scores
                    scores_total
                    scores_updated
                    author
                    space {{
                        id
                        name
                    }}
                }}
            }}
            "#,
                skip_index,
                self.snapshot_url.space_name
            )
        });

        let response = self
            .client
            .post(&self.snapshot_url.snapshot_graphql)
            .json(&query)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let proposal = serde_json::from_value::<ProposalsVector>(response["data"].clone())?
            .proposals
            .into_iter()
            .next()
            .ok_or("No proposals found")?;

        let temp_proposal = proposal.clone();
        let mut choices = Vec::new();

        for choice in temp_proposal.choices {
            choices.push(choice.clone());
        }

        let mut choices_address = Vec::new();
        for choice in choices {
            let address = fyde_response.symbol_mapping.get(&choice).unwrap();
            choices_address.push(address.clone());
        }

        let proposal_response = ProposalsResponse {
            proposals: vec![proposal],
            choices: choices_address,
        };

        Ok(proposal_response)
    }

    pub async fn fetch_votes(
        &self,
        proposal_id: &str,
        num_votes: usize,
        skip_index: usize,
    ) -> Result<Vec<Vote>, Box<dyn std::error::Error>> {
        let query = json!({
            "query": format!(
                r#"
              {{
                  votes(
                      first: {},
                      skip: {},
                      where: {{
                          proposal: "{}"
                      }}
                  ) {{
                      id
                      voter
                      vp
                      vp_state
                      created
                      proposal {{
                          id
                          choices
                          scores_total
                      }}
                      choice
                      space {{
                          id
                          name
                      }}
                  }}
              }}
              "#,
                num_votes,
                skip_index,
                proposal_id
            )
        });

        let response = self
            .client
            .post(&self.snapshot_url.snapshot_graphql)
            .json(&query)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let votes = serde_json::from_value::<VotesResponse>(response["data"].clone())?.votes;

        Ok(votes)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_latest_proposal() {
        let snapshot = Snapshot::new(Chain::Sepolia);
        let proposal = snapshot.fetch_latest_proposal(0).await.unwrap();
        println!("{:?}", proposal);
    }
}
*/
