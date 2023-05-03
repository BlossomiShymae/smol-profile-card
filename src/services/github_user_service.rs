use std::cmp::Ordering;
use std::error::Error;
use axum::http::{HeaderMap};
use tokio::sync::Mutex;
use reqwest::Client;
use std::sync::Arc;
use urlencoding::encode;

use crate::mappers::github_user_mapper;
use crate::{models::github_user::GithubUser, repositories::github_user_repository::GithubUserRepository};
use crate::time;


pub struct GithubUserService {
    pub repository: GithubUserRepository,
    pub client: Arc<Mutex<Client>>,
    pub image_client: Arc<Client>,
    pub remaining: Arc<Mutex<i64>>,
    pub reset: Arc<Mutex<i64>>,
    pub retry_after: Arc<Mutex<i64>>,
}

impl GithubUserService {
    pub async fn get_by_username(&self, username: &str) -> Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> {
        let username_clone = username.clone();
        let stored_user_option = self.repository.get_by_username(username).await;
        let result_option: Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> = match stored_user_option {
            Some(user) => {
                // Check if user in database cache is expired
                let current_timestamp = chrono::prelude::Utc::now().timestamp_millis();
                if current_timestamp >= user.expiration {
                    // Miss
                    log::info!("Expired timestamp, username: {}!", username_clone);
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

    pub async fn get_avatar_by_id(&self, id: i32) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let url = format!("https://avatars.githubusercontent.com/u/{}?v=4", id);
        let response = self.image_client.get(url)
            .header("User-Agent", "BlossomiShymae/smol-profile-card")
            .send()
            .await
            .expect("Failed to get avatar!");

        if !response.status().is_success() {
            log::error!("{:?}", response.status());
            log::error!("{:?}", response.text().await.unwrap());
            return Err(String::from("Failed to get response for avatar!"))?;
        }

        let data = response.bytes().await.expect("Failed to get bytes!");
        Ok(data.to_vec())
    }

    async fn update_user(&self, username: &str) -> Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> {
        log::info!("Miss for GitHub user, username: {}!", username);
        let url = format!("https://api.github.com/users/{}", encode(username));
        log::info!("Making request to {}...", url);

        let client = self.client.lock().await;
        let mut remaining = self.remaining.lock().await;
        let mut reset = self.reset.lock().await;
        let mut retry_after = self.retry_after.lock().await;
        // If retry after timestamp has not expired
        if retry_after.cmp(&time::get_timestamp()) == Ordering::Greater {
            log::warn!("Reached the secondary rate limit for GitHub!");
            return Err(String::from("Reached the secondary rate limit!"))?;
        }
        // If remaining requests is zero and the reset timestamp has not expired
        if remaining.cmp(&0) == Ordering::Equal && reset.cmp(&time::get_timestamp()) == Ordering::Greater {
            log::warn!("Reached the request limit for GitHub!");
            return Err(String::from("Reached the request limit!"))?;
        }
        let response = client.get(url)
            .header("User-Agent", "BlossomiShymae/smol-profile-card")
            .header("Accept", "application/json")
            .send()
            .await
            .expect("Failed to get response for user!");

        let header_map = response.headers();
        let new_remaining: i64 = GithubUserService::get_int(header_map, "x-ratelimit-remaining");
        let new_reset: i64 = GithubUserService::get_int(header_map, "x-ratelimit-reset");
        let new_retry_after: i64 = GithubUserService::get_int(header_map, "retry-after");
        *remaining = new_remaining;
        *reset = new_reset;
        *retry_after = time::get_timestamp() + new_retry_after;

        if !response.status().is_success() {
            log::error!("{:?}", response.status());
            log::error!("{:?}", response.text().await.unwrap());
            return Err(String::from("Failed to get response!"))?;
        }

        let contents = response.text().await.expect("Failed to get response!");
        let user: GithubUser = serde_json::from_str(&contents).expect("Failed to deserialize GithubUser");
        log::trace!("Upserting by login name: {}", user.login);
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

    fn get_int(header_map: &HeaderMap, key: &str) -> i64 {
        match header_map.get(key) {
            Some(value_option) => match value_option.to_str() {
                Ok(str) => match str.to_string().parse() {
                    Ok(value) => value,
                    Err(_) => 0
                },
                Err(_) => 0
            },
            None => 0
        }
    }
}