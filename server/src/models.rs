use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
#[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
}
impl User {
    pub fn new(username: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            username,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, SimpleObject)]
#[graphql(complex)]
pub struct Authenticator {
    pub user_id: Uuid,
    #[graphql(skip)]
    pub passkey: Passkey,
    pub user_agent_short: String,
    pub created_at: DateTime<Utc>,
}
