use crate::data::blog_data::get_posts;
use crate::models::blog_post::BlogPost;
use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn index(Extension(tera): Extension<Tera>) -> Html<String> {
    // Obtener posts del blog
    let posts = match get_posts() {
        Ok(mut posts) => {
            // Ordenar por fecha descendente y tomar los 3 más recientes
            posts.sort_by(|a, b| b.date.cmp(&a.date));
            posts.into_iter().take(3).collect::<Vec<BlogPost>>()
        }
        Err(e) => {
            eprintln!("Error loading blog posts from data/blog_posts.json: {}", e);
            Vec::new() // Usar lista vacía para no romper el renderizado
        }
    };

    // Configurar el contexto de Tera
    let mut context = Context::new();
    context.insert("title", "Mechardo3D - Inicio");
    context.insert("posts", &posts);

    // Log para depuración
    eprintln!("Context for index.html: {:?}", context);

    // Renderizar la plantilla
    match tera.render("index.html", &context) {
        Ok(rendered) => Html(rendered),
        Err(e) => {
            eprintln!("Error rendering index.html: {}", e);
            Html(format!(
                "<html><body><h1>Error rendering page</h1><p>{}</p></body></html>",
                e
            ))
        }
    }
}
