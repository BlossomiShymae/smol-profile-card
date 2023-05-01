pub mod index;
pub mod image;

use axum::response::Html;
use reqwest::StatusCode;
use serde::{Serialize};
use handlebars::Handlebars;

use crate::models::empty::Empty;


#[derive(Debug, Serialize)]
pub struct TemplateViewModel {
    pub title: String,
    pub body: String,
}

pub async fn get_error_page(registry: &Handlebars<'static>, status_code: StatusCode) -> (StatusCode, Html<String>) {
    let data_tuple: (&str, &str) = match status_code {
        StatusCode::INTERNAL_SERVER_ERROR => ("500 Internal Server Error", "errors/500"),
        _ => panic!("Status code not implemented")
    };
    let template_vm = TemplateViewModel {
        title: data_tuple.0.into(),
        body: registry.render(data_tuple.1.into(), &Empty).unwrap(),
    };
    let r = registry.render("template", &template_vm).unwrap();
    
    (status_code, Html(r))
}