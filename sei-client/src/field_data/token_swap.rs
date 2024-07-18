use serde_json::Value;
use super::field_data_structions::TokenSwap;
use super::{data_structions::{HashData,TxResponse,Log,Event,Attribute}};


pub fn swap_datas(hash_data:HashData) -> Vec<TokenSwap>{

    //定义高级swap的闭包
    let is_height_swap=|wasm:&Event| ->bool{
        wasm.attributes.iter().any(|attr|{
            if  attr.value=="execute_swap_and_action" || 
                attr.value=="dispatch_user_swap" ||
                attr.value=="dispatch_post_swap_action" ||
                attr.value=="execute_user_swap"{
                return true;
            }else {
                return  false;
            }
        })
    };


    let find_account=|log:&Log|->Option<String>{
        log.events.iter().find(|event|{event._type=="message"}).and_then(|event|{
            event.attributes.iter().find(|attr|{attr.key=="sender"}).and_then(|attr|{
                return Some(attr.value.clone());
            })
        })
    };

    let mut token_swap_datas:Vec<TokenSwap>=vec![];

    let tx=hash_data.tx_response.txhash;
    let ts=hash_data.tx_response.timestamp;
    let logs=hash_data.tx_response.logs;


    for log in &logs{
        
        let account=find_account(log).unwrap();
        //get wasm
        match log.events.iter().find(|event|{event._type=="wasm"}) {
            Some(wasm)=>{
                // 高级 token swap
                if is_height_swap(wasm){
                    let token_swap=height_token_swap(wasm.attributes.clone(), tx.clone(), ts.clone(), account);
                    token_swap_datas.push(token_swap);
                }else {
                    //低级 token swap
                    let token_swap=normal_token_swap(wasm.attributes.clone(), tx.clone(), ts.clone(), account);
                    token_swap_datas.push(token_swap);
                }
            },
            _=>continue,
        };        
    }

    return token_swap_datas;

}   

fn normal_token_swap(attributes:Vec<Attribute>,tx:String,ts:String,account:String)-> TokenSwap{

    let trade_type="token_swap".to_string();

    let mut swap_indexs:Vec<usize>=vec![];
    let trade_type="token_swap".to_string();
    
    for index in (0..attributes.len()){
        if attributes[index].value=="swap"{
            swap_indexs.push(index);
        }
    }

    let first_swap=swap_indexs[0];

    let last_swap=swap_indexs.last().unwrap().clone();

    let source_token=attributes[first_swap+3].value.clone();
    let source_amount=attributes[first_swap+5].value.clone();

    let target_token=attributes[last_swap+4].value.clone();
    let target_amount=attributes[last_swap+6].value.clone();
  
    return  TokenSwap{
            account:account,
            source_token:source_token,
            target_token:target_token,
            source_amount:source_amount,
            target_amount:target_amount,
            trade_type:trade_type,
            tx:tx,
            ts:ts,
        }
    

}


fn height_token_swap(attributes:Vec<Attribute>,tx:String,ts:String,account:String)->TokenSwap {
    
    let mut swap_indexs:Vec<usize>=vec![];
    let trade_type="token_swap".to_string();
    
    for index in (0..attributes.len()){
        if attributes[index].value=="swap"{
            swap_indexs.push(index);
        }
    }

    
    let first_swap=swap_indexs[0];
    let last_swap=swap_indexs.last().unwrap().clone();

    let source_token=attributes[first_swap+3].value.clone();
    let source_amount=attributes[first_swap+5].value.clone();

    let target_token=attributes[last_swap+4].value.clone();
    let target_amount=attributes[last_swap+6].value.clone();

    return TokenSwap{
        account,
        source_token,
        target_token,
        source_amount,
        target_amount,
        trade_type,
        tx,
        ts,
    };
    // let mut swap_counter=0;
    // for attr in &attributes{
    //     if attr.value=="swap"{
    //         break;
    //     }else {
    //         swap_counter+=1;
    //     }
    // }

    // let attributes=attributes[swap_counter-1..].to_vec();

    // let source_token=attributes[4].clone().value;
    // let source_amount=attributes[6].clone().value;

    // let mut counter=0;


    // // 获取 dispatch_transfer_funds_back_bank_send 定位
    // for attribute in &attributes{
    //     if attribute.value=="dispatch_transfer_funds_back_bank_send"{
    //         break;
    //     }else {
    //         counter +=1;
    //     }
    // }

    // //重新裁剪
    // let new_attributes=attributes.clone()[..counter-1].to_vec();

    // // 判断是否含有tranfer

    // if new_attributes[counter-5].value=="transfer"{
        
    //     let mut transfer_counter=0;

    //     loop {
    //         transfer_counter +=1;
    //         if new_attributes[counter-(1*transfer_counter)-(transfer_counter*4)].value !="transfer"{
    //             break;
    //         }
    //     }

    //     let offset=transfer_counter*5-1;

    //     let target_token=new_attributes[new_attributes.len()-offset-6].clone().value;
    //     let target_amount=new_attributes[new_attributes.len()-offset-4].clone().value;
        
    //     return TokenSwap{
    //         account,
    //         source_token,
    //         target_token,
    //         source_amount,
    //         target_amount,
    //         trade_type,
    //         tx,
    //         ts,
    //     };

    // }else {
    //     let target_token=new_attributes[new_attributes.len()-6].clone().value;
    //     let target_amount=new_attributes[new_attributes.len()-4].clone().value;
   
    // }





}