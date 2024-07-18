use chrono::{DateTime, Datelike, IsoWeek, NaiveDate, NaiveDateTime, Utc};
use sei_client::field_data::field_data_structions::NftAttribute;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;


use std::any::Any;
use std::fmt::Debug;

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct  UserNftHolding{
    pub collections:Vec<UserCollectionHold>
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct  UserCollectionHold{
    pub name:String,
    pub symbol:String,
    pub contract:String,
    pub creator:String,
    pub floor_price:Option<String>,
    pub nfts_holding:Vec<UserNft>
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct  UserNft{
    pub name:String,
    pub token_id:String,
    pub key:String,
    pub image:String,
    pub floor_price:Option<String>,
    pub attributes:Vec<NftAttribute>,
    
    pub buy_price:Option<String>,
    pub royalties_fee:Option<String>,
    pub market_fee:Option<String>,
    pub unrealized_gains:Option<String>,
    pub ts:String,
    pub tx_hash:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct  UserTopNft{
    pub name:String,
    pub token_id:String,
    pub key:String,
    pub image:String,
    
    pub price:Option<String>,  // 当前 的floor price
    pub price_fluctuation:Option<f64> ,  // unrealized_gains / buy price
    pub unrealized_gains:Option<String>,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct  UserNftTop{
    pub top_gainers:Vec<UserTopNft>,
    pub top_losser:Vec<UserTopNft>
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct UserTradeInfo{
    pub age_of_nft_assets:Option<AgeOfNftAssets>,
    pub transaction:Transactions,
    pub volume:Volume,

}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct AgeOfNftAssets{
    pub level1: Vec<UserNft>,
    pub level2:Vec<UserNft>,
    pub level3:Vec<UserNft>,
    pub level4:Vec<UserNft>,
    pub level5:Vec<UserNft>,
    pub level6:Vec<UserNft>,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Volume{
    pub buy_volume:Transactions,
    pub sell_volume:Transactions,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Transactions{
    pub day:HashMap<String,_Transaction>,
    pub week:HashMap<String,_Transaction>,
    pub month:HashMap<String,_Transaction>,  //(yaer month)
}
impl Transactions {
    pub fn new()->Self{
        Transactions{
            day:HashMap::new(),
            week:HashMap::new(),
            month:HashMap::new(),
        }
    }

    pub fn add_data(&mut self ,trades:Vec<_trade>){

        // 天分组 
        let mut day_trades:HashMap<NaiveDate,Vec<_trade>>=HashMap::new();
        //周分组
        let mut week_trades:HashMap<(i32,u32),Vec<_trade>>=HashMap::new();
        //月分组
        let mut month_trades:HashMap<(i32,u32),Vec<_trade>>=HashMap::new();


        trades.iter().for_each(|trade|{
            let date: NaiveDateTime=DateTime::parse_from_rfc3339(&trade.ts).unwrap().with_timezone(&Utc).naive_utc();
            
            day_trades.entry(date.clone().date()).or_insert_with(Vec::new).push(trade.clone());

            let iso_week=date.iso_week();
            week_trades.entry((iso_week.year(),iso_week.week())).or_insert_with(Vec::new).push(trade.clone());

            month_trades.entry((date.year(),date.month())).or_insert_with(Vec::new).push(trade.clone())
        });
        
        day_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.day.entry(key.clone().to_string()).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.day.entry(key.clone().to_string()).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });

        week_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.week.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.week.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });

        month_trades.into_iter().for_each(|(key,value)|{
            let transaction_amount=value.len();
            let mut sale_price:Vec<usize>=vec![];
            value.iter().for_each(|trade|{
                let volume = trade.sale_price.clone().get(0..trade.sale_price.clone().len()-4).unwrap().parse::<usize>().unwrap();
                sale_price.push(volume);
            });

            if sale_price.len()>0{
                let volume:usize=sale_price.iter().sum();
                let total_volume=format!("{}usei",volume.to_string());
                self.month.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }else {
                let total_volume="0usei".to_string();
                self.month.entry(format!("{}-{}",key.clone().0.to_string(),key.clone().1.to_string())).or_insert_with(||_Transaction{transaction_amount:transaction_amount.clone(),total_volume:total_volume});
            }
            
        });


    }
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct _Transaction{
    pub transaction_amount:usize,
    pub total_volume:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct _trade{
    pub sale_price:String,
    pub ts:String,
}