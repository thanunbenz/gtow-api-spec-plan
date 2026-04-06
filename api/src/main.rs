mod config;
mod routes;
mod services;
mod transform;
mod types;

use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

#[derive(Clone)]
pub enum DataSource {
    Local { path: PathBuf },
    S3 { client: aws_sdk_s3::Client, bucket: String, prefix: String },
}

pub struct AppState {
    pub data_source: DataSource,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::from_filename("config/.env").ok();
    dotenvy::dotenv().ok();
    env_logger::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let source_type = std::env::var("DATA_SOURCE").unwrap_or_else(|_| "local".to_string());

    let data_source = match source_type.as_str() {
        "s3" => {
            let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "ap-southeast-1".to_string());
            let bucket = std::env::var("S3_BUCKET").expect("S3_BUCKET is required when DATA_SOURCE=s3");
            let prefix = std::env::var("S3_PREFIX").unwrap_or_else(|_| "output".to_string());

            let aws_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(region))
                .load()
                .await;
            let client = aws_sdk_s3::Client::new(&aws_config);

            log::info!("Data source: S3 (bucket={bucket}, prefix={prefix})");
            DataSource::S3 { client, bucket, prefix }
        }
        _ => {
            let path = std::env::var("DATA_PATH").unwrap_or_else(|_| "output".to_string());
            log::info!("Data source: local ({path})");
            DataSource::Local { path: PathBuf::from(path) }
        }
    };

    log::info!("Starting GTO API server on port {port}");

    let state = web::Data::new(AppState { data_source });

    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route(
                "/v4/solutions/spot-solution",
                web::get().to(routes::preflop_solution::handle_spot_solution),
            )
            .route(
                "/v1/poker/next-actions",
                web::get().to(routes::next_actions::handle_next_actions),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
