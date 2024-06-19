mod params_structs;
mod routes;

use actix_web::{web, App, HttpServer};
use  routes::{get_holding_nfts};
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(||{
            
            App::new()
                .service(
                    web::scope("/user")
                        .service(get_holding_nfts)
                        // .service(inde2x)
                            )
                            // .route("/", web::post().to(index))
                    })
        .bind(("127.0.0.1", 8888))?
        .run()
        .await
}

