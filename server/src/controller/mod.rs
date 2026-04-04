use utoipa_axum::router::OpenApiRouter;

use crate::AppState;

pub mod reader_controller;
pub mod feed_controller;
pub mod sessions_controller;

pub fn create_controller() -> OpenApiRouter<AppState> {
    OpenApiRouter::<AppState>::new()
        .nest("/feed", feed_controller::router())
        .nest("/reader", reader_controller::router())
        .nest("/sessions", sessions_controller::router())
}
