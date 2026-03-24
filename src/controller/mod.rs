use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod reader_controller;
pub mod rss_feed_controller;

pub fn create_controller() -> OpenApiRouter<AppState> {
    OpenApiRouter::<AppState>::new()
        .nest("/rss_feed", rss_feed_controller::router())
        .nest("/reader", reader_controller::router())
}
