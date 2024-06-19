
use std::{collections::{HashMap, HashSet}, error::Error, sync::Arc};

use reqwest::get;
use sei_client::apis::_apis::{self, get_transaction_txs_by_event};
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, task};
use serde_json::{json, Map, Value};

extern crate chrono;
use chrono::prelude::*;


const LIST_EVENT:&str="wasm.sender";
const BUY_EVENT:&str="wasm.recipient";
const SELL_EVENT:&str="coin_received.receiver";
const OTHER_EVENT:&str="message.sender";

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash,Clone)]
pub struct  BuySellNft{
    pub token_id: String,
    pub ts: String,
    pub nft: String,
    pub collection_address: String,
    pub tx: String,
}


#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash,Clone)]
struct  WasmKeyValue{
    key: String,
    value: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash,Clone)]
pub struct  WasmBuyNow{
    pub _contract_address: String,
    pub collection_address:String,
    pub token_id:usize,
    pub buyer:String,
    pub seller:String,
    pub sale_price:String,
    pub marketplace_fee:String,
    pub royalties:String,
}

pub async fn get_holding_nfts<'nova_client>(wallet_address: String) -> Result<u32,Box<dyn Error>> {


        let mut wallet_holding_nfts:Vec<BuySellNft>=vec![];

        let events=vec![LIST_EVENT,BUY_EVENT,SELL_EVENT,OTHER_EVENT];

        let mut wallet_selling_nft_history:Vec<BuySellNft>=vec![];
        let wallet_selling_nft_history=Arc::new(Mutex::new(wallet_selling_nft_history));

        let mut wallet_buying_nft_history:Vec<BuySellNft>=vec![];
        let wallet_buying_nft_history=Arc::new(Mutex::new(wallet_buying_nft_history));


        let mut thread_handles:Vec<task::JoinHandle<()>>=vec![];

        let wallet_address=Arc::new(wallet_address);

        for event in events{

            let wallet_address=Arc::clone(&wallet_address);

            let wallet_selling_nft_history=Arc::clone(&wallet_selling_nft_history);
            let wallet_buying_nft_history=Arc::clone(&wallet_buying_nft_history);

            let hanlde=task::spawn(async move{

            let mut wallet_selling_nft_history=wallet_selling_nft_history.lock().await;
            let mut wallet_buying_nft_history=wallet_buying_nft_history.lock().await;
     
            
            match get_transaction_txs_by_event(event, &wallet_address).await {
                Ok(data)=>{
                   
                    let tx_responses=data.get("tx_responses").unwrap().as_array().unwrap();
                    
                    for tx_response in tx_responses{
                        let timestamp=tx_response.get("timestamp").unwrap().as_str().unwrap();
                        let txhash=tx_response.get("txhash").unwrap().as_str().unwrap();
                        let logs=tx_response.get("logs").unwrap().as_array().unwrap() ;//[0].as_object().unwrap();

                        for log in logs{
                            let log_ =log.as_object().unwrap().clone();
                            let log_events=log_.get("events").unwrap().as_array().unwrap();
                            
                            for log_event in log_events{
                                
                                let log_event=log_event.as_object().unwrap();
                                
                                // 获取 log event 为 wasm-buy_now 的 记录
                                if log_event.get("type").unwrap()=="wasm-buy_now"{
                                    
                                    let attributes:Vec<WasmKeyValue>=serde_json::from_value(log_event.get("attributes").unwrap().clone()).unwrap();
                                    let receive_event_nums=attributes.len()/8;
                                    for i in (1..receive_event_nums+1){
                                        let receive_event=&attributes[(i-1)*8..i*8];
                                        for arc_event in receive_event{
                                            //这里 可以解析 wasm buy now  || 后续迁移到field blockchain data
                                            if arc_event.key =="seller" && arc_event.value == *wallet_address{
                                                wallet_selling_nft_history.push(BuySellNft{
                                                    collection_address:receive_event[1].value.clone(),
                                                    nft:format!("{}-{}",receive_event[1].value,receive_event[2].value),
                                                    token_id:receive_event[2].value.clone(),
                                                    ts:timestamp.to_string(),
                                                    tx:txhash.to_string()
                                                });
                                            }
                                        }
                                    }
                                    // if attributes.len()==32{
                                    //     // println!("{:?}",attributes);
                                    //     println!("{}",txhash);
                                    // }
                                };

                                // 获取 log event 为 wasm 的 记录
                                if log_event.get("type").unwrap()=="wasm"{

                                    let attributes:Vec<WasmKeyValue>=serde_json::from_value(log_event.get("attributes").unwrap().clone()).unwrap();

                                    // action is  purchase_cart  batch_bids  || 排除交互合约
                                    if attributes.contains(&WasmKeyValue{key:"action".to_string(),value:"purchase_cart".to_string()}) ||
                                       attributes.contains(&WasmKeyValue{key:"action".to_string(),value:"batch_bids".to_string()}){
                                        
                                        //purchase_cart
                                        if attributes.contains(&WasmKeyValue{key:"action".to_string(),value:"purchase_cart".to_string()}){
                                            let receive_events=&attributes[2..];
                                            let receive_event_nums=receive_events.len()/5;

                                            for i in (1..receive_event_nums+1){
                                                let receive_event=&receive_events[(i-1)*5..i*5];
                                                for arc_event in receive_event{
                                                    if arc_event.key=="recipient".to_string() && arc_event.value==*wallet_address{
                                                        // println!("\n{:?}",&receive_event);
                                                        wallet_buying_nft_history.push(BuySellNft{
                                                            collection_address:receive_event[0].value.clone(),
                                                            nft:format!("{}-{}",receive_event[0].value,receive_event[4].value),
                                                            token_id:receive_event[4].value.clone(),
                                                            ts:timestamp.to_string(),
                                                            tx:txhash.to_string()
                                                        });
                                                    }
                                                }  

                                                // println!("{:?}",receive_event[4].value.clone());  //test nft id
                                                // println!("{:?}",txhash);
                                            }

                                            //batch_bids
                                        }else {
                                            let receive_events=&attributes[2..];
                                            let receive_event_nums=receive_events.len()/5;

                                            for i in (1..receive_event_nums+1){
                                                let receive_event=&receive_events[(i-1)*5..i*5];
                                                for arc_event in receive_event{
                                                    if arc_event.key=="recipient".to_string() && arc_event.value==*wallet_address{
                                                        // println!("\n{:?}",&receive_event);
                                                        wallet_buying_nft_history.push(BuySellNft{
                                                            collection_address:receive_event[0].value.clone(),
                                                            nft:format!("{}-{}",receive_event[0].value,receive_event[4].value),
                                                            token_id:receive_event[4].value.clone(),
                                                            ts:timestamp.to_string(),
                                                            tx:txhash.to_string()
                                                        });
                                                    }
                                                }  
                                                // println!("{:?}",receive_event[4].value.clone()); //test nft id
                                            }
                                        }

                                    // action is mint_nfte  ||  还没有搭建结构体
                                    }else if attributes.contains(&WasmKeyValue{key:"action".to_string(),value:"mint_nft".to_string()}) {
                                        let receive_event_nums=&attributes.len()/12;

                                        for i in (1..receive_event_nums+1){
                                            let receive_event=&attributes[(i-1)*12..i*12];
                                            for arc_event in receive_event{
                                                if arc_event.key=="recipient".to_string() && arc_event.value==*wallet_address{
                                                    // println!("\n{:?}",&receive_event);
                                                    wallet_buying_nft_history.push(BuySellNft{
                                                        collection_address:receive_event[2].value.clone(),
                                                        nft:format!("{}-{}",receive_event[2].value,receive_event[5].value),
                                                        token_id:receive_event[5].value.clone(),
                                                        ts:timestamp.to_string(),
                                                        tx:txhash.to_string()
                                                    });
                                                }
                                            }   
                                            // println!("{:?}",receive_event[5].value.clone());
                                        }
                                    }else {
                                        // action is tranfer_nft  || 没有交互合约
                                        let receive_event_nums=&attributes.len()/5;

                                        for i in (1..receive_event_nums+1){
                                            let receive_event=&attributes[(i-1)*5..i*5];
                                            for arc_event in receive_event{
                                                if arc_event.key=="recipient".to_string() && arc_event.value==*wallet_address{
                                                    // println!("\n{:?}",&receive_event);
                                                    wallet_buying_nft_history.push(BuySellNft{
                                                        collection_address:receive_event[0].value.clone(),
                                                        nft:format!("{}-{}",receive_event[0].value,receive_event[4].value),
                                                        token_id:receive_event[4].value.clone(),
                                                        ts:timestamp.to_string(),
                                                        tx:txhash.to_string()
                                                    });
                                                }
                                            }   

                                          
                                            // println!("{:?}",receive_event[4].value.clone());
                                            // println!("{:?}",txhash);
                                        }
                                    }
                                }
                            
                            };
                        }
                    }
                
                    // 测试上面最终返回数据 单
                    // println!("{:?}",wallet_buying_nft_history.len());
                    // println!("{:?}",wallet_selling_nft_history.len());
                    
                },
                Err(err) => {
                    eprintln!("post soccer data err -->  {:?} ",err);
                },

            };
                
               
                
                
            });
            thread_handles.push(hanlde);
        }
        
        for thread_handle in thread_handles{
            thread_handle.await.unwrap();
        }
        
        // 测试上面最终返回数据 all
        // println!("\n{:?}",wallet_buying_nft_history.lock().await);
        // println!("{:?}",wallet_selling_nft_history.lock().await.len());
        
        'check_holding:for buy_nft_info in &*wallet_buying_nft_history.lock().await{

            let sell_history=wallet_selling_nft_history.lock().await;

            if sell_history.len()==0{
                wallet_holding_nfts.push(buy_nft_info.clone());
            }else {
                for sell_nft_info in &*sell_history{
                    if sell_nft_info.nft==buy_nft_info.nft{
                        let buy_time=buy_nft_info.ts.parse::<DateTime<Utc>>().unwrap();
                        let sell_time=sell_nft_info.ts.parse::<DateTime<Utc>>().unwrap();
                        if buy_time<sell_time{
                            wallet_holding_nfts.push(buy_nft_info.clone());
                        }
                    }else {
                        wallet_holding_nfts.push(buy_nft_info.clone());
                    }
                }
            }
            
        };

        // println!("{:?}",wallet_holding_nfts);
        // for i in wallet_holding_nfts{
        //     println!("{}",i.nft)
        // }
        let mut seen = HashSet::new();
        wallet_holding_nfts.retain(|nft_info| seen.insert(nft_info.nft.clone()));  //过滤 相同数据  根据 nft

        let unique_json_str = serde_json::to_string_pretty(&wallet_holding_nfts).expect("Failed to serialize to JSON");

        println!("{:?}",wallet_holding_nfts.len());
        for i in wallet_holding_nfts{
            println!("{:?}",i.nft);
        }
      

    Ok(1)
}