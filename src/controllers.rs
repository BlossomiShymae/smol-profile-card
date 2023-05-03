pub mod index;
pub mod image;

use axum::response::Html;
use reqwest::StatusCode;
use serde::{Serialize};
use handlebars::Handlebars;


#[derive(Debug, Serialize)]
pub struct TemplateViewModel {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
pub struct TemplateErrorViewModel {
    pub code: u16,
    pub reason: String,
}

pub async fn get_error_page(registry: &Handlebars<'static>, status_code: StatusCode) -> (StatusCode, Html<String>) {
    let status_title = get_status_title(status_code);
    let template_error_vm = TemplateErrorViewModel {
        code: status_code.as_u16(),
        reason: status_code.canonical_reason().unwrap_or_else(|| {""}).to_string()
    };
    let error_r = registry.render("errors/template", &template_error_vm).unwrap();

    let template_vm = TemplateViewModel {
        title: status_title,
        body: error_r,
    };
    let r = registry.render("template", &template_vm).unwrap();
    
    (status_code, Html(r))
}

fn get_status_title(status_code: StatusCode) -> String {
    let code = status_code.as_u16().to_string();
    let reason = status_code.canonical_reason().unwrap_or_else(|| {""}).to_string();
    [code, reason].join(" ")
}