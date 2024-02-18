use std::env;
use std::sync::Arc;
use uaparser::UserAgentParser;
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
    pub ua_parser: Arc<UserAgentParser>,
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

        // useragent parser
        let parser = UserAgentParser::builder()
            .with_unicode_support(false)
            .build_from_yaml("./src/user_agents/regexes.yaml")
            .expect("Parser creation failed");

        AppState {
            webauthn,
            db,
            ua_parser: Arc::new(parser),
        }
    }
}
