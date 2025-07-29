use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlogPost {
    pub id: String,
    pub title: String,
    pub content: String,
    pub date: String,
}
