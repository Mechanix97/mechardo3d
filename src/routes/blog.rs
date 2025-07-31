use crate::data::blog_data::get_posts;
use axum::{Extension, extract::Path, response::Html};
use tera::{Context, Tera};

pub async fn blog(Extension(tera): Extension<Tera>) -> Html<String> {
    let posts = get_posts().expect("Error loading posts");
    let mut context = Context::new();
    context.insert("title", "Blog");
    context.insert("posts", &posts);

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn blog_post(Extension(tera): Extension<Tera>, Path(id): Path<String>) -> Html<String> {
    let posts = get_posts().expect("Error loading posts");
    let post: &crate::models::blog_post::BlogPost = posts
        .iter()
        .find(|p: &&crate::models::blog_post::BlogPost| p.id == id)
        .expect("Post not found");

    // Seleccionar hasta 2 posts relacionados (excluyendo el post actual)
    let related_posts: Vec<_> = posts
        .iter()
        .filter(|p| p.id != id) // Excluir el post actual
        .take(2) // Limitar a 2 posts
        .collect();

    let mut context = Context::new();
    context.insert("title", &post.title);
    context.insert("post", &post);
    context.insert("related_posts", &related_posts);

    let rendered = tera
        .render("blog_post.html", &context)
        .expect("Error rendering post");
    Html(rendered)
}
