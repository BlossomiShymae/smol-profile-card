use std::sync::Arc;
use axum::http::StatusCode;
use axum::extract::{State, Query};
use axum::response::Html;
use serde::Deserialize;

use crate::AppState;
use crate::models::empty::{Empty};

#[derive(Debug, Deserialize)]
pub struct GithubUserViewModel {
    user: String
}

pub async fn get_index(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> (StatusCode, Html<String>) {
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
        let data = super::TemplateViewModel {
            title: "500 Internal Server Error".into(),
            body: state.registry.render("errors/500", &Empty{}).unwrap(),
        };

        let r = state.registry.render("template", &data).unwrap();
        return (StatusCode::INTERNAL_SERVER_ERROR, Html(r));
    }
    
    let contents = match response.text().await {
        Ok(s) => s,
        Err(_) => panic!()
    };

    let user: crate::models::github_user::GithubUser = serde_json::from_str(&contents).unwrap();

    (StatusCode::OK, Html(user.location.into()))
}