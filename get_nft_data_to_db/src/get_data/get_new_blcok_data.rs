use futures_util::{stream::StreamExt, SinkExt};
use std::error::Error;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;



mod tests{


    use std::sync::{Arc, Mutex};

    use super::*;

    #[tokio::test]
    async fn test()  {
        
            let ws_url = "ws://89.163.148.219:26657/websocket";
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
        
            // write.send(Message::Text(query.to_string())).await.expect("msg");
            // tokio::spawn(async move {
            //     // send_heartbeat(&mut ws_stream).await;
            //     write.send(Message::Ping(vec![])).await.unwrap();
            //     sleep(Duration::from_secs(30)).await;  // 每 30 秒发送一次
            // }).await.unwrap();
            
            // Sleep to keep the async main function running
            // sleep(Duration::from_secs(10)).await;
        
        
    
    }
}