use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod article_controller;
pub mod auth_controller;
pub mod feed_controller;
pub mod reader_controller;

pub fn create_controller() -> OpenApiRouter<AppState> {
    OpenApiRouter::<AppState>::new()
        .nest("/feed", feed_controller::router())
        .nest("/reader", reader_controller::router())
        .nest("/articles", article_controller::router())
        .nest("/auth", auth_controller::router())
}
