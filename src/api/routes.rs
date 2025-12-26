use crate::api::docs::ApiDoc;
use crate::api::handlers::{auth, users};
use crate::AppState;
use axum::routing::{delete, get, patch, post};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn create_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register_handler))
        .route("/login", post(auth::login_handler))
        .route("/refresh", post(auth::refresh_handler))
        .route("/logout", post(auth::logout_handler));

    let user_routes = Router::new()
        .route("/me", get(users::get_me_handler).patch(users::update_me_handler))
        .route("/", get(users::list_users_handler))
        .route(
            "/:id",
            get(users::get_user_handler)
                .patch(users::update_user_handler)
                .delete(users::deactivate_user_handler),
        );

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/users", user_routes)
        .merge(SwaggerUi::new("/swagger").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .with_state(state)
}
