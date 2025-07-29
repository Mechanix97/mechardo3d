use crate::data::blog_data::get_posts;
use axum::{Extension, extract::Path, response::Html};
use tera::{Context, Tera};

pub async fn blog(Extension(tera): Extension<Tera>) -> Html<String> {
    let posts = get_posts().expect("Error al cargar publicaciones");
    let mut context = Context::new();
    context.insert("titulo", "Blog");
    context.insert("posts", &posts);

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error al renderizar la plantilla");
    Html(rendered)
}

pub async fn blog_post(Extension(tera): Extension<Tera>, Path(id): Path<String>) -> Html<String> {
    let posts = get_posts().expect("Error al cargar publicaciones");
    let post = posts
        .iter()
        .find(|p| p.id == id)
        .expect("Publicación no encontrada");

    let mut context = Context::new();
    context.insert("titulo", &post.title);
    context.insert("post", &post);

    let rendered = tera
        .render("blog_post.html", &context)
        .expect("Error al renderizar la plantilla");
    Html(rendered)
}
