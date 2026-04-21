use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use anyhow::Context;
use tonic::transport::Server;
use tracing::info;

mod application;
mod data;
mod domain;
mod infrastructure;
mod presentation;
mod proto;
#[cfg(test)]
mod test_mocks;

use application::{AuthService, BlogService};
use data::{PostgresPostRepository, PostgresUserRepository};
use infrastructure::{
    database::{create_pool, run_migrations},
    jwt::JwtService,
    logging::init_logging,
};
use presentation::{
    grpc_service::BlogGrpcService,
    http_handlers::{create_post, delete_post, get_post, list_posts, login, register, update_post},
    middleware::jwt_validator,
};
use proto::blog::blog_service_server::BlogServiceServer;

const HTTP_PORT: u16 = 8080;
const GRPC_PORT: u16 = 50051;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logging();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let jwt_secret = std::env::var("JWT_SECRET").context("JWT_SECRET must be set")?;

    info!("Connecting to database...");
    let pool = create_pool(&database_url).await?;

    info!("Running migrations...");
    run_migrations(&pool).await?;

    let jwt_service = Arc::new(JwtService::new(&jwt_secret));
    let user_repo: Arc<dyn data::UserRepository> =
        Arc::new(PostgresUserRepository::new(pool.clone()));
    let post_repo: Arc<dyn data::PostRepository> =
        Arc::new(PostgresPostRepository::new(pool.clone()));

    let auth_service = Arc::new(AuthService::new(user_repo, jwt_service.clone()));
    let blog_service = Arc::new(BlogService::new(post_repo));

    let grpc_service = BlogGrpcService::new(
        auth_service.clone(),
        blog_service.clone(),
        jwt_service.clone(),
    );

    let grpc_addr = format!("0.0.0.0:{GRPC_PORT}").parse()?;
    info!("gRPC server listening on {grpc_addr}");
    let grpc_future = Server::builder()
        .add_service(BlogServiceServer::new(grpc_service))
        .serve(grpc_addr);

    let auth_service_http = auth_service.clone();
    let blog_service_http = blog_service.clone();
    let jwt_service_http = jwt_service.clone();

    info!("HTTP server listening on 0.0.0.0:{HTTP_PORT}");
    let http_future = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        let auth_mw = HttpAuthentication::bearer(jwt_validator);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(web::Data::new(auth_service_http.clone()))
            .app_data(web::Data::new(blog_service_http.clone()))
            .app_data(web::Data::new(jwt_service_http.clone()))
            // Public auth routes
            .service(
                web::scope("/api/auth")
                    .route("/register", web::post().to(register))
                    .route("/login", web::post().to(login)),
            )
            // Public read routes
            .service(
                web::scope("/api/posts")
                    .route("", web::get().to(list_posts))
                    .route("/{id}", web::get().to(get_post))
                    // Protected write routes
                    .service(
                        web::scope("")
                            .wrap(auth_mw)
                            .route("", web::post().to(create_post))
                            .route("/{id}", web::put().to(update_post))
                            .route("/{id}", web::delete().to(delete_post)),
                    ),
            )
    })
    .bind(format!("0.0.0.0:{HTTP_PORT}"))?
    .run();

    tokio::select! {
        result = http_future => {
            result?;
        }
        result = grpc_future => {
            result?;
        }
    }

    Ok(())
}
