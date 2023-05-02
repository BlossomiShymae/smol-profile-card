use crate::entities;
use crate::models;

pub fn to_entity(model: &models::github_user::GithubUser) -> entities::github_user::GithubUser {
    let model_clone = model.clone();
    entities::github_user::GithubUser {
        id: model_clone.id,
        username: model_clone.login,
        avatar_url: model_clone.avatar_url,
        location: model_clone.location,
        name: model_clone.name
    }
}

pub fn to_model(entity: &entities::github_user::GithubUser) -> models::github_user::GithubUser {
    let entity_clone = entity.clone();
    models::github_user::GithubUser {
        id: entity_clone.id,
        name: entity_clone.name,
        avatar_url: entity_clone.avatar_url,
        location: entity_clone.location,
        login: entity_clone.username
    }
}