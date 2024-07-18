mod params_structs;
mod routes;

use actix_web::{web::{self, Data}, App, HttpServer};
use actix_cors::Cors;

use  routes::{get_holding_nfts, get_holding_nfts_top, get_income_nfts,get_nfts_trades_info};

use dotenv::dotenv;
use db::client_db;
use sqlx::PgPool;
use std::path::PathBuf;
use openssl::ssl::{SslAcceptor,SslFiletype,SslMethod};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    // load nova_db  connect pool
    dotenv().ok();
    let nova_database_url = std::env::var("DATABASE_URL").unwrap();
    let nova_db_pool=PgPool::connect(&nova_database_url).await.unwrap();

    // set SSL
    let mut ssl_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    ssl_builder.set_private_key_file("./openssl/key.pem", SslFiletype::PEM).unwrap();
    ssl_builder.set_certificate_chain_file("./openssl/cert.pem").unwrap();

    HttpServer::new(move ||{
            
            App::new()
                .app_data(Data::new(nova_db_pool.clone()))
                .wrap(
                    Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("https://localhost:3000")
                    .allowed_origin("https://novafrontend-dev.vercel.app")
                    .allow_any_header()
                    .allow_any_method()
                    .max_age(3600) 
                )
                .service(
                    web::scope("/nfts")
                        .service(get_holding_nfts)
                        .service(get_income_nfts)
                        .service(get_holding_nfts_top)
                        .service(get_nfts_trades_info)
                            )
                            
                    })
        .bind_openssl(("0.0.0.0", 9999),ssl_builder)?
        .run()
        .await
}

