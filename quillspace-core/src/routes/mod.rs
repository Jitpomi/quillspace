pub mod analytics;
pub mod auth;
pub mod content;
pub mod pages;
pub mod sites;
pub mod templates;
pub mod tenants;
pub mod users;
pub mod security;

use axum::Router;
use crate::AppState;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::create_routes())
        .nest("/content", content::create_routes())
        .nest("/sites", sites::sites_router())
        .nest("/pages", pages::pages_router())
        .nest("/templates", templates::templates_router())
        .nest("/tenants", tenants::create_routes())
        .nest("/users", users::create_routes())
        .nest("/analytics", analytics::create_routes())
        .nest("/security", security::security_router())
}
