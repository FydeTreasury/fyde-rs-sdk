use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio;
use chrono::Utc;

#[derive(Serialize, Deserialize, Debug)]
struct Proposal {
  id: String,
  title: String,
  body: String,
  choices: Vec<String>,
  start: u64,
  end: u64,
  snapshot: String,
  state: String,
  scores: Vec<f64>,
  scores_total: f64,
  scores_updated: u64,
  author: String,
  space: Space,
}

#[derive(Serialize, Deserialize, Debug)]
struct Space {
    id: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProposalsResponse {
    proposals: Vec<Proposal>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vote {
    id: String,
    voter: String,
    vp: f64,
    vp_state: String,
    created: u64,
    proposal: VoteProposal,
    choice: serde_json::Value,
    space: Space,
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

pub struct Snapshot {
  client: Client,
}

impl Snapshot {
  pub fn new() -> Self {
      Self {
          client: Client::new(),
      }
  }

  pub async fn fetch_latest_proposal(&self) -> Result<Proposal, Box<dyn std::error::Error>> {
      let query = json!({
          "query": r#"
          {
              proposals(
                  first: 1,
                  skip: 0,
                  where: {
                      space_in: ["veFyde.eth"]
                  },
                  orderBy: "created",
                  orderDirection: desc
              ) {
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
                  space {
                      id
                      name
                  }
              }
          }
          "#
      });

      let response = self
          .client
          .post("https://testnet.hub.snapshot.org/graphql")
          .json(&query)
          .send()
          .await?
          .json::<serde_json::Value>()
          .await?;

      let proposal = serde_json::from_value::<ProposalsResponse>(response["data"].clone())?
          .proposals
          .into_iter()
          .next()
          .ok_or("No proposals found")?;
      
      Ok(proposal)
  }

  pub async fn fetch_votes(&self, proposal_id: &str, num_votes: usize) -> Result<Vec<Vote>, Box<dyn std::error::Error>> {
      let query = json!({
          "query": format!(
              r#"
              {{
                  votes(
                      first: {},
                      skip: 0,
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

