use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct GithubUser {
    pub id: i32,
    pub login: String,
    pub name: Option<String>,
    pub location: Option<String>,
    pub avatar_url: String,
}