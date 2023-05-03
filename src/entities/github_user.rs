
#[derive(Debug, Clone)]
pub struct GithubUser {
    pub id: i32,
    pub username: String,
    pub name: Option<String>,
    pub location: Option<String>,
    pub avatar_url: String,
    pub expiration: i64,
}
