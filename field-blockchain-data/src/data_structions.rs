use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize,Clone)]
pub struct CollectionInfo{
    pub sei_address:String,
    pub evm_address:String,
    pub creater:String,
    pub name:String,
    pub symbol:String,
    pub pfp:String,  // https://static-assets.pallet.exchange/pfp/{name}.jpg  小写
    pub items:String,
}



#[derive(Serialize, Deserialize,Clone)]
pub struct NftToken{
    pub id:String,
    pub name:String, // CollectionInfo name + # + id 
    pub key:String, // collection + - +id
    pub image:String,
    pub attributes:Vec<Attributes>,
    pub buy_price:Option<String>,
    pub marketplace_fee:Option<String>,
    pub royalties:Option<String>,
    pub gas:Option<String>,
    pub buy_time:String,
    pub tx:String,
    
}
#[derive(Serialize, Deserialize,Clone)]
pub struct Attributes{
    pub trait_type:String,
    pub value:String,
}



// user  nft transaction history
#[derive(Serialize, Deserialize,Clone)]
pub struct  NftHoldTransaction{
    pub collection:String,
    pub id:String,
    pub key:String, // collection + id

    pub sender:String,
    pub recipient:String,
    pub price:Option<String>,
    pub marketplace_fee:Option<String>,
    pub royalties:Option<String>,
    pub gas:Option<String>,

    pub buy_time:String,
    pub tx:String,
}

// user tokens transcations history
#[derive(Serialize, Deserialize,Clone)]
pub struct TokensTranscations{
    pub source_denom_account:String,
    pub target_denom_account:String,
    pub source_amounts:String,
    pub target_amount:String,
    pub gas:String,
    pub ts:String,
    pub tx:String
}