pub mod index;
pub mod image;

use serde::{Serialize};



#[derive(Debug, Serialize)]
pub struct TemplateViewModel {
    pub title: String,
    pub body: String,
}