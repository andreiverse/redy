use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod article_controller;
pub mod auth_controller;
pub mod category_controller;
pub mod feed_category_controller;
pub mod feed_controller;
pub mod reader_controller;
pub mod user_controller;
pub mod user_feed_favorite_controller;
pub mod worker_controller;

pub fn create_controller() -> OpenApiRouter<AppState> {
    OpenApiRouter::<AppState>::new()
        .nest("/feed", feed_controller::router())
        .nest("/feed", feed_category_controller::router())
        .nest("/category", category_controller::router())
        .nest("/reader", reader_controller::router())
        .nest("/articles", article_controller::router())
        .nest("/auth", auth_controller::router())
        .nest("/favorites", user_feed_favorite_controller::router())
        .nest("/user", user_controller::router())
        .nest("/workers", worker_controller::router())
}
