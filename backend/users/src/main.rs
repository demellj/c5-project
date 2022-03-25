use actix_cors::Cors;
use actix_web::middleware::Compress;
use actix_web::middleware::Logger;
use actix_web::middleware::NormalizePath;
use actix_web::{
    web::Data,
    App, HttpServer,
};
use env_logger;

use common::config::Config;
use common_web::database;
use common_web::router::RouteBuilder;

mod controllers;
mod requests;
mod responses;

use controllers::UserRouter;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let config = Config::load_dotenv().await?;
    let db_conn = Data::new(database::create_db_conn_pool(&config)?);
    let config = Data::new(config);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .wrap(Compress::default())
            .wrap(NormalizePath::trim())
            .app_data(db_conn.clone())
            .app_data(config.clone())
            .configure(|srv| {
                RouteBuilder::new(srv)
                    .extend::<UserRouter>("/api/v0/users")
                    .build();
            })
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
