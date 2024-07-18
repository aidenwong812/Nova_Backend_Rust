use serde::{Serialize, Deserialize};
use serde_json::Value;
use sqlx::FromRow;








#[derive(Serialize, Deserialize,Clone,Debug,FromRow,PartialEq,Eq)]
pub struct _Collection{
    pub collection :String,
    pub name:String,
    pub symbol:String,
    pub creator:String,
    pub count:String,
    pub nfts:Value,
}




#[derive(Serialize, Deserialize,Clone,Debug,FromRow,PartialEq,Eq)]
pub struct _User{
    pub wallet_address:String,
    pub nfts_holding:Value,   // Vec<Collection>
    pub nfts_transactions:Value, // Vec<NFTtransaction>
    pub token_transactions:Value,  // Vev<TokenSwap>

}

#[derive(Serialize, Deserialize,Clone,Debug,FromRow,PartialEq,Eq)]
pub struct _User_nft_holding{
    pub nfts_holding:Vec<Value>,
}

#[derive(Serialize, Deserialize,Clone,Debug,FromRow,PartialEq,Eq)]
pub struct _ContractCreateAuctions{
    pub contract_address:String,
    pub create_auction_transactions:Value,
}