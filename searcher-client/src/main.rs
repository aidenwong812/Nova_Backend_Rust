use futures_util::{stream::StreamExt, SinkExt};
use serde_json::Value;
use tokio::sync::Mutex;
use std::{error::Error, thread};
use std::sync::mpsc::channel;
use tokio::{sync::{self,Semaphore, mpsc::channel as tokio_channel}, time::{sleep, Duration}};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use std::collections::HashMap;
use std::sync::Arc;
use  searcher_client::{   operate_db, return_nft_transaction_data, return_token_swap_data, send_token_swap_data, transcation_datas_routes, websocket_run};

use db::client_db;

#[tokio::main]
async fn main()  {

    let subscription_message = serde_json::json!({
        // "jsonrpc": "2.0",
        "id": 420,
        "method": "subscribe",
        "params": {
             "query":"tm.event='Tx'"
        }
    });

    // 89.163.148.219
    // 127.0.0.1
    let query="tm.event='Tx'";
    let ws_url = "ws://173.0.55.178:26657/websocket";

    let (hash_tx,hash_rx)=channel();
    let (nft_tx,nft_rx)=channel();
    let (token_tx,token_rx)=channel();

    let (token_swap_tx,token_swap_rx)=channel();
    

    let (nft_msg_tx,nft_msg_rx)=channel();

    let mut routes=HashMap::new();

    
    let conn=Arc::new(Mutex::new(client_db().await.unwrap()));


    routes.insert("nft".to_string(), nft_tx);
    routes.insert("token".to_string(), token_tx);

    // println!("{:?}",routes);

    tokio::spawn(async move {
        websocket_run(ws_url, query, hash_tx).await;
    });
    

    let routes_semaphore = Arc::new(Semaphore::new(2000));
    tokio::spawn(async move{
        transcation_datas_routes(hash_rx, routes,routes_semaphore).await;
    });



    let a=thread::spawn(move ||{
        send_token_swap_data(token_rx, token_swap_tx)
    });

    let conn1=Arc::clone(&conn);
    
    tokio::spawn(async move {
        return_token_swap_data(token_swap_rx, conn1).await;
    });

    
    let nft_msg_tx=Arc::new(nft_msg_tx);
    tokio::spawn(async move {
        return_nft_transaction_data(nft_rx,nft_msg_tx).await;
    });

    let conn2=Arc::clone(&conn);
    operate_db(nft_msg_rx,conn2).await;


    
}