use crate::{
    auth::ExtractMe,
    models::{Authenticator, User},
    state::AppState,
};
use async_graphql::{
    http::GraphiQLSource, ComplexObject, Context, EmptyMutation, EmptySubscription, Json, Object,
    Schema,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    response::{self, IntoResponse},
    Extension,
};
use webauthn_rs::prelude::Passkey;

// graphiql handler
pub async fn graphiql() -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub type GraphQLSchema = Schema<Query, EmptyMutation, EmptySubscription>;

// build schema and write (req independent) state to it
pub fn build_schema(app_state: AppState) -> GraphQLSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(app_state)
        .finish()
}

// add req based data to the context
pub async fn graphql_handler(
    schema: Extension<GraphQLSchema>,
    ExtractMe(me): ExtractMe,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut req = req.into_inner();
    if let Some(me) = me {
        req = req.data(me);
    }
    schema.execute(req).await.into()
}

// impl resolvers for our types

#[ComplexObject]
impl User {
    async fn authenticators(&self, ctx: &async_graphql::Context<'_>) -> Vec<Authenticator> {
        let app_state = ctx.data::<crate::state::AppState>().unwrap();
        let me_id = self.id.clone();
        app_state
            .db
            .conn
            .call(move |conn| {
                crate::queries::get_authenticators_for_user_id(conn, me_id).map_err(|e| e.into())
            })
            .await
            .unwrap()
    }
}

#[ComplexObject]
impl Authenticator {
    async fn passkey(&self) -> Json<Passkey> {
        Json(self.passkey.clone())
    }
}

// root query
pub struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> &'static str {
        "🌍"
    }
    async fn me(&self, ctx: &Context<'_>) -> Option<User> {
        ctx.data_opt::<User>().cloned()
    }
}
