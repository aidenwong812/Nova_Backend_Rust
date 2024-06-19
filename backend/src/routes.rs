
use actix_web::{get, post,web, HttpResponse, Responder, Result};

use crate::params_structs::{Wallet};



#[post("/get_holding_nfts")]
pub async fn get_holding_nfts(info: web::Json<Wallet>) -> Result<impl Responder> {

    
    Ok(format!("Welcome {}!", info.wallet_address))
}


// #[get("/users/{user_id}/{friend}")] // <- define path parameters
// pub async fn inde2x(path: web::Path<(u32, String)>) -> Result<String> {
//     let (user_id, friend) = path.into_inner();
//     Ok(format!("Welcome {}, user_id {}!", friend, user_id))
// }
