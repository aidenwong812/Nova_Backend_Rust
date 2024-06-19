use std::error::Error;

use reqwest::{Client};

use std::collections::HashMap;
use serde_json::{Map, Value};


pub const BASEURL:&str="http://89.163.148.219:1317";


pub async fn get_transaction_txs_by_event<'nova_client>(
        events:&'nova_client str, 
        value:&'nova_client str,   
    ) -> Result <Map<String,Value>,Box<dyn Error>> {
        
        
        let url=format!("{}/cosmos/tx/v1beta1/txs?events={}=%27{}%27",BASEURL,events,value);
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

pub async fn get_transaction_txs_by_tx<'nova_client>(txhash:&'nova_client str) -> Result<Map<String,Value>,Box<dyn Error>> {
        
        let url=format!("{}/cosmos/tx/v1beta1/txs/{}",BASEURL,txhash);
        let client=&Client::new();
        let rp_data:Value=client.get(url)
                                     .send()
                                     .await?
                                     .json()
                                     .await?;
                                    // code

        let data=rp_data.as_object().unwrap();

        if data.get("code").is_none(){
            Ok(data.clone())
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