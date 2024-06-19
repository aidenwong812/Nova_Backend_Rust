use futures_util::{stream::StreamExt, SinkExt};
use std::error::Error;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {

    let ws_url="ws://89.163.148.219:26657/websocket";
    let (mut ws_stream, _) = connect_async(ws_url).await.expect("Failed to connect");

    println!("WebSocket connected");

    let (mut write, read) = ws_stream.split();
        
    // Subscribe to transaction events at height 3
    let query = r#"{"jsonrpc":"2.0","method":"subscribe","id":1,"params":["tm.event = 'Tx'"]}"#;
    
    write.send(Message::Text(query.to_string())).await.expect("msg");
    
   
    tokio::spawn(async move {
        let mut read = read;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Parse the incoming message
                  println!("{:?}",text);
            
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("WebSocket error: {:?}", e);
                }
            }
        }
    }).await.expect("msg");


    Ok(())
}