use std::time::Instant;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;

use crate::graphql::schema::AppSchema;

pub async fn graphql_handler(
    State(schema): State<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let start = Instant::now();
    let resp = schema.execute(req.into_inner()).await;
    let duration = start.elapsed().as_secs_f64();
    metrics::histogram!("graphql_request_duration_seconds").record(duration);
    resp.into()
}
