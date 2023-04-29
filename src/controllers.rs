pub mod index;

use serde::{Serialize};



#[derive(Debug, Serialize)]
pub struct TemplateViewModel {
    pub title: String,
    pub body: String,
}