use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GithubUser {
    pub id: i32,
    pub login: String,
    pub name: String,
    pub location: String,
    pub avatar_url: String,
}