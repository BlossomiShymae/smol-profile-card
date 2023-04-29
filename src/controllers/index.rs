use std::sync::Arc;
use axum::{response::Html};
use axum::extract::State;

use crate::AppState;
use crate::models::{empty::Empty};


pub async fn get_index(State(state): State<Arc<AppState>>) -> Html<String> {
    let data = super::TemplateViewModel {
        title: "Home".into(),
        body: state.registry.render("index", &Empty{}).unwrap(),
    };

    let r = state.registry.render("template", &data).unwrap();
    Html(r)
}