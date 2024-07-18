use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Wallet{
    pub wallet_address:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct DontHaveData{
    pub wallet_address:String,
    pub result:Option<String>
}