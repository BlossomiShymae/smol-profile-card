use std::sync::Arc;
use axum::http::StatusCode;
use axum::extract::{State, Query};
use axum::response::Html;
use serde::Deserialize;
use rusqlite::params;

use crate::AppState;
use crate::models::empty::{Empty};

#[derive(Debug, Deserialize)]
pub struct GithubUserViewModel {
    user: String
}

pub async fn get_index(query: Query<GithubUserViewModel>, State(state): State<Arc<AppState>>) -> (StatusCode, Html<String>) {
    let vm = query.0;
    let username = vm.user.to_string();

    // Attempt to use database cache before making request
    let user_option = state.github_user_repository
        .get_by_username(&username)
        .await;

    // Return a user based on option
    let user = match user_option {
        Some(user) => {
            log::info!("Hit for GitHub user with username: {}!", user.username);
            user.clone()
        },
        None => {
            log::info!("Miss for GitHub user with username: {}!", username);
            let url = format!("https://api.github.com/users/{}", username);
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
                    body: state.registry.render("errors/500", &Empty).unwrap(),
                };

                let r = state.registry.render("template", &data).unwrap();
                return (StatusCode::INTERNAL_SERVER_ERROR, Html(r));
            }
            
            let contents = match response.text().await {
                Ok(s) => s,
                Err(_) => panic!()
            };

            let user: crate::models::github_user::GithubUser = serde_json::from_str(&contents).unwrap();
            let entity = crate::entities::github_user::GithubUser {
                id: user.id,
                username: user.login,
                name: user.name,
                avatar_url: user.avatar_url,
                location: user.location
            };

            let s_entity = entity.clone();
            // Save entity to database cache
            state.conn.call(move |conn| {
                let query = "INSERT INTO GitHubUser (id, username, name, location, avatar_url) VALUES (?1, ?2, ?3, ?4, ?5)";
                conn.execute(query, params![
                    s_entity.id, 
                    s_entity.username, 
                    s_entity.name, 
                    s_entity.location, 
                    s_entity.avatar_url
                    ]
                ).unwrap();

                Ok(())
            }).await.unwrap();

            entity
        },
    };

    (StatusCode::OK, Html(user.location.into()))
}