use std::collections::HashSet;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use webauthn_rs::prelude::*;

/*
 * server side app state and setup
 */

use crate::db::DB;

#[derive(Clone)]
pub struct AppState {
    // Webauthn has no mutable inner state, so Arc and read only is sufficent.
    // Alternately, you could use a reference here provided you can work out
    // lifetimes.
    pub webauthn: Arc<Webauthn>,
    pub db: DB,
    // Channel used to send messages to all connected clients.
    // for chat example
    pub tx: broadcast::Sender<String>,
    pub connected_usernames: Arc<Mutex<HashSet<String>>>,
    pub recent_messages: Arc<Mutex<Vec<String>>>,
}

impl AppState {
    pub async fn new() -> Self {
        // Effective domain name. Ff changed, all credentials are invalidated!!
        let rp_id = env::var("RP_ID").expect("RP_ID environment variable not set");

        // Url containing the effective domain name
        // MUST include the port number!
        let rp_origin =
            Url::parse(&env::var("RP_ORIGIN").expect("RP_ORIGIN environment variable not set"))
                .expect("Invalid URL");

        let builder = WebauthnBuilder::new(&rp_id, &rp_origin).expect("Invalid configuration");

        // Set a "nice" relying party name. Has no security properties and
        // may be changed in the future.
        let rp_name = env::var("RP_NAME").expect("RP_NAME environment variable not set");
        let builder = builder.rp_name(&rp_name);

        // Consume the builder and create our webauthn instance.
        let webauthn = Arc::new(builder.build().expect("Invalid configuration"));

        // db
        let db = DB::new().await;

        let (tx, _rx) = broadcast::channel(100);
        let user_set = Arc::new(Mutex::new(HashSet::new()));
        let recent_messages = Arc::new(Mutex::new(Vec::new()));
        AppState {
            webauthn,
            db,
            tx,
            connected_usernames: user_set,
            recent_messages,
        }
    }
}
