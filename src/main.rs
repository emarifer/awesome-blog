// #![allow(unused)] // silence unused warnings while exploring (to comment out)

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, Router},
};

use askama::Template;
use chrono::{DateTime, Datelike, Utc};
use dotenv;
use postgrest::Postgrest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tower_http::services::ServeDir;

// homepage template
// localhost:4000/
#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    home_title: String,
    home_links: &'a Vec<(String, String, String)>,
    year: i32,
}

// post template
// localhost:4000/post/:query_title
// Es necesario establecer escape = "none" en la plantilla
// para que no escape el Html que encuentre en el Markdown.
#[derive(Template)]
#[template(path = "posts.html", escape = "none")]
pub struct PostTemplate<'a> {
    post_title: &'a str,
    post_date: &'a DateTime<Utc>,
    post_body: &'a str,
    post_author: &'a str,
    post_reading_time: i8,
    post_avatar: &'a str,
    year: i32,
}

// SQL query will return all posts
// into a Vec<Post>
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    // #[serde(rename(deserialize = "postId"))]
    pub id: String,
    pub post_title: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub author: String,
    pub reading_time: i8,
    pub avatar: String,
}

// Our custom Askama filters
mod filters {

    use chrono::{DateTime, Utc};
    use chrono_tz::Europe::Madrid;
    use pulldown_cmark::{Options, Parser};

    // Filter to replace spaces with dashes in the title
    // now in our templates with can add tis filter e.g. {{ post_title|rmdash }}
    pub fn rmdashes(title: &str) -> askama::Result<String> {
        Ok(title.replace("-", " ").into())
    }

    pub fn frdate(created_at: &DateTime<Utc>) -> askama::Result<String> {
        let madrid_time = created_at.with_timezone(&Madrid);

        Ok(madrid_time.format("%H:%M • %d-%m-%Y").to_string())
    }

    // Filtro markdown personalizado. Si se usa el filtro markdown de Askama
    // hay que activar la feature de Askama en Cargo.toml
    pub fn frmarkdown(body: &str) -> askama::Result<String> {
        // used as part of pulldown_cmark for setting flags to enable extra
        // features - we're not going to use any of those, hence the `empty();`
        let options = Options::empty();

        let parser = Parser::new_ext(&body, options);

        // Write to a new String buffer.
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);

        Ok(html_output)
    }
}

// home router (homepage) will return all blog titles in anchor links
async fn home(State(state): State<Arc<Vec<Post>>>) -> impl IntoResponse {
    let s = state.clone();
    let mut plinks: Vec<(String, String, String)> = Vec::new();

    for i in 0..s.len() {
        plinks.push((
            format!("{}-{}", &s[i].post_title, &s[i].id),
            s[i].post_title.clone(),
            s[i].author.clone(),
        ));
    }

    let template = HomeTemplate {
        home_title: String::from("El Blog de los Caminantes"),
        home_links: &plinks,
        year: Utc::now().year(),
    };

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to render template. Error {}", err),
        )
            .into_response(),
    }
}

// post router uses two extractors
// Path to extract the query: localhost:4000/post/thispart
// State that holds a Vec<Post> used to render the post that the query matches
async fn post(
    Path(query_title): Path<String>,
    State(state): State<Arc<Vec<Post>>>,
) -> impl IntoResponse {
    let mut template = PostTemplate {
        post_title: "none",
        post_date: &DateTime::from(Utc::now()),
        post_body: "none",
        post_author: "none",
        post_reading_time: 0i8,
        post_avatar: "none",
        year: Utc::now().year(),
    };

    for item in state.iter() {
        if query_title == format!("{}-{}", &item.post_title, &item.id) {
            template = PostTemplate {
                post_title: &item.post_title,
                post_date: &item.created_at,
                post_body: &item.body,
                post_author: &item.author,
                post_reading_time: item.reading_time,
                post_avatar: &item.avatar,
                year: Utc::now().year(),
            };
            break;
        } else {
            continue;
        }
    }

    // 404 if no title found matching the user's query
    if &template.post_title == &"none" {
        return (StatusCode::NOT_FOUND, "404 not found").into_response();
    }

    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "try again later").into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let client = Postgrest::new(dotenv::var("SUPABASE_URL").unwrap())
        .insert_header("apikey", dotenv::var("SUPABASE_ANON_KEY").unwrap());

    let resp = client
        .from("posts")
        .select("*")
        .order("created_at.desc")
        .execute()
        .await?;

    let data = resp.text().await?;

    let mut posts: Vec<Post> = serde_json::from_str(&data)?;

    for post in &mut posts {
        post.post_title = post.post_title.replace(" ", "-");
    }

    let shared_state = Arc::new(posts);

    let app = Router::new()
        .route("/", get(home))
        .route("/post/:query_title", get(post))
        .with_state(shared_state)
        .nest_service("/static", ServeDir::new("static"));

    axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/*
 * https://www.youtube.com/watch?v=XZtlD_m59sM
 * https://mortenvistisen.com/posts/how-to-build-a-simple-blog-using-rust
 * https://spacedimp.com/blog/using-rust-axum-postgresql-and-tokio-to-build-a-blog/
 * https://fasterthanli.me/articles/a-new-website-for-2020
 * https://djc.github.io/askama/
 * https://codebeautify.org/minify-html
 * https://codevoweb.com/rust-crud-api-example-with-axum-and-postgresql/
 * https://codevoweb.com/?s=axum
 * https://codevoweb.com/ (para Axum o Actix)
 *
 * https://docs.rs/tower-http/latest/tower_http/cors/index.html
 */
