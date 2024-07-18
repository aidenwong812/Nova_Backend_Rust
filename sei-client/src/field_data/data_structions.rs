use serde::{Serialize, Deserialize};
use serde_json::Value;
use super::field_data_structions::NftAttribute;


// #[derive(Serialize, Deserialize,Clone,Debug)]
// pub struct CollectionInfo{
//     pub sei_address:String,
//     pub evm_address:String,
//     pub creater:String,
//     pub name:String,
//     pub symbol:String,
//     pub pfp:String,  // https://static-assets.pallet.exchange/pfp/{name}.jpg  小写
//     pub items:String,
// }






// // user  nft transaction history
// #[derive(Serialize, Deserialize,Clone,Debug)]
// pub struct  NftHoldTransaction{
//     pub collection:String,
//     pub id:String,
//     pub key:String, // collection + id

//     pub sender:String,
//     pub recipient:String,
//     pub price:Option<String>,
//     pub marketplace_fee:Option<String>,
//     pub royalties:Option<String>,
//     pub gas:Option<String>,

//     pub buy_time:String,
//     pub tx:String,
// }

// // user tokens transcations history
// #[derive(Serialize, Deserialize,Clone,Debug)]
// pub struct TokensTranscations{
//     pub source_denom_account:String,
//     pub target_denom_account:String,
//     pub source_amounts:String,
//     pub target_amount:String,
//     pub gas:String,
//     pub ts:String,
//     pub tx:String
// }


// data from block chain 
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct HashData{
    pub tx:Tx,
    pub tx_response:TxResponse
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Tx{
    pub auth_info:TxAuthInfo,
    pub body:TxBody,
        signatures:Vec<String>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxResponse {
    pub code:u64,
        codespace:Value,
        data:Value,
    pub events:Value,
    pub gas_used:String,
    pub gas_wanted:String,
        height:String,
        info:Value,
    pub logs:Vec<Log>,
        raw_log:Value,
    pub timestamp:String,
        tx:Value,
    pub txhash:String,

}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Log{
    pub events:Vec<Event>,
        log:Value,
        msg_index:Value,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Event{
    
    pub attributes:Vec<Attribute>,
    #[serde(rename = "type")]
    pub _type:String,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Attribute{
    pub key:String,
    pub value:String,
}



// Tx 组件
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxAuthInfo{
    pub fee:Fee,
        signer_infos:Value,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Fee{
    pub amount:Vec<FeeAmount>,
    pub gas_limit:String,
        granter:String,
        payer:String,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct FeeAmount{
    pub amount:String,
    pub denom:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TxBody{
    pub extension_options:Value,
    pub memo:Value,
    pub messages:Value,
    pub non_critical_extension_options:Value,
    pub timeout_height:Value,
}





// searcher nft info cosmos
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct AllNftInfo{
    pub access:Access,
    pub info:Nftinfo,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Access{
        approvals:Value,
    pub owner:String
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Nftinfo{
    pub extension:Extension,
    pub token_uri:String,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Extension{
        animation_url:Value,
        attributes:Value,
        background_color:Value,
        description:Value,
        external_url:Value,
        image:Value,
        image_data:Value,
        name:Value,
        royalty_payment_address:Value,
    pub royalty_percentage:u64,
        youtube_url :Value
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Attributes{
    // pub name:String,
    // pub symbol:String,
    // // pub collection:String,
    // pub description:String,
    // pub image:String,
    // pub edition:u64,
    pub attributes:Vec<NftAttribute>
}
