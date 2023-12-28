use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::{self, FromRow};

#[derive(Deserialize)]
struct CreateUserBody {
    username: String,
    password: String,
}

#[derive(Serialize, FromRow)]
struct UserNoPassword {
    id: i32,
    username: String,
}

#[derive(Deserialize)]
struct UpdateUserBody {
    username: String,
}

// Handler for GET /users
pub async fn get_users(state: web::Data<crate::AppState>) -> HttpResponse {
    match sqlx::query_as::<_, UserNoPassword>("SELECT id, username FROM users")
        .fetch_all(&state.db)
        .await
    {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

// Handler for GET /users/{id}
pub async fn get_user_by_id(
    state: web::Data<crate::AppState>,
    path: web::Path<(i32,)>,
) -> HttpResponse {
    let user_id = path.0;

    match sqlx::query_as::<_, UserNoPassword>(
        "SELECT id, username FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

// Handler for POST /users
pub async fn add_user(
    state: web::Data<crate::AppState>,
    body: web::Json<CreateUserBody>,
) -> HttpResponse {
    let user_data: CreateUserBody = body.into_inner();

    let hash_secret = std::env::var("HASH_SECRET").expect("HASH_SECRET must be set!");
    let mut hasher = argonautica::Hasher::default();
    let hash = hasher
        .with_password(user_data.password)
        .with_secret_key(hash_secret)
        .hash()
        .unwrap();

    match sqlx::query_as::<_, UserNoPassword>(
        "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id, username",
    )
    .bind(user_data.username)
    .bind(hash)
    .fetch_one(&state.db)
    .await
    {
        Ok(new_user) => HttpResponse::Created().json(new_user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

// Handler for PUT /users/{id}
pub async fn update_user(
    state: web::Data<crate::AppState>,
    path: web::Path<(i32,)>,
    body: web::Json<UpdateUserBody>,
) -> HttpResponse {
    let user_id = path.0;
    let new_username = &body.username;

    match sqlx::query_as::<_, UserNoPassword>(
        "UPDATE users SET username = $1 WHERE id = $2 RETURNING id, username",
    )
    .bind(new_username)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(updated_user) => HttpResponse::Ok().json(updated_user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}

// Handler for DELETE /users/{id}
pub async fn delete_user(
    state: web::Data<crate::AppState>,
    path: web::Path<(i32,)>,
) -> HttpResponse {
    let user_id = path.0;

    match sqlx::query_as::<_, UserNoPassword>(
        "DELETE FROM users WHERE id = $1 RETURNING id, username",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(deleted_user) => HttpResponse::Ok().json(deleted_user),
        Err(error) => HttpResponse::InternalServerError().json(format!("{:?}", error)),
    }
}
