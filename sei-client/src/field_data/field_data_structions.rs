use serde::{Serialize, Deserialize};
use serde_json::Value;

use std::any::Any;
use std::fmt::Debug;


// #[derive(Serialize, Deserialize,Clone,Debug)]
// pub struct HoldingNft{
//     pub account:String,
// }

// // 有transfer才要改写数据库的holding
// trait NftTransaction :Debug+Any {   
// }



// #[derive(Serialize, Deserialize,Clone,Debug)]
// pub struct NftSale{

//     pub trade_type:String,
//     pub ts:String,
//     pub tx:String,
// }
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct CollectionInfo{
    pub name:String,
    pub symbol:String,
    pub creator:String,
    pub count :String,
}




#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TokenSwap{
    pub account:String,
    pub source_token:String,
    pub target_token:String,
    pub source_amount:String,
    pub target_amount:String,
    pub trade_type:String,    // swap_token
    pub ts:String,
    pub tx:String,
}



#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NftToken{
    pub token_id:String,
    pub name:String, // CollectionInfo name + # + id 
    pub key:String, // collection + - +id
    pub image:String,
    pub royalty_percentage:u64,
    pub attributes:Vec<NftAttribute>,
}
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NftAttribute{
    pub trait_type:String,
    pub value:String,
}



#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct Collection{
    pub collection :String,
    pub name:String,
    pub symbol:String,
    pub creator:String,
    pub count:String,
    pub nfts:Vec<NftToken>,
}


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct User{
    pub wallet_address:String,
    pub nfts_holding:Vec<Collection>,   // Vec<Collection>
    pub nfts_transactions:Vec<NFTtransaction>, // Vec<NFTtransaction>
    pub token_transactions:Vec<TokenSwap>,  // Vev<TokenSwap>

}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct NFTtransaction{
    pub transaction:_NftTransaction,
    pub _type:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub enum _NftTransaction {
    Mint(MintNFt),
    BatchBids(BatchBidsNft),
    OnlyTransfer(TransferNft),
    CancelAuction(CancelAuctionNft),
    CretaeAuction(CretaeAuctionNft),
    PurchaseCart(PurcjaseCartNft),
    AcceptBid(AcceptBidNft),
    FixedSell(FixedSellNft),
    Unkonw,
}
// nft transactions
#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct TransferNft{
    pub collection:String,
    pub sender:String,
    pub recipient:String,
    pub token_id:String,
    pub ts:String,
    pub tx:String,
    // pub trade_type:String,

}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct MintNFt{
    pub collection:String,
    pub recipient:String,
    pub token_id:String,
    pub price:String,
    pub ts:String,
    pub tx:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct BatchBidsNft{
    pub transfer:TransferNft,
    pub sale_price:String,
}


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct FixedSellNft{
    pub transfer:TransferNft,
    pub price:String,
}


#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct CretaeAuctionNft{
    pub transfer:TransferNft,
    pub auction_price:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct CancelAuctionNft{
    pub transfer:TransferNft,
    pub auction_price:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct PurcjaseCartNft{
    pub transfer:TransferNft,
    pub buyer:String,
    pub seller:String,
    pub sale_price:String,
    pub marketplace_fee:String,
    pub royalties:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct AcceptBidNft{
    pub transfer:TransferNft,
    pub bidder:String,
    pub seller : String,
    pub sale_price:String,
    pub marketplace_fee:String,
    pub royalties:String,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct ContractCreateAuctions{
    pub contract_address:String,
    pub create_auctions:Vec<OnlyCreateAuction>,
}

#[derive(Serialize, Deserialize,Clone,Debug)]
pub struct OnlyCreateAuction{
    pub collection_address:String,
    pub token_id:String,
    pub auction_price:String,
    pub ts:String,
}
