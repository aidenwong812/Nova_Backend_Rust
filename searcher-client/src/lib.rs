use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Arc;
use std::thread;
use sei_client::field_data::nft_transaction::{nft_transaction_data, NftMessage};
use sei_client::field_data::{
    data_structions::{HashData,Log},
    token_swap::swap_datas,
};

use sei_client::field_data::field_data_structions::{NFTtransaction, NftToken, TokenSwap, _NftTransaction};

use sqlx::PgConnection;
use tokio::sync::{Mutex, MutexGuard, Semaphore};
use tokio::{sync::{self, mpsc::channel as tokio_channel}, time::{sleep, Duration}};

use sei_client::apis::_apis::{get_nft_info, get_transaction_txs_by_tx};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{stream::StreamExt, SinkExt};

use serde_json::{Map, Value};

use db::{client_db, update_contract_create_auctions, update_nfts_holding, update_nfts_transactions, update_token_transaction};


            // token 的路由 logs 中 的events中的 event的 type 为 mint 或  coinbase  delegate 
            //  token 的路由 logs 中 的events中的 event的 type 为 wasm 里面的key 为swap   || pool swap
            
            // nft 路由   logs 中 的events中的 event的 type 为 wasm 里面的key 为  mint_nft |  bacth_bids  | transfer_nft  | purchase_cart
            //nft 路由   logs 中 的events中的 event的 type 为 wasm-buy_now  wasm-accept_bid  wasm-withdraw_bid  wasm-create_auction  wasm-place_bid

pub async fn transcation_datas_routes(mut hash_rx:Receiver<String>,routes:HashMap<String,Sender<HashData>>,semaphore:Arc<Semaphore>) { //Result<Map<String,Value>,Box<dyn Error>>{
  
    //定义闭包

    // token swap
    let is_token=|logs:&Vec<Log>| ->bool{
                logs.iter().any(|log|{
                    log.events.iter().any(|event|{
                        if event._type=="wasm".to_string(){
                            event.attributes.iter().any(|att|{
                                if att.value=="swap".to_string(){
                                    return true;
                                }else {
                                    return false;
                                }
                            })
                        }else {
                            return  false;
                        }
                    })
                })
    };
    

    let is_nft=|logs:&Vec<Log>| ->bool{
        logs.iter().any(|log|{
            log.events.iter().any(|event|{
                if  event._type=="wasm-accept_bid".to_string() ||                       //同意bid
                    event._type==" wasm-removed_token_from_offers".to_string() ||       // 交易成功后的处理
                    event._type=="wasm-create_auction".to_string() ||                   //创建list
                    event._type=="wasm-place_bid".to_string() ||                        //bid
                    event._type=="wasm-cancel_auction".to_string() ||                   //取消list 
                    event._type=="wasm-buy_now".to_string() ||                          //sales
                    event._type=="wasm-withdraw_bid".to_string()                        //取消bid
                    {
                        return true;
                }else if event._type=="wasm".to_string(){
                    event.attributes.iter().any(|att|{
                        if  att.value=="transfer_nft".to_string() || 
                            att.value=="batch_bids".to_string()   ||
                            att.value=="purchase_cart".to_string() ||
                            att.value=="mint_nft".to_string() {
                                return true;
                        }else {
                            return false;
                        }
                    })
                }else {
                    return false;
                }
            })
        })
    };

    
    let nft_data_sender=Arc::new(routes.get("nft").unwrap().clone());
    let token_data_sender=Arc::new(routes.get("token").unwrap().clone());
     
   
    while let Ok(hash) = hash_rx.recv() {

        let semaphore = Arc::clone(&semaphore);
        let token_data_sender = Arc::clone(&token_data_sender);
        let nft_data_sender=Arc::clone(&nft_data_sender);

        tokio::spawn(async move {

            //限制并发
            let permit = semaphore.acquire().await.unwrap();

            
            let hash_vale = get_transaction_txs_by_tx(&hash).await.unwrap();

            let hash_data =serde_json::from_value::<HashData>(hash_vale).unwrap();  //反序列化数据
            
            let transaction_status_code: &u64 =&hash_data.tx_response.code;

            if transaction_status_code==&0{

                let logs:&Vec<Log>=&hash_data.tx_response.logs;
                

                // token route
                if is_token(logs){
                    token_data_sender.send(hash_data).unwrap();
                    
                }else if is_nft(logs) {
                    // nft route
                    nft_data_sender.send(hash_data).unwrap();
                    // println!("{:?}",hash)

                }
            
            }
             //释放限制
             drop(permit);
            
           
        });
    }
}

// run wss  || retrun hash
pub async fn websocket_run(url:&str,query:&str,hash_sender:Sender<String>) -> Result<(),Box<dyn Error>> {

    let sub_msg=serde_json::json!({
        "jsonrpc": "2.0",
        "id": 420,
        "method": "subscribe",
        "params": {
             "query":query
        }
    });

    let unsub_msg=serde_json::json!({
        "jsonrpc": "2.0",
        "id": 420,
        "method": "unsubscribe",
       
    });


    match connect_async(url).await{
        
        Ok((mut ws_stream,_))=>{
         
            let (mut write, mut read) = ws_stream.split();
            write.send(Message::Text(sub_msg.to_string())).await?;

            tokio::spawn(async move{
                
                while let Some(msg)=read.next().await {
                    
                    match msg {
                        Ok(Message::Text(text))=>{
                            
                            let _data:Value=serde_json::from_str(&text).unwrap();
        
                            //定义 过滤 json 闭包  || 排除 投票
                            let is_aggreate_vote=|json:&Value| ->bool{
                                json.get("result")
                                    .and_then(|result| result.get("events"))
                                    .map_or(true, |event| !event.get("aggregate_vote.exchange_rates").is_some())
                                    
                            };


                            // 获取 tx hash 闭包
                            let get_hash=|json:&Value,keys:&Vec<&str>| ->Option<String>{
                                  keys.iter().fold(Some(json),|acc,&key|{

                                    acc.and_then(|inner| inner.get(key))
                                
                                }).and_then(|val| val.as_array().map(|hash_arr| {
                                    hash_arr.iter().filter_map(|v| v.as_str().map(|hash| hash.to_string())).collect()}))};

                            
                            // 解析 tx hash 路径
                            let keys_paths:&Vec<&str>=&vec!["result","events","tx.hash"];

                         
                            if let Some(hash) = get_hash(&_data,keys_paths) {
                                if is_aggreate_vote(&_data){
                                        // println!("{:?}",hash);
                                    hash_sender.send(hash).unwrap();
                                }

                            }else {
                                if let None =_data.get("result")  {
                                    // write.send(Message::Text(unsub_msg.to_string())).await.unwrap();
                                    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;  // 防止崩溃
                                    write.send(Message::Text(sub_msg.to_string())).await.unwrap();
                                }
                            }
                        },
                        
                        Ok(_)=>{},
                        
                        Err(e)=>{
                            eprintln!("WebSocket error: {:?}", e);
                        }
                    };
                   
                };

            }).await;
        },

        Err(e)=>eprintln!("WebSocket error: {:?}", e),
    
    }


    Ok(())
}





pub fn send_token_swap_data(mut token_rx:Receiver<HashData>,token_swpan_data_sender:Sender<Vec<TokenSwap>>) {
    
    let token_swpan_data_sender=Arc::new(token_swpan_data_sender);
    while let Ok(hash_data) =token_rx.recv()  {
        let token_swpan_data_sender=Arc::clone(&token_swpan_data_sender);
        thread::spawn( move || {
            let token_swap_data=swap_datas(hash_data);
            token_swpan_data_sender.send(token_swap_data).unwrap();
        }).join().unwrap();
        
    }
}

pub async fn return_token_swap_data(mut token_swap_data_re:Receiver<Vec<TokenSwap>>,conn:Arc<Mutex<PgConnection>>)  {
    
    while let Ok(token_swap_datas) =token_swap_data_re.recv()  {
        
        let conn=Arc::clone(&conn);
        tokio::spawn(async move {
            
            let mut conn=conn.lock().await;

            for token_swap_data in token_swap_datas{
                
                                
                    if let Some(_) =update_token_transaction(&token_swap_data.account, &mut conn, vec![token_swap_data.clone()]).await  {
                        println!("add token swap to db sucess");
                    }else {
                        println!("add token swap to db erro");
                    }

            }
        }).await.unwrap()
    }
}

pub async fn return_nft_transaction_data(mut nft_rx:Receiver<HashData>,mut nft_msg_tx:Arc<Sender<NftMessage>>) {


   

    while let Ok(hash_data) =nft_rx.recv()  {
        let nft_msg_tx=Arc::clone(&nft_msg_tx);    
        tokio::spawn(async move{
            nft_transaction_data(hash_data,nft_msg_tx).await;
        }).await.unwrap();
        
    }  
    
}


pub async fn operate_db(nft_msg_rx:Receiver<NftMessage>,conn:Arc<Mutex<PgConnection>>)  {
    println!("操作数据库");

    let mut  conn=conn.lock().await;
   
        for transaction in nft_msg_rx{

        // }
        // while let Ok(transaction) =nft_msg_rx.recv()  {
            println!("{:?}",transaction);
        
            match transaction {
                NftMessage::AcceptBidNft(msgs)=>{
                    for msg in msgs{
                        
 
                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::AcceptBid(msg.clone()),
                            _type:"accpet_bid_nft".to_string(),
                        };
                        
                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || accpet bid ")
                        }else {
                            println!("update db err || accpet bid  ")
                        }
                    }
                },
                NftMessage::CretaeAuctionNft(msgs)=>{
                    for msg in msgs{
                        
                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::CretaeAuction(msg.clone()),
                            _type:"create_auction_nft".to_string()
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || CretaeAuctionNft")
                        }else {
                            println!("update db eroo || CretaeAuctionNft")
                        }
                        
                    }
                },
                NftMessage::CancelAuctionNft(msgs)=>{
                    for msg in msgs{
                        
                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::CancelAuction(msg.clone()),
                            _type:"cancel_auction_nft".to_string()
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || CancelAuctionNft")
                        }else {
                            println!("update db eroo || CancelAuctionNft")
                        }
                        
                    }
                },
                NftMessage::OnlyTransferNft(msgs)=>{
                    for msg in msgs{

                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::OnlyTransfer(msg.clone()),
                            _type:"only_transfer_nft".to_string()
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.sender, &msg.collection, &msg.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.recipient, &msg.collection, &msg.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || OnlyTransferNft")
                        }else {
                            println!("update db eroo || OnlyTransferNft")
                        }

                        
                    }
                },
                NftMessage::BatchBids(msgs)=>{
                    for msg in msgs{

                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::BatchBids(msg.clone()),
                            _type:"batch_bids_nft".to_string()
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || BatchBids")
                        }else {
                            println!("update db eroo || BatchBids")
                        }

                        
                    }
                },
                NftMessage::PurchaseCartNft(msgs)=>{
                    for msg in msgs{

                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::PurchaseCart(msg.clone()),
                            _type:"purchase_cart_nft".to_string()
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;


                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || PurchaseCartNft")
                        }else {
                            println!("update db eroo || PurchaseCartNft")
                        }

                        
                    }
                },
                NftMessage::Mint(msgs)=>{
                    for msg in msgs{
                        
                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::Mint(msg.clone()),
                            _type:"mint_nft".to_string()
                        };

                      
                        let update_recipient_nft_holding=update_nfts_holding(&msg.recipient, &msg.collection, &msg.token_id, "add", &mut conn).await;
                        
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.recipient, &mut conn, vec![transaction.clone()]).await;


                        if  update_recipient_nft_holding.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || Mint")
                        }else {
                            println!("update db eroo || Mint")
                        }
                        
                    }
                },
                NftMessage::FixedSellNft(msgs)=>{
                    for msg in msgs{
                        
                        let transaction=NFTtransaction{
                            transaction:_NftTransaction::FixedSell(msg.clone()),
                            _type:"fixed_price".to_string(),
                        };

                        let update_sender_nft_holding=update_nfts_holding(&msg.transfer.sender, &msg.transfer.collection, &msg.transfer.token_id, "del", &mut conn).await;
                        let update_recipient_nft_holding=update_nfts_holding(&msg.transfer.recipient, &msg.transfer.collection, &msg.transfer.token_id, "add", &mut conn).await;
                        
                        let update_sender_nft_transactions=update_nfts_transactions(&msg.transfer.sender, &mut conn, vec![transaction.clone()]).await;
                        let update_recipient_nft_transactons=update_nfts_transactions(&msg.transfer.recipient, &mut conn, vec![transaction.clone()]).await;

                        if update_sender_nft_holding.is_some() && update_recipient_nft_holding.is_some() && update_sender_nft_transactions.is_some() && update_recipient_nft_transactons.is_some(){
                            println!("update db sucess || PurchaseCartNft")
                        }else {
                            println!("update db eroo || PurchaseCartNft")
                        }

                    }
                },
                NftMessage::OnlyCreateAuction(msgs)=>{
                    for msg in msgs{
                        let update_contranct_create_auctions=update_contract_create_auctions(&msg.collection_address, vec![msg.clone()], &mut conn).await;
                        if update_contranct_create_auctions.is_some(){
                            println!("update db sucess || OnlyCreateAuction")
                        }else {
                            println!("update db erro || OnlyCreateAuction")
                        }
                    }
                },
                NftMessage::Unkonw(hash)=>{
                    println!("unkonw : {:?}",hash);
                }
            }
        
        
        
        }
    
    
    
}














































































        // match transaction {
        
        //      NftMessage::Mint(msgs)=> {
                
                
        //         if let Some(mint_nft_msgs)=msgs{
                    
                    
        //             for mint_nft_msg in mint_nft_msgs{

                        
        //                 let nft_info=query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: mint_nft_msg.collection.clone(), token_id: mint_nft_msg.token_id.clone() }).await;
                            
                     
                            
                            // if let QueryContractSmartResult::NftToken(token_info) =nft_info  {
                                
                            //     let transaction=NFTtransaction{
                            //         transaction:_NftTransaction::Mint(mint_nft_msg.clone()),
                            //         _type:"mint_nft".to_string()
                            //     };

                                // let mut conn=conn.await;
                                // let inster_nft_holding=UserDb::operate(UserDb::NftHolding { wallet_address: mint_nft_msg.recipient.clone(), collection_account: mint_nft_msg.collection.clone(), nft: token_info, operate: "add".to_string() }, &mut conn).await;
                                // let inster_nft_transaction=UserDb::operate(UserDb::NftTransaction { wallet_address: mint_nft_msg.recipient.clone(), nfts_transaction:vec![] }, &mut conn).await;

                                // if let (UserDbOperate::InsertOrUpdate(update_nft_holding),UserDbOperate::InsertOrUpdate(update_transactions)) = (inster_nft_holding,inster_nft_transaction) {
                                //     if let (Ok(_),Ok(_)) =(update_nft_holding,update_transactions)  {
                                //         println!("update mint nft sucess");
                                //     }else {
                                //         println!("upte mint nft erro");
                                //     }
                                // }
                            // }
                        
                        
            //         }
            //     }
            //  },

            //  NftMessage::BatchBids(msgs)=>{
                
            //     if let Some(batch_bids_nft_msgs) =msgs  {
                    
            //         for batch_bid_nft_msg in batch_bids_nft_msgs{
                        
            //             let sneder=batch_bid_nft_msg.transfer.sender.clone();
            //             let recipient=batch_bid_nft_msg.transfer.recipient.clone();

            //             if let Ok(nft_info) =query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: batch_bid_nft_msg.transfer.collection.clone(), token_id: batch_bid_nft_msg.transfer.token_id.clone()}).await  {
                            
            //                 if let QueryContractSmartResult::NftToken(token_info)=nft_info{

            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::BatchBids(batch_bid_nft_msg.clone()),
            //                         _type:"batch_bid_nft".to_string(),
            //                     };
            //                     // let mut  conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address:sneder , collection_account: batch_bid_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate: "del".to_string() }, &mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: recipient, collection_account: batch_bid_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate:"add".to_string() }, &mut conn).await;

            //                     let update_nft_transactions_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: batch_bid_nft_msg.transfer.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transactions_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: batch_bid_nft_msg.transfer.recipient.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;

            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transactions_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transactions_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transactions_sender,update_nft_transactions_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transactions_sender_,update_nft_transactions_recipient_) {
            //                             println!("update batch bids nft sucess");
            //                         }else {
            //                             println!("update batch bids nft erro");
            //                         }
            //                     }
            //                 }
            //             }
            //         }
            //     }
            //  },
             
            //  NftMessage::OnlyTransferNft(msgs)=>{
                
            //     if let Some(only_transfer_msgs)=msgs{
                
            //         for only_transfer_msg in only_transfer_msgs{

            //             if let Ok(nft_info) =query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: only_transfer_msg.collection.clone(), token_id: only_transfer_msg.token_id.clone() }).await  {
                            
            //                 if let QueryContractSmartResult::NftToken(token_info) = nft_info {
                                
            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::OnlyTransfer(only_transfer_msg.clone()),
            //                         _type:"only_transfer_nft".to_string(),
            //                     };

            //                     // let mut  conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address: only_transfer_msg.sender.clone(), collection_account:only_transfer_msg.collection.clone() , nft: token_info.clone(), operate:"del".to_string() },&mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: only_transfer_msg.recipient.clone(), collection_account:only_transfer_msg.collection.clone(), nft: token_info.clone(), operate: "add".to_string() }, &mut conn).await;

            //                     let update_nft_transaction_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: only_transfer_msg.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transaction_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: only_transfer_msg.recipient.clone(), nfts_transaction: vec![transaction.clone()] },&mut conn).await;

            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transaction_sender,update_nft_transaction_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transaction_sender_,update_nft_transaction_recipient_) {
            //                             println!("update only transfer nft sucess");
            //                         }else {
            //                             println!("update only transfer nft erro");
            //                         }
            //                     }

            //                 }
            //             }

            //         }
            //     }
            //  },
             
            //  NftMessage::CretaeAuctionNft(msgs)=>{

            //     if let Some(create_auction_nft_msgs) =msgs  {
            //         for create_auction_msg in create_auction_nft_msgs{
            //             if let Ok(nft_info)=query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: create_auction_msg.transfer.collection.clone(), token_id:create_auction_msg.transfer.token_id.clone() }).await{
            //                 if let QueryContractSmartResult::NftToken(token_info) =nft_info  {
                                
            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::CretaeAuction(create_auction_msg.clone()),
            //                         _type:"create_auction_nft".to_string(),
            //                     };

            //                     // let mut  conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address: create_auction_msg.transfer.sender.clone(), collection_account: create_auction_msg.transfer.collection.clone(), nft: token_info.clone(), operate:"del".to_string() }, &mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: create_auction_msg.transfer.recipient.clone(), collection_account: create_auction_msg.transfer.collection.clone(), nft: token_info.clone(), operate: "add".to_string() }, &mut conn).await;

            //                     let update_nft_transaction_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: create_auction_msg.transfer.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transaction_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: create_auction_msg.transfer.recipient.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
                                
            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transaction_sender,update_nft_transaction_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transaction_sender_,update_nft_transaction_recipient_) {
            //                             println!("update create auction nft sucess");
            //                         }else {
            //                             println!("update create auction nft erro");
            //                         }
            //                     }

            //                 }
            //             }
            //         }
            //     }
            //  },
             
            //  NftMessage::CancelAuctionNft(msgs)=>{
            //     if let Some(cancel_auction_nft_msgs) =msgs  {
            //         for cancel_auction_nft_msg in cancel_auction_nft_msgs{
            //             if let Ok(nft_info)=query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: cancel_auction_nft_msg.transfer.collection.clone(), token_id:cancel_auction_nft_msg.transfer.token_id.clone() }).await{
            //                 if let QueryContractSmartResult::NftToken(token_info) =nft_info  {
                                
            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::CancelAuction(cancel_auction_nft_msg.clone()),
            //                         _type:"cancel_auction_nft".to_string(),
            //                     };

            //                     // let mut  conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address: cancel_auction_nft_msg.transfer.sender.clone(), collection_account: cancel_auction_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate:"del".to_string() }, &mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: cancel_auction_nft_msg.transfer.recipient.clone(), collection_account: cancel_auction_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate: "add".to_string() }, &mut conn).await;

            //                     let update_nft_transaction_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: cancel_auction_nft_msg.transfer.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transaction_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: cancel_auction_nft_msg.transfer.recipient.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
                                
            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transaction_sender,update_nft_transaction_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transaction_sender_,update_nft_transaction_recipient_) {
            //                             println!("update cancel nft sucess");
            //                         }else {
            //                             println!("update cancel nft erro");
            //                         }
            //                     }

            //                 }
            //             }
            //         }
            //     }
            //  },
             
            //  NftMessage::PurchaseCartNft(msgs)=>{
            //     if let Some(purchase_cart_nft_msgs) =msgs  {
            //         for purchase_cart_nft_msg in purchase_cart_nft_msgs{
            //             if let Ok(nft_info) =query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: purchase_cart_nft_msg.transfer.collection.clone(), token_id: purchase_cart_nft_msg.transfer.token_id.clone() }).await  {
            //                 if let QueryContractSmartResult::NftToken(token_info) =nft_info  {
                                
            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::PurchaseCart(purchase_cart_nft_msg.clone()),
            //                         _type:"purchase_cart_nft".to_string(),
            //                     };

            //                     // let mut conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address: purchase_cart_nft_msg.transfer.sender.clone(), collection_account: purchase_cart_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate:"del".to_string() }, &mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: purchase_cart_nft_msg.transfer.recipient.clone(), collection_account: purchase_cart_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate: "add".to_string() }, &mut conn).await;

            //                     let update_nft_transaction_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: purchase_cart_nft_msg.transfer.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transaction_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: purchase_cart_nft_msg.transfer.recipient.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
                                
            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transaction_sender,update_nft_transaction_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transaction_sender_,update_nft_transaction_recipient_) {
            //                             println!("update purchase cart nft sucess");
            //                         }else {
            //                             println!("update purchase cart nft erro");
            //                         }
            //                     }
            //                 }
            //             }
            //         }
            //     }
            //  },
             
            //  NftMessage::AcceptBidNft(msgs)=>{
            //     if let Some(accept_bid_nft_msgs) =msgs  {
            //         for accept_bid_nft_msg in accept_bid_nft_msgs{
            //             if let Ok(nft_info) =query_contract_smart_type::get_data(query_contract_smart_type::searcher_nft_token_info { contract_address: accept_bid_nft_msg.transfer.collection.clone(), token_id: accept_bid_nft_msg.transfer.token_id.clone() }).await  {
            //                 if let QueryContractSmartResult::NftToken(token_info) =nft_info  {
                                
            //                     let transaction=NFTtransaction{
            //                         transaction:_NftTransaction::AcceptBid(accept_bid_nft_msg.clone()),
            //                         _type:"accpet_bid_nft".to_string(),
            //                     };

            //                     // let mut conn=conn.await;
            //                     let update_nft_holding_sender=UserDb::operate(UserDb::NftHolding { wallet_address: accept_bid_nft_msg.transfer.sender.clone(), collection_account: accept_bid_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate:"del".to_string() }, &mut conn).await;
            //                     let update_nft_holding_recipient=UserDb::operate(UserDb::NftHolding { wallet_address: accept_bid_nft_msg.transfer.recipient.clone(), collection_account: accept_bid_nft_msg.transfer.collection.clone(), nft: token_info.clone(), operate: "add".to_string() }, &mut conn).await;

            //                     let update_nft_transaction_sender=UserDb::operate(UserDb::NftTransaction { wallet_address: accept_bid_nft_msg.transfer.sender.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
            //                     let update_nft_transaction_recipient=UserDb::operate(UserDb::NftTransaction { wallet_address: accept_bid_nft_msg.transfer.recipient.clone(), nfts_transaction: vec![transaction.clone()] }, &mut conn).await;
                                
            //                     if let (UserDbOperate::InsertOrUpdate(update_nft_holding_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_holding_recipient_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_sender_),
            //                             UserDbOperate::InsertOrUpdate(update_nft_transaction_recipient_),
            //                             ) = (update_nft_holding_sender,update_nft_holding_recipient,update_nft_transaction_sender,update_nft_transaction_recipient) {
            //                         if let (Ok(_),Ok(_),Ok(_),Ok(_)) = (update_nft_holding_sender_,update_nft_holding_recipient_,update_nft_transaction_sender_,update_nft_transaction_recipient_) {
            //                             println!("update accpet bid nft sucess");
            //                         }else {
            //                             println!("update accpet  bid nft erro");
            //                         }
            //                     }

            //                 }
            //             }
            //         }
            //     }
            //  },
             
            //  NftMessage::Unkonw(msgs)=>{
            //     println!("xxxxxxx{:?}",msgs)
            //  }, 
        //      _=>{},
        // }
    
