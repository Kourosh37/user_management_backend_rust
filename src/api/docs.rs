use crate::api::dto::auth::{LoginRequest, LoginResponse, RefreshRequest, RegisterRequest};
use crate::api::dto::user::{UpdateProfileRequest, UpdateUserRequest, UserResponse};
use crate::api::handlers::{auth, users};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::register_handler,
        auth::login_handler,
        auth::refresh_handler,
        auth::logout_handler,
        users::get_me_handler,
        users::update_me_handler,
        users::list_users_handler,
        users::get_user_handler,
        users::update_user_handler,
        users::deactivate_user_handler
    ),
    components(
        schemas(
            RegisterRequest,
            LoginRequest,
            RefreshRequest,
            LoginResponse,
            UserResponse,
            UpdateProfileRequest,
            UpdateUserRequest
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
