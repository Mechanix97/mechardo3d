use crate::data::blog_data::get_posts;
use axum::{Extension, extract::Path, response::Html};
use tera::{Context, Tera};

pub async fn blog(Extension(tera): Extension<Tera>) -> Html<String> {
    let posts = get_posts().expect("Error loading post");
    let mut context = Context::new();
    context.insert("titulo", "Blog");
    context.insert("posts", &posts);

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn blog_post(Extension(tera): Extension<Tera>, Path(id): Path<String>) -> Html<String> {
    let posts = get_posts().expect("Error loading posts");
    let post = posts.iter().find(|p| p.id == id).expect("Post not found");

    let mut context = Context::new();
    context.insert("titulo", &post.title);
    context.insert("post", &post);

    let rendered = tera
        .render("blog_post.html", &context)
        .expect("Error rendering post");
    Html(rendered)
}
