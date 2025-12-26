use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use serial_test::serial;
use tower::ServiceExt;
use user_management_backend_rust::api;
use user_management_backend_rust::config::AppConfig;
use user_management_backend_rust::infra::auth::jwt::JwtService;
use user_management_backend_rust::infra::db;
use user_management_backend_rust::infra::db::user_repo::SqlxUserRepository;
use user_management_backend_rust::AppState;

async fn setup_app() -> (AppState, axum::Router) {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
    let config = AppConfig {
        app_host: "0.0.0.0".to_string(),
        app_port: 0,
        database_url,
        jwt_secret: "test-secret".to_string(),
        access_token_minutes: 15,
        refresh_token_days: 7,
        cors_allowed_origins: vec!["http://localhost:3000".to_string()],
    };

    let pool = db::create_pool(&config.database_url)
        .await
        .expect("failed to create pool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let state = AppState {
        db: pool,
        config,
    };

    let app = api::routes::create_router(state.clone());
    (state, app)
}

async fn reset_db(state: &AppState) {
    sqlx::query("TRUNCATE TABLE users")
        .execute(&state.db)
        .await
        .expect("failed to truncate users");
}

#[tokio::test]
#[serial]
async fn register_login_and_profile_flow() {
    let (state, app) = setup_app().await;
    reset_db(&state).await;

    let register_body = json!({
        "email": "user@example.com",
        "username": "userone",
        "password": "password123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let login_body = json!({
        "email": "user@example.com",
        "password": "password123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let access_token = body.get("access_token").unwrap().as_str().unwrap();

    let response = app
        .oneshot(
            Request::get("/users/me")
                .header("authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn admin_list_users_flow() {
    let (state, app) = setup_app().await;
    reset_db(&state).await;

    let repo = SqlxUserRepository::new(state.db.clone());
    let jwt = JwtService::new(&state.config);

    let register_body = json!({
        "email": "admin@example.com",
        "username": "adminuser",
        "password": "password123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::post("/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let admin_user = repo
        .find_by_email("admin@example.com")
        .await
        .unwrap()
        .unwrap();
    let _ = repo
        .set_role(admin_user.user.id, user_management_backend_rust::domain::Role::Admin)
        .await
        .unwrap();

    let admin_user = repo
        .find_by_id(admin_user.user.id)
        .await
        .unwrap()
        .unwrap();
    let access_token = jwt.create_access_token(&admin_user.user).unwrap();

    let response = app
        .oneshot(
            Request::get("/users")
                .header("authorization", format!("Bearer {}", access_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
