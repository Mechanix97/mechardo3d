use crate::models::blog_post::BlogPost;
use serde_json;
use std::fs;
use std::io;

pub fn get_posts() -> Result<Vec<BlogPost>, io::Error> {
    let path = "data/blog_posts.json";
    let data = fs::read_to_string(path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("No se pudo leer el archivo {}: {}", path, e),
        )
    })?;
    let posts: Vec<BlogPost> = serde_json::from_str(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error al parsear JSON en {}: {}", path, e),
        )
    })?;
    Ok(posts)
}
