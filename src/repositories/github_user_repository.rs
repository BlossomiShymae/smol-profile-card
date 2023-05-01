use rusqlite::params;
use tokio_rusqlite::Connection;

use crate::entities::github_user::GithubUser;


pub struct GitHubUserRepository {
    pub conn: Connection,
}

impl GitHubUserRepository {
    pub async fn get_by_username(&self, username: &str) -> Option<GithubUser> {
        let username_clone = username.to_string();
        self.conn.call(move |conn| {
            let query = format!("SELECT * FROM GitHubUser WHERE username = '{}'", username_clone);
            let mut stmt = conn.prepare(query.as_str()).unwrap();
            let users = stmt.query_map([], |row| {
                Ok(crate::entities::github_user::GithubUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    name: row.get(2)?,
                    location: row.get(3)?,
                    avatar_url: row.get(4)?,
                })
            })?.collect::<Result<Vec<crate::entities::github_user::GithubUser>, rusqlite::Error>>()?;
    
            Ok::<_, rusqlite::Error>(users)
        }).await.unwrap().first().cloned()
    }

    pub async fn insert(&self, entity: GithubUser) -> Result<(), tokio_rusqlite::Error> {
        let entity_clone = entity.clone();
        self.conn.call(move |conn| {
            let query = "INSERT INTO GitHubUser (id, username, name, location, avatar_url) VALUES (?1, ?2, ?3, ?4, ?5)";
            conn.execute(query, params![
                entity_clone.id, 
                entity_clone.username, 
                entity_clone.name, 
                entity_clone.location, 
                entity_clone.avatar_url
                ]
            ).unwrap();

            Ok(())
        }).await
    }
}