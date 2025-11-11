use crate::data::blog_data::get_posts;
use crate::language::Language;
use axum::{Extension, extract::Path, response::Html};
use rand::rng;
use rand::seq::IteratorRandom;
use std::fs;
use tera::{Context, Tera};

pub async fn blog(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut posts = get_posts().expect("Error loading posts");
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "Blog");
    context.insert("posts", &posts);

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn blog_post(
    Path((lang, id)): Path<(String, String)>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let posts = get_posts().expect("Error loading posts");
    let post: &crate::models::blog_post::BlogPost = posts
        .iter()
        .find(|p: &&crate::models::blog_post::BlogPost| p.id == id)
        .expect("Post not found");

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", &post.title);
    context.insert("post", &post);

    if let Some(post_content_route) = &post.route {
        let content = fs::read_to_string(format!("templates/blog/{}", post_content_route))
            .expect("Post content not found");
        context.insert("content", &content);
    } else {
        context.insert("content", &post.summary);
    }

    let related_posts: Vec<_> = posts
        .iter()
        .filter(|p| p.id != id)
        .choose_multiple(&mut rng(), 2);

    context.insert("related_posts", &related_posts);

    let rendered = tera
        .render("blog_post.html", &context)
        .expect("Error rendering post");
    Html(rendered)
}
