use std::error::Error;
use std::sync::mpsc::{Sender,Receiver};
use std::sync::Arc;
use serde_json::Value;
use super::{data_structions::{Attribute, Event, HashData, Log, TxResponse}};
use super::field_data_structions::{AcceptBidNft, BatchBidsNft, CancelAuctionNft, CretaeAuctionNft, FixedSellNft, MintNFt, PurcjaseCartNft, TransferNft,OnlyCreateAuction};

// 定义 nft transaction 解析后 函数返回数据
#[derive(Debug)]
pub enum NftMessage {
    Mint(Vec<MintNFt>),
    BatchBids(Vec<BatchBidsNft>),
    OnlyTransferNft(Vec<TransferNft>),
    CretaeAuctionNft(Vec<CretaeAuctionNft>),
    CancelAuctionNft(Vec<CancelAuctionNft>),
    PurchaseCartNft(Vec<PurcjaseCartNft>),
    AcceptBidNft(Vec<AcceptBidNft>),
    FixedSellNft(Vec<FixedSellNft>),
    OnlyCreateAuction(Vec<OnlyCreateAuction>),
    Unkonw(String),
}


pub async fn nft_transaction_data(hash_data:HashData,nft_msg_tx:Arc<Sender<NftMessage>>)  {
    
    // 定义过滤闭包
    // mint nft
    let is_mint_nft=|events:&Vec<Event>| ->bool{
        events.iter().any(|event|{
            event._type=="wasm" && event.attributes.iter().any(|attribute|{attribute.value=="mint_nft"})
        })
    };

    // batch_bids
    let is_batch_bids=|events:&Vec<Event>| ->bool{
        if events.iter().any(|event|{
            event._type=="wasm" && event.attributes.iter().any(|attribute|{attribute.value=="batch_bids"}) && event.attributes.iter().any(|attribute|{attribute.value=="transfer_nft"}) 
        }) && events.iter().any(|event|{event._type=="wasm-buy_now"}){
            return  true;
        }else {
            return false;
        }
     };


    let is_fixed_sell=|events:&Vec<Event>| ->bool{
      if events.iter().any(|event|{
        event._type=="wasm" &&event.attributes.iter().any(|attr|{attr.value=="fixed_sell"})
      }){
        return true;
      }else {
          return false;
      }
    };
    

     //
    let is_only_transfer_nft=|events:&Vec<Event>|->bool{
        if events.iter().any(|event|{
            event._type=="wasm" &&event.attributes.iter().any(|attribute|{attribute.value=="transfer_nft"})&&event.attributes[1].value=="transfer_nft"
            }) && events.last().unwrap()._type=="wasm"{
                return  true;
            }else {
                return  false;
            }
    };

    let is_create_auction_nft=|events:&Vec<Event>| -> bool{
      if events.iter().any(|event|{event._type=="wasm"}) && events.iter().any(|event|{event._type=="wasm-create_auction"}){
        return true;
      }else {
          return false;
      }
    };
    
    let is_cancel_auction_nft=|events:&Vec<Event>| -> bool{
        if events.iter().any(|event|{event._type=="wasm"}) && events.iter().any(|event|{event._type=="wasm-cancel_auction"}){
            return true;
          }else {
              return false;
          }
      };
    
    let is_purchase_nft=|events:&Vec<Event>| ->bool{
        if events.iter().any(|event|{
            event._type=="wasm" && event.attributes.iter().any(|attribute|{attribute.value=="purchase_cart"}) && event.attributes.iter().any(|attribute|{attribute.value=="transfer_nft"}) 
        }) && events.iter().any(|event|{event._type=="wasm-buy_now"}){
            return  true;
        }else {
            return false;
        }
    };

    // only create auction
    let is_only_create_aucton_nft=|events:&Vec<Event>|->bool{
        if events.iter().any(|event|{event._type=="wasm-create_auction"}){
            return true;
        }else {
            return false;
        }
    };


    let is_accept_bid_nft=|events:&Vec<Event>| -> bool{
        if events.iter().any(|event|{event._type=="wasm"}) && events.iter().any(|event|{event._type=="wasm-accept_bid"}){
            return true;
          }else {
              return false;
          }
    };

    let logs=hash_data.tx_response.logs.clone();
    let ts=hash_data.tx_response.timestamp.clone();
    let tx=hash_data.tx_response.txhash.clone();
    

    for log in logs{

        let nft_msg_tx=nft_msg_tx.clone();

        if is_only_create_aucton_nft(&log.events){

            let wasm_create_auction=log.events.iter().find(|event|{event._type=="wasm-create_auction"}).unwrap();
            let data=only_create_aucton_nft(wasm_create_auction.attributes.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::OnlyCreateAuction(data.unwrap()));
            }
        };

        if is_mint_nft(&log.events){
        
            let wasm=log.events.iter().find(|event|{event._type=="wasm"}).unwrap();
           
            let data=mint_nft_datas(wasm.attributes.clone(), tx.clone(), ts.clone()).await;
            
            if data.is_some(){
                nft_msg_tx.send(NftMessage::Mint(data.unwrap()));
            }
           
        
        }else if is_batch_bids(&log.events) {
        
            let data=batch_bids_nfts(log.events.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::BatchBids(data.unwrap()));
            }
        
        }else if is_only_transfer_nft(&log.events) {
        
            let wasm=log.events.iter().find(|event|{event._type=="wasm"}).unwrap();
            let data=only_transfer_nft(wasm.attributes.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::OnlyTransferNft(data.unwrap()));
            }
        
        }else if is_create_auction_nft(&log.events) {
            
            let data=create_auction_nft(log.events.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::CretaeAuctionNft(data.unwrap()));
            }
        }else if  is_cancel_auction_nft(&log.events)  {
            
            let data=cancel_auction_nft(log.events.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::CancelAuctionNft(data.unwrap()));
            }
        
        }else if is_purchase_nft(&log.events) {
            let data=purchase_cart_nft(log.events.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::PurchaseCartNft(data.unwrap()));
            }
        
        }else if is_accept_bid_nft(&log.events){
            let data=accept_bid_nft(log.events.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::AcceptBidNft(data.unwrap()));
            }
        }else if  is_fixed_sell(&log.events){
            let wasm=log.events.iter().find(|event|{event._type=="wasm"}).unwrap();
            let data=fixed_sell_nfts(wasm.attributes.clone(), tx.clone(), ts.clone()).await;
            if data.is_some(){
                nft_msg_tx.send(NftMessage::FixedSellNft(data.unwrap()));
            }   
        
        }else {
            // println!("{:?}",tx);
            nft_msg_tx.send(NftMessage::Unkonw(tx.clone()));
        }
    }
    

}


// 以下都是 wsam 中的
async fn mint_nft_datas(wasm_attrs:Vec<Attribute>,tx:String,ts:String) ->Option<Vec<MintNFt>>{
    
    let mut mint_nfts:Vec<MintNFt>=vec![];
    let mut mint_nft_indexs:Vec<usize>=vec![];

    // 定义 添加索引的闭包
    let mut add_index=|attrs:&Vec<Attribute>|{
        attrs.iter().enumerate().for_each(|( mut index,attr)|{
            if attr.value=="mint_nft"{
                mint_nft_indexs.push(index);
            }
        })
    };

    add_index(&wasm_attrs);

    for mint_nft_index in mint_nft_indexs{
        
        let collection=wasm_attrs[mint_nft_index+1].value.clone();
        let recipient=wasm_attrs[mint_nft_index+3].value.clone();
        let token_id=wasm_attrs[mint_nft_index+4].value.clone();
        let price=wasm_attrs[mint_nft_index+5].value.clone();
        
        mint_nfts.push(MintNFt { collection: collection, recipient: recipient, token_id: token_id, price: price ,ts:ts.clone(),tx:tx.clone()})

    }

    if mint_nfts.len()>0{
        Some(mint_nfts)
    }else {
        None
    }

    
}

async fn batch_bids_nfts(events:Vec<Event>,tx:String,ts:String) ->Option<Vec<BatchBidsNft>>{//-> Result<()> {
    
    let mut batch_bids_nft_datas:Vec<BatchBidsNft>=vec![]; 

    let mut nft_token_ids:Vec<String>=vec![];
    let mut nft_token_id_indexs:Vec<usize>=vec![];
    

    // 添加 token id ，并添加 transfer 的data
    let mut add_token_id=|events:&Vec<Event>|{
        events.iter().find(|event|{
            event._type=="wasm"
        }).and_then(|event|{
            Some(event.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
                if attribute.key=="token_id"{
                    nft_token_ids.push(attribute.value.clone());
                    nft_token_id_indexs.push(index);
                }
            }))
        })
    };

    let wasm=events.iter().find(|event|{event._type=="wasm"}).unwrap();
    let wasm_buy_now=events.iter().find(|event|{event._type=="wasm-buy_now"}).unwrap();

    if let Some(_) =add_token_id(&events)  {

        let mut index_counter:usize=0;

        for token_id in nft_token_ids{
            let wasm_nft_index=nft_token_id_indexs[index_counter];
            
            wasm_buy_now.attributes.iter().enumerate().for_each(|(mut wasm_buy_now_index,attribute)|{
                if attribute.key=="nft_token_id" && attribute.value == token_id{
                    batch_bids_nft_datas.push(
                        
                        BatchBidsNft{
                            transfer:TransferNft{
                                collection:wasm.attributes[wasm_nft_index-4].value.clone(),
                                sender:wasm.attributes[wasm_nft_index-2].value.clone(),
                                recipient:wasm.attributes[wasm_nft_index-1].value.clone(),
                                token_id:token_id.clone(),
                                ts:ts.clone(),
                                tx:tx.clone(),
                            },
                            sale_price:wasm_buy_now.attributes[wasm_buy_now_index-2].value.clone()
                        }
                    );
                    index_counter+=1;
                }
            });
            
        }

        if batch_bids_nft_datas.len()>0{
            Some(batch_bids_nft_datas)
        }else {
            None
        }
    
    }else {
        None
    }



}

async fn only_transfer_nft(wasm_attrs:Vec<Attribute>,tx:String,ts:String) ->Option<Vec<TransferNft>> {
    
    let mut transfer_nft_datas:Vec<TransferNft>=vec![];
    
    wasm_attrs.iter().enumerate().for_each(|(mut index,attribute)|{
        if attribute.key=="action" && attribute.value=="transfer_nft"{
            transfer_nft_datas.push(TransferNft{
                collection:wasm_attrs[index-1].value.clone(),
                sender:wasm_attrs[index+1].value.clone(),
                recipient:wasm_attrs[index+2].value.clone(),
                token_id:wasm_attrs[index+3].value.clone(),
                ts:ts.clone(),
                tx:tx.clone(),
            })
        }
    });

    if transfer_nft_datas.len()>0{
        Some(transfer_nft_datas)
    }else {
        None
    }
}

async fn fixed_sell_nfts(wasm_attrs:Vec<Attribute>,tx:String,ts:String) -> Option<Vec<FixedSellNft>> {
    let mut fixed_sell:Vec<FixedSellNft>=vec![];

    wasm_attrs.iter().enumerate().for_each(|(mut index,attr)|{
        if attr.key=="action" && attr.value=="fixed_sell"{
            fixed_sell.push(FixedSellNft { 
                transfer: TransferNft { 
                    collection: wasm_attrs[index+7].value.clone(), 
                    sender: wasm_attrs[index+9].value.clone(), 
                    recipient: wasm_attrs[index+10].value.clone(), 
                    token_id: wasm_attrs[index+3].value.clone(), 
                    ts: ts.clone(), 
                    tx: tx.clone() }, 
                price: wasm_attrs[index+4].value.clone() 
            })
        }
    });

    if fixed_sell.len()>0{
        Some(fixed_sell)
    }else {
        None
    }
}

async fn create_auction_nft(events:Vec<Event>,tx:String,ts:String) ->Option<Vec<CretaeAuctionNft>>{

    let mut create_auctions:Vec<CretaeAuctionNft>=vec![];    
    let mut token_ids:Vec<String>=vec![];
    let mut token_id_indexs:Vec<usize>=vec![];

    let wasm=events.iter().find(|event|{event._type=="wasm"}).unwrap();

    wasm.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
        if attribute.key=="token_id"{
            token_ids.push(attribute.value.clone());
            token_id_indexs.push(index);
        }
    });

    let wasm_create_auction=events.iter().find(|event|{event._type=="wasm-create_auction"}).unwrap();

    let mut index_counter:usize=0;
    for token_id in token_ids{
        
        let wasm_nft_id_index=token_id_indexs[index_counter];

        wasm_create_auction.attributes.iter().enumerate().for_each(|(mut wasm_carete_crate_auction_index,attribute)|{
            if attribute.key=="token_id" && attribute.value == token_id{
                create_auctions.push(
                    CretaeAuctionNft { 
                        transfer:TransferNft{
                            collection:wasm.attributes[wasm_nft_id_index-4].value.clone(),
                            sender:wasm.attributes[wasm_nft_id_index-2].value.clone(),
                            recipient:wasm.attributes[wasm_nft_id_index-1].value.clone(),
                            token_id:token_id.clone(),
                            ts:ts.clone(),
                            tx:tx.clone(),
                        },
                        auction_price: wasm_create_auction.attributes[wasm_carete_crate_auction_index+3].value.clone() }
                );
                index_counter+=1;
            }
        });
    }


    if create_auctions.len()>0{
        Some(create_auctions)
    }else {
        None
    }
    
    
}

async fn cancel_auction_nft(events:Vec<Event>,tx:String,ts:String) -> Option<Vec<CancelAuctionNft>> {
    
    let mut cancel_auctions:Vec<CancelAuctionNft>=vec![];    
    let mut token_ids:Vec<String>=vec![];
    let mut token_id_indexs:Vec<usize>=vec![];

    let wasm=events.iter().find(|event|{event._type=="wasm"}).unwrap();

    wasm.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
        if attribute.key=="token_id"{
            token_ids.push(attribute.value.clone());
            token_id_indexs.push(index);
        }
    });

    let wasm_cancel_auction=events.iter().find(|event|{event._type=="wasm-cancel_auction"}).unwrap();

    let mut index_counter:usize=0;
    for token_id in token_ids{
        
        let wasm_nft_id_index=token_id_indexs[index_counter];

        wasm_cancel_auction.attributes.iter().enumerate().for_each(|(mut wasm_cancel_crate_auction_index,attribute)|{
            if attribute.key=="token_id" && attribute.value == token_id{
                cancel_auctions.push(
                    CancelAuctionNft { 
                        transfer:TransferNft{
                            collection:wasm.attributes[wasm_nft_id_index-4].value.clone(),
                            sender:wasm.attributes[wasm_nft_id_index-2].value.clone(),
                            recipient:wasm.attributes[wasm_nft_id_index-1].value.clone(),
                            token_id:token_id.clone(),
                            ts:ts.clone(),
                            tx:tx.clone(),
                        },
                        auction_price: wasm_cancel_auction.attributes[wasm_cancel_crate_auction_index+2].value.clone() }
                );
                index_counter+=1;
            }
        });
    }

    if cancel_auctions.len() > 0{
        Some(cancel_auctions)
    }else {
        None
    }
}

async fn purchase_cart_nft(events:Vec<Event>,tx:String,ts:String) -> Option<Vec<PurcjaseCartNft>> {
    
    let mut purcjase_cart:Vec<PurcjaseCartNft>=vec![];
    let mut token_ids:Vec<String>=vec![];
    let mut token_id_indexs:Vec<usize>=vec![];

    let wasm=events.iter().find(|event|{event._type=="wasm"}).unwrap();
    let wasm_buy_now=events.iter().find(|event|{event._type=="wasm-buy_now"}).unwrap();

    wasm.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
        if attribute.key=="token_id"{
            token_ids.push(attribute.value.clone());
            token_id_indexs.push(index);
        }
    });

    let mut index_counter:usize=0;
    for token_id in token_ids{
        
        let wasm_nft_id_index=token_id_indexs[index_counter];

        wasm_buy_now.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
            if attribute.key=="token_id" && attribute.value == token_id{
                purcjase_cart.push(
                    PurcjaseCartNft { 
                        transfer:TransferNft { 
                            collection: wasm.attributes[wasm_nft_id_index-4].value.clone(), 
                            sender:wasm.attributes[wasm_nft_id_index-2].value.clone(), 
                            recipient:wasm.attributes[wasm_nft_id_index-1].value.clone(), 
                            token_id:token_id.clone(), 
                            ts: ts.clone(), 
                            tx: tx.clone() }, 
                        buyer: wasm_buy_now.attributes[index+1].value.clone(), 
                        seller: wasm_buy_now.attributes[index+2].value.clone(), 
                        sale_price: wasm_buy_now.attributes[index+3].value.clone(), 
                        marketplace_fee: wasm_buy_now.attributes[index+4].value.clone(), 
                        royalties: wasm_buy_now.attributes[index+5].value.clone(),
                    }
                );
                index_counter+=1;
            }
        });
    }

    if purcjase_cart.len() >0{
        Some(purcjase_cart)
    }else {
        None
    }
}

async fn accept_bid_nft(events:Vec<Event>,tx:String,ts:String) -> Option<Vec<AcceptBidNft>>{
    
    let mut accpet_bid_nft_datas:Vec<AcceptBidNft>=vec![];
    let mut token_ids:Vec<String>=vec![];
    let mut token_id_indexs:Vec<usize>=vec![];

    let wasm=events.iter().find(|event|{event._type=="wasm"}).unwrap();
    let wasm_accept_bid=events.iter().find(|event|{event._type=="wasm-accept_bid"}).unwrap();

    wasm.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
        if attribute.key=="token_id"{
            token_ids.push(attribute.value.clone());
            token_id_indexs.push(index);
        }
    });

    let mut index_counter=0;
    for token_id in token_ids{

        let wasm_nft_id_index=token_id_indexs[index_counter];

        wasm_accept_bid.attributes.iter().enumerate().for_each(|(mut index,attribute)|{
            if attribute.key=="token_id" && attribute.value == token_id{
                accpet_bid_nft_datas.push(
                    AcceptBidNft { 
                        transfer: TransferNft { 
                            collection: wasm.attributes[wasm_nft_id_index-4].value.clone(), 
                            sender:wasm.attributes[wasm_nft_id_index-2].value.clone(), 
                            recipient:wasm.attributes[wasm_nft_id_index-1].value.clone(), 
                            token_id:token_id.clone(), 
                            ts: ts.clone(), 
                            tx: tx.clone() },  
                        bidder: wasm_accept_bid.attributes[index+3].value.clone(), 
                        seller: wasm_accept_bid.attributes[index+2].value.clone(), 
                        sale_price:wasm_accept_bid.attributes[index+1].value.clone(), 
                        marketplace_fee:wasm_accept_bid.attributes[index+8].value.clone(), 
                        royalties:wasm_accept_bid.attributes[index+12].value.clone()
                     }
                );
                index_counter +=1;
            }
        });
    }

    if accpet_bid_nft_datas.len() > 0{
        Some(accpet_bid_nft_datas)
    }else {
        None
    }


}

async fn only_create_aucton_nft(wasm_create_auction_attrs:Vec<Attribute>,ts:String) -> Option<Vec<OnlyCreateAuction>> {
    
    let mut auctions:Vec<OnlyCreateAuction>=vec![];

    wasm_create_auction_attrs.iter().enumerate().for_each(|(mut index,attr)|{
        if attr.key=="token_id"{
            auctions.push(
                OnlyCreateAuction{
                    collection_address:wasm_create_auction_attrs[index-1].value.clone(),
                    token_id:attr.value.clone(),
                    auction_price:wasm_create_auction_attrs[index+3].value.clone(),
                    ts:ts.clone()
                }
            )
        }
    });

    if auctions.len() >0{
        Some(auctions)
    }else {
        None
    }
}