use core::time;
use std::{collections::HashMap, sync::Arc};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use db::{client_db, search_contract_createauctions, search_user};
use sei_client::field_data::{data_structions::{Event, TxResponse}, field_data_structions::{Collection, NFTtransaction, _NftTransaction}};
use serde_json::{Map, Value};
use sqlx::PgConnection;
use sei_client::apis::_apis::get_transaction_txs_by_event;
use tokio::sync::{Mutex, Semaphore};
use super::response_structs::*;
use std::mem::MaybeUninit;


pub async fn get_user_nfts_holidng(wallet_address:&str,conn:&mut PgConnection ) -> Option<UserNftHolding>{
    
    
    
    let add_ck_hashmap=|holding_nfts:&Vec<Collection>,ck_hashmap:&mut HashMap<String,Vec<String>>| {
        
        holding_nfts.iter().for_each(|collection|{
            
            let mut nft_keys:Vec<String>=vec![];
            
            collection.nfts.iter().for_each(|nft|{
                nft_keys.push(nft.token_id.clone());
            });

            ck_hashmap.insert(collection.collection.clone(), nft_keys);
        })
    };

 
    if let Some(user_info)=search_user(wallet_address,  conn).await{
        

        if user_info.nfts_holding.len()>0{
            
            let mut hold_collection_keys:HashMap<String, Vec<String>>=HashMap::new();
            add_ck_hashmap(&user_info.nfts_holding,&mut hold_collection_keys);

            //获取 floor_price
            let collection_nfts_floor_prices=get_collection_nfts_floorprice(&hold_collection_keys).await;
            
            let mut user_collection_holding:Vec<UserCollectionHold>=vec![];

            for collection_holding in user_info.nfts_holding{
                
                let mut user_nft_holding:Vec<UserNft>=vec![];
                
                let floor_price_datas=collection_nfts_floor_prices.get(&collection_holding.collection).unwrap();
                
                let collection_floor_price=floor_price_datas.get(&collection_holding.collection).unwrap();
                // 创建 未初始化指针
                let mut floor_price:MaybeUninit<Option<String>>=MaybeUninit::uninit();
                let mut buy_price:MaybeUninit<Option<String>>=MaybeUninit::uninit();
                let mut royalties_fee:MaybeUninit<Option<String>>=MaybeUninit::uninit();
                let mut market_fee:MaybeUninit<Option<String>>=MaybeUninit::uninit();
                let mut unrealized_gains:MaybeUninit<Option<String>>=MaybeUninit::uninit();
                let mut ts:MaybeUninit<String>=MaybeUninit::uninit();
                let mut tx_hash:MaybeUninit<String>=MaybeUninit::uninit();

                let floor_price_ptr:*mut Option<String>=floor_price.as_mut_ptr();
                let buy_price_ptr:*mut Option<String>=buy_price.as_mut_ptr();
                let royalties_fee_ptr:*mut Option<String>=royalties_fee.as_mut_ptr();
                let market_fee_ptr:*mut Option<String>=market_fee.as_mut_ptr();
                let unrealized_gains_ptr:*mut Option<String>=unrealized_gains.as_mut_ptr();
                let ts_ptr:*mut String=ts.as_mut_ptr();
                let tx_hash_ptr:*mut String=tx_hash.as_mut_ptr();
               
                collection_holding.nfts.iter().for_each(|(nft)|{

                    // 获取 nft floor price
                    let nft_floor_price=floor_price_datas.get(&nft.token_id);
                    
                    unsafe {
                        if let Some(price)=nft_floor_price{
                            if price.is_some(){
                                floor_price_ptr.write(Some(price.to_owned().unwrap()));
                            }else {
                                if let Some(collection_floor_price) = floor_price_datas.get(&collection_holding.collection) {
                                    if collection_floor_price.is_some(){
                                        floor_price_ptr.write(Some(collection_floor_price.to_owned().unwrap()));
                                    }else {
                                        floor_price_ptr.write(None);
                                    }
                                }else {
                                    floor_price_ptr.write(None);
                                }
                            }
                        
                        }else {
                       
                            if let Some(collection_floor_price) = floor_price_datas.get(&collection_holding.collection) {
                                if collection_floor_price.is_some(){
                                    floor_price_ptr.write(Some(collection_floor_price.to_owned().unwrap()));
                                }else {
                                    floor_price_ptr.write(None);
                                }
                            }else {
                                floor_price_ptr.write(None);
                            }
                    
                        }
                    }
                    
                    //获取 buy_price sesf_fre  unrealized_gains
                    unsafe{
                        user_info.nfts_transactions.iter().for_each(|transaction|{
                            
                            match transaction.to_owned().transaction {
                                _NftTransaction::AcceptBid(data)=>{
                                    
                                    if &data.transfer.collection ==&collection_holding.collection && &data.transfer.token_id==&nft.token_id && &data.transfer.recipient==wallet_address{
                                        market_fee_ptr.write(Some(data.marketplace_fee));
                                        royalties_fee.write(Some(data.royalties));
                                        buy_price_ptr.write(Some(data.sale_price));
                                        ts_ptr.write(data.transfer.ts);
                                        tx_hash_ptr.write(data.transfer.tx);
                                 
                                    }
                                },
                                _NftTransaction::BatchBids(data)=>{
                                    
                                    if &data.transfer.collection ==&collection_holding.collection && &data.transfer.token_id==&nft.token_id && &data.transfer.recipient==wallet_address{
                                        let sale_price=data.sale_price.clone();
                                        let market_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                                        let market_fee=format!("{}usei",market_fee.to_string());
                                        
                                        market_fee_ptr.write(Some(market_fee));
                                        royalties_fee.write(None);
                                        buy_price_ptr.write(Some(data.sale_price));
                                        ts_ptr.write(data.transfer.ts);
                                        tx_hash_ptr.write(data.transfer.tx);
                                    }
                                },
                    
                                _NftTransaction::FixedSell(data)=>{
                                    
                                    if &data.transfer.collection ==&collection_holding.collection && &data.transfer.token_id==&nft.token_id && &data.transfer.recipient==wallet_address{
                                        let sale_price=format!("{}usei",data.price.clone());
                                        let market_fee=(sale_price.get(0..sale_price.len()-4).unwrap().parse::<f64>().unwrap() * 0.2) as u64;
                                        let market_fee=format!("{}usei",market_fee.to_string());
                                        
                                        market_fee_ptr.write(Some(market_fee));
                                        royalties_fee.write(None);
                                        buy_price_ptr.write(Some(sale_price));
                                        ts_ptr.write(data.transfer.ts);
                                        tx_hash_ptr.write(data.transfer.tx);
                                    }
                                },
                                _NftTransaction::Mint(data)=>{

                                    if &data.collection==&collection_holding.collection && &data.token_id==&nft.token_id && &data.recipient==wallet_address{
                                        market_fee_ptr.write(None);
                                        royalties_fee.write(None);
                                        buy_price_ptr.write(Some(format!("{}usei",data.price)));
                                        ts_ptr.write(data.ts);
                                        tx_hash_ptr.write(data.tx);
                                    }
                                    
                                },
                                _NftTransaction::OnlyTransfer(data)=>{
                                    
                                    if &data.collection ==&collection_holding.collection && &data.token_id==&nft.token_id && &data.recipient==wallet_address{
                                        market_fee_ptr.write(None);
                                        royalties_fee.write(None);
                                        buy_price_ptr.write(None);
                                        ts_ptr.write(data.ts);
                                        tx_hash_ptr.write(data.tx);
                                    }
                                },
                                _NftTransaction::PurchaseCart(data)=>{
                                    
                                    if &data.transfer.collection ==&collection_holding.collection && &data.transfer.token_id==&nft.token_id && &data.transfer.recipient==wallet_address{
                                        market_fee_ptr.write(Some(data.marketplace_fee));
                                        royalties_fee.write(Some(data.royalties));
                                        buy_price_ptr.write(Some(data.sale_price));
                                        ts_ptr.write(data.transfer.ts);
                                        tx_hash_ptr.write(data.transfer.tx);
                                    }
                                },
                                _=>{},
                            }
                        });

                        
                    };
           
                    unsafe{


                        let buy_price=(*buy_price_ptr).clone();
                      
                        let floor_price=(*floor_price_ptr).clone();

                        if floor_price.is_none(){
                             unrealized_gains_ptr.write(None)
                        }else if floor_price.is_some() && buy_price.is_some() {
                            
                            let bp=buy_price.unwrap();
                            let fp=floor_price.unwrap();

                            let bp=bp.get(0..bp.len()-4).unwrap().parse::<i64>().unwrap();
                            let fp=fp.get(0..fp.len()-4).unwrap().parse::<i64>().unwrap();
                            let ugp:i64= fp - bp;
                          

                            let ugp=format!("{}usei",ugp.to_string());
                            unrealized_gains_ptr.write(Some(ugp));
                        }else if floor_price.is_some() && buy_price.is_none() {
                            let ugp=floor_price.unwrap();
                            unrealized_gains_ptr.write(Some(ugp));
                        }

                        user_nft_holding.push(
                            UserNft { 
                                name: nft.name.clone(), 
                                token_id:nft.token_id.clone(), 
                                key: nft.key.clone(), 
                                image: nft.image.clone(), 
                                floor_price: (*floor_price_ptr).clone(), 
                                attributes: nft.attributes.clone(), 
                                buy_price: (*buy_price_ptr).clone(), 
                                royalties_fee: (*royalties_fee_ptr).clone(), 
                                market_fee: (*market_fee_ptr).clone(), 
                                unrealized_gains: (*unrealized_gains_ptr).clone(),
                                ts:(*ts_ptr).clone(),
                                tx_hash:(*tx_hash_ptr).clone()
                            }
                        );
                        
                    };

                });

                // println!("{:?}",user_nft_holding);

                user_collection_holding.push(
                    UserCollectionHold{
                        name:collection_holding.name.clone(),
                        symbol:collection_holding.symbol.clone(),
                        contract:collection_holding.collection.clone(),
                        creator:collection_holding.creator.clone(),
                        floor_price:collection_floor_price.clone(),
                        nfts_holding:user_nft_holding,
                    }
                );

                // drop ptr
                unsafe {
                    drop(floor_price_ptr);
                    drop(buy_price_ptr);
                    drop(royalties_fee_ptr);
                    drop(market_fee_ptr);
                    drop(unrealized_gains_ptr);
                    drop(ts_ptr);
                    drop(tx_hash_ptr);
                }
            }
            
            Some(
                UserNftHolding{
                    collections:user_collection_holding,
                }
            )

            // println!("{:?}",user_collection_holding);
        }else {
            None
        }
    }else {
        None
    }
}


// 在 holding nfts 的基础上，判断 unrealized_gains 大于0;
pub async fn get_user_income_holding_nfts(wallet_address:&str,conn:&mut PgConnection) -> Option<UserNftHolding> {
    
    if let Some(mut user_nfts_holding_data) =get_user_nfts_holidng(wallet_address, conn).await  {
        
        let  user_nfts_holding=&mut user_nfts_holding_data.collections;

        user_nfts_holding.retain_mut(|collection|{
            let  holding_nft=&mut collection.nfts_holding;
            
            holding_nft.retain(|nft|{
                if let Some(unrealized_gains) =&nft.unrealized_gains  {
                    let unrealized_gains=unrealized_gains.get(0..unrealized_gains.len()-4).unwrap().parse::<i64>().unwrap();
                    unrealized_gains>0
                }else {
                    nft.unrealized_gains .is_some()
                }
            });
            
            holding_nft.len()!=0
        });



        if user_nfts_holding.len()>0{
            // println!("{:?}",user_nfts_holding_data);
            Some(
                UserNftHolding{
                    collections:user_nfts_holding_data.collections,
                })
        }else {
            
            None
        }
        
    }else {
        None
    }
}

// // 在 holding nfts 的基础上，提取所有nft 并根据 unrealized_gains排序
pub async fn get_user_top_nfts(wallet_address:&str,conn:&mut PgConnection) -> Option<UserNftTop> {
    
    if let Some(user_nft_collections) =get_user_nfts_holidng(wallet_address, conn).await  {
        
        let mut nfts:Vec<UserNft>=vec![];

        user_nft_collections.collections.iter().for_each(|collection|{
            collection.nfts_holding.iter().for_each(|nft|{
                nfts.push(nft.clone());
            })
        });

        let mut top_gainers_nfts:Vec<UserTopNft>=vec![];
        let mut  top_losser_nfts:Vec<UserTopNft>=vec![];

        nfts.iter().for_each(|nft|{
            
            let buy_pirce=&nft.buy_price;
            let floor_price=&nft.floor_price;
            let unrealized_gains=&nft.unrealized_gains;

           

            if unrealized_gains.is_some(){
                let unrealized_gains=unrealized_gains.clone().unwrap().get(0..unrealized_gains.clone().unwrap().len()-4).unwrap().parse::<f64>().unwrap();
              
                if unrealized_gains>0.0{
                    
                    if buy_pirce.is_some() {
                        let buy_price=buy_pirce.clone().unwrap().get(0..buy_pirce.clone().unwrap().len()-4).unwrap().parse::<f64>().unwrap();
                        
                        let price_fluctuation=unrealized_gains/buy_price/100.0; 
                        top_gainers_nfts.push(
                            UserTopNft { 
                                name: nft.name.clone(), 
                                token_id: nft.token_id.clone(), 
                                key: nft.key.clone(), 
                                image: nft.image.clone(), 
                                price: nft.floor_price.clone(), 
                                price_fluctuation: Some(price_fluctuation),
                                unrealized_gains: nft.unrealized_gains.clone(), 
                            })
                    }else {
                        top_gainers_nfts.push(
                            UserTopNft { 
                                name: nft.name.clone(), 
                                token_id: nft.token_id.clone(), 
                                key: nft.key.clone(), 
                                image: nft.image.clone(), 
                                price: nft.floor_price.clone(), 
                                price_fluctuation: None,
                                unrealized_gains: nft.unrealized_gains.clone(), 
                            })
                    }
                    
                }else if unrealized_gains<0.0 {
                    if buy_pirce.is_some() {
                        let buy_price=buy_pirce.clone().unwrap().get(0..buy_pirce.clone().unwrap().len()-4).unwrap().parse::<f64>().unwrap();
                        
                        let price_fluctuation=unrealized_gains/buy_price/100.0; 
                        top_losser_nfts.push(
                            UserTopNft { 
                                name: nft.name.clone(), 
                                token_id: nft.token_id.clone(), 
                                key: nft.key.clone(), 
                                image: nft.image.clone(), 
                                price: nft.floor_price.clone(), 
                                price_fluctuation: Some(price_fluctuation),
                                unrealized_gains: nft.unrealized_gains.clone(), 
                            })
                    }else {
                        top_losser_nfts.push(
                            UserTopNft { 
                                name: nft.name.clone(), 
                                token_id: nft.token_id.clone(), 
                                key: nft.key.clone(), 
                                image: nft.image.clone(), 
                                price: nft.floor_price.clone(), 
                                price_fluctuation: None,
                                unrealized_gains: nft.unrealized_gains.clone(), 
                            })
                    }
                }
            }

        });

        //降序 排 top_gainers_nfts
        top_gainers_nfts.sort_by_key(|nft|{
            let unrealized_gains=nft.unrealized_gains.clone().unwrap();
            let unrealized_gains_price=unrealized_gains.get(0..unrealized_gains.len()-4).unwrap().parse::<f64>().unwrap();

        });

        //升序 排  top_losser_nfts
        top_losser_nfts.sort_by_key(|nft|{
            let unrealized_gains=nft.unrealized_gains.clone().unwrap();
            let unrealized_gains_price=unrealized_gains.get(0..unrealized_gains.len()-4).unwrap().parse::<f64>().unwrap();

        });
        top_losser_nfts.reverse();

        Some(UserNftTop { top_gainers:top_gainers_nfts, top_losser: top_losser_nfts })
    }else {
        None
    }
}


pub async fn get_user_trade_info_nfts(wallet_address:&str,conn:&mut PgConnection) ->Option<UserTradeInfo> {

        let mut age_of_nft_assets:MaybeUninit<Option<AgeOfNftAssets>>=MaybeUninit::uninit();
        let age_of_nft_assets_ptr:*mut Option<AgeOfNftAssets>=age_of_nft_assets.as_mut_ptr();

        if let Some(user_holding_nfts) =get_user_nfts_holidng(wallet_address, conn).await  {
            
            let day_now=Utc::now().date_naive();

            let mut holding_1_week_nfts:Vec<UserNft>=vec![];   // =<7
            let mut holding_1_to_4_weeks_nfts:Vec<UserNft>=vec![];    //  7 <x =< 28
            let mut holding_1_to_3_months_nfts:Vec<UserNft>=vec![];   // 28< x =< 90
            let mut holding_3_to_6_months_nfts:Vec<UserNft>=vec![];    // 90 < x =< 189
            let mut holding_6_to_12_months_nfts:Vec<UserNft>=vec![];    // 180 < x =< 360
            let mut holding_more_than_1_years_nfts:Vec<UserNft>=vec![];    // 360 <x

            let user_collection_assets=user_holding_nfts.collections;
            user_collection_assets.iter().for_each(|collection|{
                collection.nfts_holding.iter().for_each(|nft|{
                    
                    let ts=DateTime::parse_from_rfc3339(&nft.ts).unwrap().with_timezone(&Utc).date_naive();
                    let duration=day_now.signed_duration_since(ts);
                    
                    if duration>Duration::days(360){
                        holding_more_than_1_years_nfts.push(nft.clone())
                    }else if duration>=Duration::days(189) {
                        holding_6_to_12_months_nfts.push(nft.clone())
                    }else if duration >=Duration::days(90) {
                        holding_3_to_6_months_nfts.push(nft.clone())
                    }else if duration >= Duration::days(28) {
                        holding_1_to_3_months_nfts.push(nft.clone())
                    }else if duration >=Duration::days(7) {
                        holding_1_to_4_weeks_nfts.push(nft.clone())
                    }else {
                        holding_1_week_nfts.push(nft.clone())
                    }
                })
            });
            unsafe {
                age_of_nft_assets_ptr.write(Some(
                    AgeOfNftAssets { 
                        level1: holding_1_week_nfts, 
                        level2: holding_1_to_4_weeks_nfts, 
                        level3: holding_1_to_3_months_nfts, 
                        level4: holding_3_to_6_months_nfts, 
                        level5: holding_6_to_12_months_nfts, 
                        level6: holding_more_than_1_years_nfts, 
                    }
                ));   
                    
            }

        }else {
            unsafe {
                age_of_nft_assets_ptr.write(None);
            }
        }
        
        if let Some(user_info) =search_user(wallet_address, conn).await  {
            
        
            
            let mut all_buy_trades:Vec<_trade>=vec![];
            let mut all_sell_trades:Vec<_trade>=vec![];
            let mut all_trades:Vec<_trade>=vec![];   // all  transaction

            let mut trades_day:Vec<_trade>=vec![];
            let mut trades_week:Vec<_trade>=vec![];
            let mut trades_month:Vec<_trade>=vec![];

            let nft_transactions=&user_info.nfts_transactions;

            nft_transactions.iter().for_each(|transaction|{
                match transaction.to_owned().transaction {
                    _NftTransaction::AcceptBid(data)=>{
                        
                        let ts=data.transfer.ts;
                        let sale_price=data.sale_price;

                        let trade=_trade{
                            sale_price,
                            ts,
                        };
                        all_trades.push(trade.clone());
                        if &data.transfer.recipient ==wallet_address{
                            all_buy_trades.push(trade.clone());
                        }else if &data.transfer.sender == wallet_address {
                            all_sell_trades.push(trade.clone());
                        };


                    },
                    _NftTransaction::BatchBids(data)=>{

                        let ts=data.transfer.ts;
                        let sale_price=data.sale_price;
                        let trade=_trade{
                            sale_price,
                            ts,
                        };
                        all_trades.push(trade.clone());
                        if &data.transfer.recipient ==wallet_address{
                            all_buy_trades.push(trade.clone());
                        }else if &data.transfer.sender == wallet_address {
                            all_sell_trades.push(trade.clone());
                        };

                    },
                    _NftTransaction::CancelAuction(data)=>{

                        let ts=data.transfer.ts;
                        let sale_price=data.auction_price;
                        let trade=_trade{
                            sale_price,
                            ts,
                        };

                        all_trades.push(trade.clone());
                    },
                    _NftTransaction::CretaeAuction(data)=>{
                        let ts=data.transfer.ts;
                        let sale_price=data.auction_price;
                        let trade=_trade{
                            sale_price,
                            ts,
                        };

                        all_trades.push(trade.clone());
                        if &data.transfer.recipient == wallet_address{
                            all_buy_trades.push(trade.clone());
                        }else if &data.transfer.sender == wallet_address {
                            all_sell_trades.push(trade.clone());
                        }
                    },
                    _NftTransaction::FixedSell(data)=>{
                        
                        let ts=data.transfer.ts;
                        let sale_price=data.price;
                        let trade=_trade{
                            sale_price,
                            ts,
                        };

                        all_trades.push(trade.clone());
                        if &data.transfer.recipient == wallet_address{
                            all_buy_trades.push(trade.clone());
                        }else if &data.transfer.sender == wallet_address {
                            all_sell_trades.push(trade.clone());
                        }
                    },
                    _NftTransaction::Mint(data)=>{
                        let ts=data.ts;
                        let sale_price=format!("{}usei",data.price);
                        let trade=_trade{
                            sale_price,
                            ts,
                        };

                        all_trades.push(trade.clone());
                    },
                    _NftTransaction::OnlyTransfer(data)=>{
                        let ts=data.ts;
                        let sale_price="0usei".to_string();
                        let trade=_trade{
                            sale_price,
                            ts,
                        };
                        all_trades.push(trade.clone());

                    },
                    _NftTransaction::PurchaseCart(data)=>{
                        let ts=data.transfer.ts;
                        let sale_price=data.sale_price;
                        let trade=_trade{
                            sale_price,
                            ts,
                        };
                        all_trades.push(trade.clone());
                        if &data.transfer.recipient == wallet_address{
                            all_buy_trades.push(trade.clone());
                        }else if &data.transfer.sender == wallet_address {
                            all_sell_trades.push(trade.clone());
                        }
                        
                    },
                    _=>{},
                }
            });

            let mut transactions=Transactions::new();
            Transactions::add_data(&mut transactions, all_trades);

            let mut buy_volume=Transactions::new();
            Transactions::add_data(&mut buy_volume, all_buy_trades);
           
            let mut sell_volume=Transactions::new();
            Transactions::add_data(&mut sell_volume, all_sell_trades);

            unsafe {
                let  age_of_nft_assets=(*age_of_nft_assets_ptr).clone();
                drop(age_of_nft_assets_ptr);
                Some(
                    UserTradeInfo{
                        age_of_nft_assets:age_of_nft_assets,
                        transaction:transactions,
                        volume:Volume { 
                            buy_volume: buy_volume, 
                            sell_volume: sell_volume }
                    }
                )
                
            }
            
            
        }else {
            None
        }
            
            
           
            // println!("{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}\n",holding_1_week_nfts,holding_1_to_4_weeks_nfts,holding_1_to_3_months_nfts,holding_3_to_6_months_nfts,holding_6_to_12_months_nfts,holding_more_than_1_years_nfts);
       
        
}

async fn get_collection_nfts_floorprice(hold_collection_keys:&HashMap<String,Vec<String>>) ->HashMap<String,HashMap<String,Option<String>>>{
    
    let mut collection_nfts_floorprice:HashMap<String,HashMap<String,Option<String>>>=HashMap::new();
    
    let collection_nfts_floorprice=Arc::new(Mutex::new(collection_nfts_floorprice));


    let semaphore=Arc::new(Semaphore::new(16));
    
    let mut handles:Vec<tokio::task::JoinHandle<()>>=vec![];
    
    let conn=Arc::new(Mutex::new(client_db().await.unwrap()));
    
    
    for collection in hold_collection_keys.clone(){
        
        let semaphore=Arc::clone(&semaphore);
        let collection_nfts_floorprice=Arc::clone(&collection_nfts_floorprice);

        let conn=Arc::clone(&conn);


        let handle: tokio::task::JoinHandle<()>=tokio::spawn(async move {

            let mut floor_price:HashMap<String,Option<String>>=HashMap::new();
            let permit=semaphore.acquire().await.unwrap();
            let  collection_nfts_floorprice=&mut collection_nfts_floorprice.lock().await;
            let mut conn=conn.lock().await;
            let mut floor_price:HashMap<String,Option<String>>=HashMap::new();


            if let Some(data) =search_contract_createauctions(&collection.0,&mut conn).await  {
                
                let  auctions=data.create_auctions;
                
                if auctions.len()>0{
                    for token_id in collection.1{
                        auctions.iter().for_each(|auction|{
                            
                            
                            if auction.token_id == token_id{
                                let new_price=auction.auction_price.clone();
                                let _new_price=new_price.get(0..new_price.len()-4).unwrap().parse::<u64>().unwrap();

                                if let Some(price_data) =floor_price.get(&token_id).cloned()  {
                                    if price_data.is_some(){
                                        let old_price=price_data.unwrap();
                                        let old_price=old_price.get(0..old_price.len()-4).unwrap().parse::<u64>().unwrap();
                                        if _new_price<old_price{
                                            floor_price.remove(&token_id);
                                            floor_price.insert(token_id.clone(), Some(new_price));
                                        }
                                    }else {
                                        floor_price.insert(token_id.clone(), Some(new_price));
                                    }
                                }else {
                                    floor_price.insert(token_id.clone(), Some(new_price));
                                }
                            }

                            let new_price=auction.auction_price.clone();
                            let _new_price=new_price.get(0..new_price.len()-4).unwrap().parse::<u64>().unwrap();
                            if let Some(price_data) =floor_price.get(&collection.0).cloned() {
                                if price_data.is_some(){
                                    let old_price=price_data.unwrap();
                                    let old_price=old_price.get(0..old_price.len()-4).unwrap().parse::<u64>().unwrap();
                                    if _new_price<old_price{
                                        floor_price.remove(&collection.0);
                                        floor_price.insert(collection.0.clone(), Some(new_price));
                                    }
                                }else {
                                    floor_price.insert(collection.0.clone(), Some(new_price));
                                }
                            }else {
                                floor_price.insert(collection.0.clone(), Some(new_price));
                            }
                        });

                    };
                    collection_nfts_floorprice.insert(collection.0.clone(),floor_price.clone());
                }else {
                    floor_price.insert(collection.0.clone(), None);
                    collection_nfts_floorprice.insert(collection.0.clone(),floor_price.clone());
                }
            }else {
                floor_price.insert(collection.0.clone(), None);
                collection_nfts_floorprice.insert(collection.0.clone(),floor_price.clone());
            }
            
            
             
            drop(permit);    
        });
        
        handles.push(handle);
    };

    for hanlde in handles{
        hanlde.await.unwrap();
    }
    return collection_nfts_floorprice.lock().await.to_owned() ;
}



mod db_tests{
    use super::*;


    #[tokio::test]
    async fn test_db()  {
        let mut conn=client_db().await.unwrap();
        // // let conn=Arc::new(Mutex::new(conn));
        let a=get_user_trade_info_nfts("sei10l9hc655uyzwv5xq5ww3l6h93tccwyuljnrk03", &mut conn).await;
        println!("{:?}",a)
        // let b=search_contract_createauctions("sei13l8rdgguhhmfpe9mqfp0q6ywnw068gmf46tgadvjvdjwnd89ymyq85nnw8", &mut conn).await;
        // println!("{:?}",b);

     
        

    }
}