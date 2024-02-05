use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use webauthn_rs::prelude::*;

/*
 * server side app state and setup
 */

// Configure the Webauthn instance by using the WebauthnBuilder.
// implications: you can NOT change your rp_id (relying party id), without
// invalidating all webauthn credentials.

use crate::db::DB;

pub struct Data {
    pub name_to_id: HashMap<String, Uuid>,
    pub keys: HashMap<Uuid, Vec<Passkey>>,
}

#[derive(Clone)]
pub struct AppState {
    // Webauthn has no mutable inner state, so Arc and read only is sufficent.
    // Alternately, you could use a reference here provided you can work out
    // lifetimes.
    pub webauthn: Arc<Webauthn>,
    // This needs mutability, so does require a mutex.
    pub users: Arc<Mutex<Data>>,
    pub db: DB,
}

impl AppState {
    pub async fn new() -> Self {
        // Effective domain name.
        // if changed, all credentials are invalidated!!
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

        // in memory storage
        let users = Arc::new(Mutex::new(Data {
            name_to_id: HashMap::new(),
            keys: HashMap::new(),
        }));

        // db
        let db = DB::new().await;

        AppState {
            webauthn,
            users,
            db,
        }
    }
}
