use std::{sync::{Arc}, net::{SocketAddr, IpAddr, Ipv4Addr}, str::FromStr};
use clap::Parser;
use axum::{routing::get, Router};
use axum::http::{Response, StatusCode};
use axum::body::{boxed, Body};
use handlebars::Handlebars;
use services::github_user_service::GitHubUserService;
use tokio::sync::Mutex;
use tokio_rusqlite::{Connection};
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use reqwest::Client;

pub mod models;
pub mod controllers;
pub mod entities;
pub mod repositories;
pub mod services;
pub mod mappers;
pub mod time;

use controllers::{index, image};
use repositories::github_user_repository::GitHubUserRepository;


// Command line interface
#[derive(Parser, Debug)]
#[clap(name="smol-profile-card", about="Another image generator server!")]
struct Opt {
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    #[clap(long = "static_dir", default_value = "static")]
    static_dir: String,
}

pub struct AppState {
    registry: Handlebars<'static>,
    client: Client,
    conn: Connection,
    github_user_service: GitHubUserService
}

#[tokio::main]
async fn main() {
    // Fetch console arguments
    let opt = Opt::parse();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level));
    }
    // Enable console logging
    tracing_subscriber::fmt::init();

    // Register templates
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("template", include_str!("templates/template.hbs")).unwrap();
    handlebars.register_template_string("index", include_str!("templates/index.hbs")).unwrap();
    handlebars.register_template_string("about", include_str!("templates/about.hbs")).unwrap();
    handlebars.register_template_string("errors/500", include_str!("templates/errors/500.hbs")).unwrap();
    handlebars.register_template_string("errors/503", include_str!("templates/errors/503.hbs")).unwrap();

    // Create database connection
    let conn = Connection::open("db.sqlite").await.unwrap_or_else(|err| {
        panic!("Failed to create a connection to database!\n{:?}", err);
    });
    conn.call(|conn| {
        conn.execute("CREATE TABLE IF NOT EXISTS GitHubUser (
            id          INTEGER PRIMARY KEY,
            username    TEXT NOT NULL COLLATE NOCASE,
            name        TEXT NOT NULL,
            location    TEXT NOT NULL,
            avatar_url  TEXT NOT NULL,
            expiration  INTEGER NOT NULL
        )",
        ()).unwrap_or_else(|err| {
        panic!("Failed to create table for GitHubUser!\n{:?}", err);
        });

        Ok(())
    }).await.unwrap_or_else(|err| {
        panic!("Something went wrong!\n{:?}", err);
    });

    // Create reqwest client
    let client = Client::new();

    // Setup repositories
    let github_user_repository = GitHubUserRepository {
        conn: conn.clone()
    };

    // Setup services
    let github_user_service = GitHubUserService {
        client: Arc::new(Mutex::new(client.clone())),
        image_client: Arc::new(client.clone()),
        repository: github_user_repository,
        remaining: Arc::new(Mutex::new(0)),
        reset: Arc::new(Mutex::new(0)),
        retry_after: Arc::new(Mutex::new(0))
    };

    // Setup controller routes and inject app state
    let app_state = Arc::new(AppState { 
        registry: handlebars,
        client: client,
        conn,
        github_user_service,
    });
    let app = Router::new()
        .route("/", get(index::get_index))
        .route("/about", get(index::get_about))
        .route("/image", get(image::get_index))
        .fallback_service(get(|req| async move {
            match ServeDir::new(opt.static_dir).oneshot(req).await {
                Ok(res) => res.map(boxed),
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .expect("error response"),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(app_state);

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port
    ));
    log::info!("Now listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
