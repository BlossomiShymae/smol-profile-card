use std::sync::Arc;
use axum::http::StatusCode;
use axum::extract::{State, Query};
use axum::response::Html;
use serde::Deserialize;

use crate::AppState;


#[derive(Debug, Deserialize)]
pub struct GithubUserViewModel {
    user: String
}

#[axum_macros::debug_handler]
pub async fn get_index(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    let vm = query.0;
    let username = vm.user.to_string();

    let user_result = state.github_user_service
        .get_by_username(&username)
        .await;

    if let Ok(user_option) = user_result {
        if let Some(user) = user_option {
            return (StatusCode::OK, Html(user.location.into()));
        }
    }

    super::get_error_page(&state.registry, StatusCode::INTERNAL_SERVER_ERROR).await
}