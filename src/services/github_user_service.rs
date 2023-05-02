use std::error::Error;
use tokio::sync::Mutex;
use reqwest::Client;
use std::sync::Arc;

use crate::mappers::github_user_mapper;
use crate::{models::github_user::GithubUser, repositories::github_user_repository::GitHubUserRepository};


pub struct GitHubUserService {
    pub repository: GitHubUserRepository,
    pub client: Arc<Mutex<Client>>,
}

impl GitHubUserService {
    pub async fn get_by_username(&self, username: &str) -> Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> {
        let username_clone = username.clone();
        let stored_user_option = self.repository.get_by_username(username).await;
        let result_option: Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> = match stored_user_option {
            Some(user) => {
                // Check if user in database cache is expired
                let current_timestamp = chrono::prelude::Utc::now().timestamp_millis();
                if current_timestamp >= user.expiration {
                    // Miss
                    return self.update_user(username).await;
                }
                // Hit
                log::info!("Hit for GitHub user, username: {}!", username_clone);
                Ok(Some(github_user_mapper::to_model(&user)))
            },
            None => self.update_user(username).await
        };
        result_option
    }

    async fn update_user(&self, username: &str) -> Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> {
        log::info!("Miss for GitHub user, username: {}!", username);
        let url = format!("https://api.github.com/users/{}", username);
        log::info!("Making request to {}...", url);

        let client = self.client.lock().await;
        let response = client.get(url)
            .header("User-Agent", "BlossomiShymae/smol-profile-card")
            .header("Accept", "application/json")
            .send()
            .await
            .expect("Failed to get response");

        if !response.status().is_success() {
            log::error!("{:?}", response.status());
            log::error!("{:?}", response.text().await.unwrap());
            return Err(String::from("Failed to get response!"))?;
        }

        let contents = response.text().await.expect("Failed to get response!");
        let user: GithubUser = serde_json::from_str(&contents).expect("Failed to deserialize GithubUser");
        let upsert_result = self.repository.upsert(github_user_mapper::to_entity(&user)).await;

        match upsert_result {
            Ok(()) => Ok(Some(user)),
            Err(e) => {
                log::error!("Failed to upsert user: {}", user.login);
                log::error!("{:?}", e);
                return Err(String::from("Failed to upsert user!"))?;
            }
        }
    }
}