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
                    expiration: row.get(5)?
                })
            })?.collect::<Result<Vec<crate::entities::github_user::GithubUser>, rusqlite::Error>>()?;
    
            Ok::<_, rusqlite::Error>(users)
        }).await.unwrap().first().cloned()
    }

    pub async fn get_by_id(&self, id: i32) -> Option<GithubUser> {
        self.conn.call(move |conn| {
            let query = format!("SELECT * FROM GitHubUser WHERE id = {}", id);
            let mut stmt = conn.prepare(query.as_str()).unwrap();
            let users = stmt.query_map([], |row| {
                Ok(crate::entities::github_user::GithubUser {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    name: row.get(2)?,
                    location: row.get(3)?,
                    avatar_url: row.get(4)?,
                    expiration: row.get(5)?
                })
            })?.collect::<Result<Vec<crate::entities::github_user::GithubUser>, rusqlite::Error>>()?;

            Ok::<_, rusqlite::Error>(users)
        }).await.unwrap().first().cloned()
    }

    pub async fn upsert(&self, entity: GithubUser) -> Result<(), tokio_rusqlite::Error> {
        let insert_result = self.insert(entity.clone()).await;
        match insert_result {
            Ok(_) => Ok(()),
            Err(_) => self.update(entity.clone()).await
        }
    }

    pub async fn update(&self, entity: GithubUser) -> Result<(), tokio_rusqlite::Error> {
        let entity_clone = entity.clone();
        self.conn.call(move |conn| {
            let query = "UPDATE GitHubUser
                SET id = ?1,
                SET username = ?2,
                SET name = ?3,
                SET location = ?4,
                SET avatar_url = ?5,
                SET expiration = ?6,
                WHERE id = ?1";
            let execute_result = conn.execute(query, params![
                entity_clone.id,
                entity_clone.username,
                entity_clone.name,
                entity_clone.location,
                entity_clone.avatar_url,
                entity_clone.expiration
                ]
            );

            match execute_result {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        }).await
    }

    pub async fn insert(&self, entity: GithubUser) -> Result<(), tokio_rusqlite::Error> {
        let entity_clone = entity.clone();
        self.conn.call(move |conn| {
            let query = "INSERT INTO GitHubUser (id, username, name, location, avatar_url, expiration) VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
            let execute_result = conn.execute(query, params![
                entity_clone.id, 
                entity_clone.username, 
                entity_clone.name, 
                entity_clone.location, 
                entity_clone.avatar_url,
                entity_clone.expiration
                ]
            );

            match execute_result {
                Ok(_) => Ok(()),
                Err(e) => Err(e)
            }
        }).await
    }
}