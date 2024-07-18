extern crate base64;

use base64::encode;
// use futures::lock::Mutex;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::JoinHandle;
use std::{error::Error, sync::Arc};
use reqwest::{Client, Proxy};
use std::collections::HashMap;
use serde_json::{json, Map, Value};
use crate::field_data::field_data_structions::{self, Collection, NftAttribute, NftToken};
use crate::field_data::{data_structions::{AllNftInfo, Attributes}, field_data_structions::CollectionInfo};


// http://127.0.0.1:1317  http://173.0.55.178:1317
pub const BASEURL:&str="http://173.0.55.178:1317";


pub async fn get_transaction_txs_by_event<'nova_client>(
        events:&'nova_client str, 
        value:&'nova_client str,   
    ) -> Result <Map<String,Value>,Box<dyn Error>> {
        
        
        let url=format!("{}/cosmos/tx/v1beta1/txs?events={}=%27{}%27",BASEURL,events,value);
        // print!("{:?}",url);
        let client=Client::new();
        let rp_data:Value=client.get(url)
                                       .send().await?.json().await?;

        

        let data=rp_data.as_object().unwrap();
        if data.get("txs").unwrap().as_array().is_some() && data.get("tx_responses").unwrap().as_array().unwrap().len()!=0{
            Ok(data.clone())
        }else {
            Err("don't have data".into())
        }
    
    }

pub async fn get_transaction_txs_by_tx<'nova_client>(txhash:&'nova_client str) -> Result<Value,Box<dyn Error>> {
        
        let url=format!("{}/cosmos/tx/v1beta1/txs/{}",BASEURL,txhash);
        let client=&Client::new();
        let rp_data:Value=client.get(url)
                                     .send()
                                     .await?
                                     .json()
                                     .await?;
                                    // code

        let data: &Map<String, Value>=rp_data.as_object().unwrap();

        if data.get("code").is_none(){
            Ok(rp_data.clone())
        }else {
            
            Err("don't have data".into())
        }
        
    }

pub async fn get_last_height() -> Result<u64,Box<dyn Error>>  {
    
    let url=format!("{}/cosmos/base/tendermint/v1beta1/blocks/latest",BASEURL);
    let client=&Client::new();
    let rp_data:Value=client.get(url)
                                .send()
                                .await?
                                .json()
                                .await?;
                            // code

    let data=rp_data.as_object().unwrap();
    let block_data=data.get("block").unwrap().as_object().unwrap();
    let block_height=block_data.get("header").unwrap().as_object().unwrap().get("height").unwrap().clone();
    
    
    Ok(block_height.as_str().unwrap().parse::<u64>().unwrap())
}


pub async fn get_txs_by_block_height(height:u64) -> Result<Map<String,Value>,Box<dyn Error>> {

    let url=format!("{}/cosmos/tx/v1beta1/txs/block/{}",BASEURL,height);
    let client=&Client::new();
    let rp_data:Value=client.get(url)
                                .send()
                                .await?
                                .json()
                                .await?;
                            // code

    let data=rp_data.as_object().unwrap();
    Ok(data.clone())

}


pub async fn get_contract_info(contract_address:String) ->Option<CollectionInfo> {
        
        let mut rp_data:Vec<Value>=vec![];
        let rp_data=Arc::new(Mutex::new(rp_data));
        let client=Arc::new(Client::new());
       

        let mut thread_handles:Vec<JoinHandle<()>>=vec![];

        let query_datas=vec![
            encode(json!({"contract_info":{}}).to_string()),
            encode(json!({"minter":{}}).to_string()),            
            encode(json!({"num_tokens":{}}).to_string())];

        
        for query_data in query_datas{

            let contract_address=Arc::new(contract_address.clone());
            let client =Arc::clone(&client);
            let contraction_address=Arc::clone(&contract_address);
            let rp_data=Arc::clone(&rp_data);
            
            let handel=tokio::spawn(async move {
                let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",BASEURL,contract_address,query_data);
                match client.get(url).send().await{
                    Ok(data)=>{
                        let data:Value=data.json().await.expect("feild json err");
                        rp_data.lock().await.push(data);
                    },
                    Err(_)=>{},
                }
            });

            thread_handles.push(handel);
        }


        for thread_handle in thread_handles{
            thread_handle.await.unwrap();
        }

        let contrac_data=rp_data.lock().await.to_vec();

        let mut name = String::new();
        let mut symbol = String::new();
        let mut creator = String::new();
        let mut count = String::new();
    
        for value in contrac_data {
            if let Value::Object(map) = value {
                if let Some(data) = map.get("data") {
                    if let Value::Object(data_map) = data {
                        if let Some(Value::Number(num)) = data_map.get("count") {
                            count = num.to_string();
                        }
                        if let Some(Value::String(n)) = data_map.get("name") {
                            name = n.clone();
                        }
                        if let Some(Value::String(s)) = data_map.get("symbol") {
                            symbol = s.clone();
                        }
                        if let Some(Value::String(m)) = data_map.get("minter") {
                            creator = m.clone();
                        }
                    }
                }
            }
        }
    
        Some(CollectionInfo{
            name,
            symbol,
            creator,
            count,
        })
}


pub async fn get_nft_info(contract_address:String,token_id:String) ->Option<NftToken> {
        
        // test proxy
        // let proxy=Proxy::all("http://127.0.0.1:8080/").expect("msg");
        // let client=Client::builder().proxy(proxy).build().expect("msg");
        let client=Client::new();
        
        let query_data=encode(json!({"all_nft_info":{"token_id":token_id}}).to_string());

        let url=format!("{}/cosmwasm/wasm/v1/contract/{}/smart/{}",BASEURL,contract_address,query_data);
        
        match client.get(url).send().await {
            Ok(data)=>{
                
                let data:Value=data.json().await.expect("msg");
                let data=data.get("data");

                if data.is_none(){
                    return None;
                }
                let data=data.unwrap();
                if let Ok(nft_data)=serde_json::from_value::<AllNftInfo>(data.clone()){

                
                    // println!("{:?}",&nft_data.info.token_uri);
                    match client.get(nft_data.info.token_uri).send().await {
                        Ok(nft_info)=>{
                            let res: Result<_, reqwest::Error>=nft_info.json().await;
                            if res.is_ok(){

                                let nft_info_data:Value=res.unwrap();
                                if let Some(_attributes)=nft_info_data.get("attributes"){

                                        let _attributes=_attributes.clone();
                                
                                        let _attributes=serde_json::from_value::<Vec<NftAttribute>>(nft_info_data.get("attributes").unwrap().clone());
                                        let name=serde_json::from_value::<String>(nft_info_data.get("name").unwrap().clone());
                                        let image=serde_json::from_value::<String>(nft_info_data.get("image").unwrap().clone());
                                        
                                        if _attributes.is_ok() && name.is_ok() && image.is_ok(){
                                            let _attributes=_attributes.unwrap();
                                            let name=name.unwrap();
                                            let image=image.unwrap();
                                            Some(NftToken{
                                                token_id:token_id.clone(),
                                                name:name,
                                                key:format!("{}-{}",contract_address,token_id.clone()),
                                                image:image,
                                                royalty_percentage:nft_data.info.extension.royalty_percentage,
                                                attributes:_attributes,
                                            })
                                        }else {
                                            None
                                        }
                            }else {
                                None
                            }
                                    
                            }else {
                                None
                            }
                        },
                        Err(e)=>None,
                    }
                }else {
                    None
                }
            
            },
            Err(e)=>None,
        }

}

pub async fn get_all_nft_info(contract_address:String,max_concurrent_tasks: usize) -> Option<Collection> {

        let _contract_address=contract_address.clone();
        

        let collection=get_contract_info(contract_address.clone()).await.unwrap();


        let mut nft_tokens_info:Vec<NftToken>=vec![];
        let nft_tokens_info=Arc::new(Mutex::new(nft_tokens_info));

        let contract_address=Arc::new(contract_address);
        
        let semaphore = Arc::new(Semaphore::new(max_concurrent_tasks)); // 限制并发量

        let mut handles:Vec<JoinHandle<()>>=vec![];

        // println!("{:?}",contract_address);
        // collection.count.parse::<u64>().unwrap()
        for token_id in (1..=collection.count.parse::<u64>().unwrap()){
            println!("{:?}",token_id);
            let semaphore = Arc::clone(&semaphore);

            let nft_tokens_info=Arc::clone(&nft_tokens_info);
            let contract_address=Arc::clone(&contract_address);
            
            
            let handle=tokio::spawn(async move{
                let _permit = semaphore.acquire().await.expect("获取许可失败");
                let data=get_nft_info(contract_address.to_string(), token_id.to_string()).await.expect("msg");
                nft_tokens_info.lock().await.push(data);
                drop(_permit); // 显式释放许可
              
            });
            

            handles.push(handle);
        }

        for handle in handles{
            handle.await;
        }

        let nft_tokens=nft_tokens_info.lock().await.to_vec();
        
        Some(Collection{
            collection:_contract_address,
            name:collection.name,
            symbol:collection.symbol,
            creator:collection.creator,
            count:collection.count,
            nfts:nft_tokens,
        })

}


