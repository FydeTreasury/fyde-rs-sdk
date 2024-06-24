use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
struct Space {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProposalsVector {
    proposals: Vec<Proposal>,
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
struct VoteProposal {
    id: String,
    choices: Vec<String>,
    scores_total: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct VotesResponse {
    votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse {
    symbol_mapping: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalsResponse {
    pub proposals: Vec<Proposal>,
    pub choices: Vec<String>,
}

pub struct Snapshot {
  client: Client,
}

impl Snapshot {
  pub fn new() -> Self {
      Self {
          client: Client::new(),
      }
  }

  pub async fn fetch_address(&self) -> Result<APIResponse, Box<dyn std::error::Error>> {
    let url = "https://test.fyde.fi/api/assets";

    let response = self
        .client
        .get(url)
        .send()
        .await?
        .json::<Vec<AssetFields>>().await?;

    let mut asset_map = std::collections::HashMap::new();

    for asset in response {
        asset_map.insert(asset.symbol, asset.address);
    } 

    Ok(APIResponse {
        symbol_mapping: asset_map,
    })
    }

    pub async fn fetch_latest_proposal(&self, skip_index: usize) -> Result<ProposalsResponse, Box<dyn std::error::Error>> {
        println!("Fetching latest proposal...");
        let fyde_response = self.fetch_address().await?;

        let query = json!({
            "query": format!(r#"
            {{
                proposals(
                    first: 1,
                    skip: {},
                    where: {{
                        space_in: ["veFyde.eth"],
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
            skip_index
            )
        });

    let response = self
        .client
        .post("https://testnet.hub.snapshot.org/graphql")
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

  pub async fn fetch_votes(&self, proposal_id: &str, num_votes: usize, skip_index: usize) -> Result<Vec<Vote>, Box<dyn std::error::Error>> {
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
          .post("https://testnet.hub.snapshot.org/graphql")
          .json(&query)
          .send()
          .await?
          .json::<serde_json::Value>()
          .await?;

      let votes = serde_json::from_value::<VotesResponse>(response["data"].clone())?.votes;
      
      Ok(votes)
  }
}

