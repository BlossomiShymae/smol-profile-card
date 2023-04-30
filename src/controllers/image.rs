use std::sync::Arc;
use axum::http::StatusCode;
use axum::extract::{State, Query};
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct GithubUserViewModel {
    user: String
}

pub async fn get_index(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> (StatusCode, String) {
    let vm = query.0;
    let url = format!("https://api.github.com/users/{}", vm.user.as_str());
    log::info!("{}", url);

    let response = match state.client.get(url)
        .header("User-Agent", "BlossomiShymae/smol-profile-card")
        .header("Accept", "application/json")
        .send()
        .await {
            Ok(r) => r,
            Err(_) => panic!()
        };
    if !response.status().is_success() {
        log::error!("{:?}", response.status());
        log::error!("{:?}", response.text().await.unwrap());
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".into());
    }
    
    let contents = match response.text().await {
        Ok(s) => s,
        Err(_) => panic!()
    };

    let user: crate::models::github_user::GithubUser = serde_json::from_str(&contents).unwrap();

    (StatusCode::OK, user.location.into())
}