pub mod auth;
pub mod connected_websites;
// pub mod consultations; // TODO: Fix calendly service dependencies

use axum::Router;
use crate::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::create_routes())
        .nest("/connected-websites", connected_websites::connected_websites_routes())
        // .nest("/consultations", consultations::consultation_routes()) // TODO: Fix calendly service
}
