use std::error::Error;

use reqwest::Client;

use crate::mappers::github_user_mapper;
use crate::{models::github_user::GithubUser, repositories::github_user_repository::GitHubUserRepository};


pub struct GitHubUserService {
    pub repository: GitHubUserRepository,
    pub client: Client
}

impl GitHubUserService {
    pub async fn get_by_username(&self, username: &str) -> Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> {
        let username_clone = username.clone();
        let stored_user_option = self.repository.get_by_username(username).await;
        let result_option: Result<Option<GithubUser>, Box<dyn Error + Send + Sync>> = match stored_user_option {
            Some(user) => {
                log::info!("Hit for GitHub user, username: {}!", username_clone);
                Ok(Some(github_user_mapper::to_model(&user)))
            },
            None => {
                log::info!("Miss for GitHub user, username: {}!", username_clone);
                let url = format!("https://api.github.com/users/{}", username_clone);
                log::info!("Making request to {}...", url);

                let response = self.client.get(url)
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
                let insert_result = self.repository.insert(github_user_mapper::to_entity(&user)).await;

                match insert_result {
                    Ok(()) => Ok(Some(user)),
                    Err(e) => {
                        log::error!("Failed to insert user: {}", user.login);
                        log::error!("{:?}", e);
                        return Err(String::from("Failed to insert user!"))?;
                    }
                }
            }
        };
        result_option
    }
}