use serde::{Serialize, Deserialize};
use field_blockchain_data::data_structions::{NftHoldTransaction,TokensTranscations,CollectionInfo,NftToken};

//定义 sql table

// wallet 的主表
#[derive(Serialize, Deserialize,Clone)]
pub struct WalletNftInfo{
    pub wallet_address:String,
    pub nft_holding:Vec<HoldingNft>,
    pub nft_history_transcations:Option<Vec<NftHoldTransaction>>,
    pub token_history_transactions:Option<Vec<TokensTranscations>>,
}

// holding nft 
#[derive(Serialize, Deserialize,Clone)]
pub struct HoldingNft{
    pub collection_info:CollectionInfo,
    pub holding_tokens:Vec<NftToken>,
}

// CollectionInfo 的表

#[derive(Serialize, Deserialize,Clone)]
pub struct  CollectionInfoDb{
    pub collection_info:CollectionInfo,
    pub tokens:Vec<NftToken>
}