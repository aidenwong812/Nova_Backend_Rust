pub mod tables;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::Value;
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult}, types::Json, Connection, PgConnection, Pool
    };
use std::{env, error::Error};
use dotenv::dotenv;

use tables::*;

use sei_client::{ apis::_apis::{get_contract_info, get_nft_info}, field_data::field_data_structions::{Collection, NFTtransaction, NftToken, TokenSwap, User,FixedSellNft,OnlyCreateAuction,ContractCreateAuctions}};





pub async fn client_db() -> Result<PgConnection,Box<dyn Error>> {
    
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    // 连接到数据库
    let connect = PgConnection::connect(&database_url).await?;
    
    Ok(connect)
}


#[derive(Debug)]
pub enum NftCollectionDbOperate {
    SearchQuery(Option<Collection>),
    InsertOrUpdate(Result<PgQueryResult,Box<dyn Error>>)
}

pub enum NftCollectionDb {
    Search(String),
    Insert(Collection),
    Count{collection_account:String,count:String},
    Nfts{collection_account:String,nfts:Vec<NftToken>},

}impl NftCollectionDb {

    pub async fn operate(op:NftCollectionDb,conn:&mut PgConnection) -> NftCollectionDbOperate {
        match op {
            NftCollectionDb::Search(collection_account)=>{
                let data=NftCollectionDb::search_collection(&collection_account, conn).await;
                return NftCollectionDbOperate::SearchQuery(data);

            },
            NftCollectionDb::Insert(collection)=>{
                let data=NftCollectionDb::insert_collection(collection, conn).await;
                return NftCollectionDbOperate::InsertOrUpdate(data);
            },
            NftCollectionDb::Count { collection_account, count }=>{
                let data=NftCollectionDb::update_count(&collection_account, conn, count).await;
                return NftCollectionDbOperate::InsertOrUpdate(data);
            },
            NftCollectionDb::Nfts { collection_account, nfts}=>{
                let data=NftCollectionDb::update_nfts(&collection_account, conn,  nfts).await;
                return  NftCollectionDbOperate::InsertOrUpdate(data);
            },
        }
    }
    
    async fn search_collection(collection_account:&str,conn:&mut PgConnection) -> Option<Collection> {
        
        let res=sqlx::query_as::<_,_Collection>(
            r#"SELECT collection , name , symbol , creator , count , nfts FROM "NFTSCollection" WHERE collection =$1"#
        )
        .bind(collection_account)
        .fetch_one(conn).await;

        if let Ok(data) =res  {

            let nfts=serde_json::from_value::<Vec<NftToken>>(data.nfts).unwrap();
            
            Some(Collection{
                collection:data.collection,
                name:data.name,
                symbol:data.symbol,
                creator:data.creator,
                count:data.count,
                nfts:nfts
            })
        }else {
            None
        }
    }

    async fn insert_collection(collection:Collection,conn:&mut PgConnection) -> Result<PgQueryResult,Box<dyn Error>> {
        
        let insert=sqlx::query::<sqlx::Postgres>(
             r#"INSERT INTO "NFTSCollection" (collection, name, symbol, creator,count,nfts) VALUES ($1, $2, $3, $4,$5,$6)"#
        )
        .bind(collection.collection)
        .bind(collection.name)
        .bind(collection.symbol)
        .bind(collection.creator)
        .bind(collection.count)
        .bind(serde_json::to_value(&collection.nfts).unwrap())
        .execute(conn).await;

        if let Ok(insert_option) =insert  {
            Ok(insert_option)
        }else {
            Err("insert data erro".into())
        }
       
    }

    async fn update_count(collection_account:&str,conn:&mut PgConnection,count:String) -> Result<PgQueryResult,Box<dyn Error>> {
        
        let res=sqlx::query(
            r#"UPDATE "NFTSCollection SET count =$2 WHERE collection =$1"#
        )
        .bind(collection_account)
        .bind(count)
        .execute(conn)
        .await;

        
        if let Ok(result) =res  {
            Ok(result)
        }else {
            Err("update count to NFTSCollection erro".into())
        }

    }

    async fn update_nfts(collection_account:&str,conn:&mut PgConnection,_nfts: Vec<NftToken>) -> Result<PgQueryResult,Box<dyn Error>> {
        
        if let Some(data)=NftCollectionDb::search_collection(collection_account, conn).await{
            
            // let mut _nft=_nfts;
            let mut nfts=data.nfts;
            nfts.extend(_nfts.iter().cloned());

            
            let res=sqlx::query(
                r#"UPDATE "NFTSCollection SET nfts =$2 WHERE collection =$1"#
            )
            .bind(collection_account)
            .bind(serde_json::to_value(nfts).unwrap())
            .execute(conn)
            .await;

            if let Ok(result) =res  {
                Ok(result)
            }else {
                Err("update nfts to NFTSCollection erro".into())
            }

        }else {
            Err("don't have this cellection account".into())
        }

        
    }




}

pub async fn search_user(wallet_address:&str,conn:&mut PgConnection) ->Option<User>{
        
        let res=sqlx::query_as::<_,_User>(
            r#"SELECT wallet_address, nfts_holding, nfts_transactions, token_transactions FROM "User" WHERE wallet_address = $1"#
        )
        .bind(wallet_address)
        .fetch_one(conn).await;

        if let Ok(data) =res  {

            let nfts_holding=serde_json::from_value::<Vec<Collection>>(data.nfts_holding).unwrap();
            let nfts_transactions=serde_json::from_value::<Vec<NFTtransaction>>(data.nfts_transactions).unwrap();
            let token_transactions=serde_json::from_value::<Vec<TokenSwap>>(data.token_transactions).unwrap();

            Some(User { 
                wallet_address: data.wallet_address, 
                nfts_holding: nfts_holding, 
                nfts_transactions: nfts_transactions, 
                token_transactions: token_transactions }
            )
        }else {
            None
        }
}

pub async fn insert_user(user:User,conn:&mut PgConnection) -> Option<PgQueryResult> {
        
        let insert=sqlx::query::<sqlx::Postgres>(
            r#"INSERT INTO "User" (wallet_address, nfts_holding, nfts_transactions, token_transactions) VALUES ($1, $2, $3, $4)"#
        )
        .bind(user.wallet_address)
        .bind(serde_json::to_value(user.nfts_holding).unwrap())
        .bind(serde_json::to_value(user.nfts_transactions).unwrap())
        .bind(serde_json::to_value(user.token_transactions).unwrap())
        .execute(conn).await;

        
        if let Ok(result) = insert {
            Some(result)
        }else {
            // Err("inster user erro".into())
            None
        }

}

pub async fn update_nfts_holding(wallet_address:&str,collection_account:&str,nft_id:&str,operate:&str,conn:&mut PgConnection) -> Option<PgQueryResult> {
        
        let nft_res=get_nft_info(collection_account.to_string(), nft_id.to_string()).await;
        if nft_res.is_none(){
            return None;
        }

        let nft=nft_res.unwrap();

        if let Some(mut user) =search_user(wallet_address, conn).await  {
            
          
            // 寻找用户的collection ，如果 有进行 追加或 删除
            if let Some(mut nfts_holding)  = user.nfts_holding.iter_mut().find(|collection|{collection.collection ==collection_account}) {
                
                if operate=="add"{
                    
                    if  nfts_holding.nfts.iter().find(|_nft|{_nft.key==nft.key}).is_none(){
                        nfts_holding.nfts.push(nft);
                    }

                }else if operate=="del" {
                   

                    if let Some(remove_nft)=nfts_holding.nfts.iter().position(|_nft|_nft.key==nft.key){
                        nfts_holding.nfts.remove(remove_nft);
                        
                    }
                }

                if nfts_holding.nfts.len() == 0{
                   if let Some(remove_collection)=user.nfts_holding.iter().position(|collection|collection.collection==collection_account){
                    user.nfts_holding.remove(remove_collection);
                   }
                } 
            
            }else {
                
                if operate=="add"{
                    
                    if let Some(res)=get_contract_info(collection_account.to_string()).await{
                            user.nfts_holding.push(Collection{
                                collection:collection_account.to_string(),
                                name:res.name,
                                symbol:res.symbol,
                                creator:res.creator,
                                count:res.count,
                                nfts:vec![nft],
                            })
                        
                    }
                }
            }

            let update=sqlx::query(
                 r#"UPDATE "User" SET nfts_holding =$2 WHERE wallet_address = $1 "#
            )
            .bind(wallet_address)
            .bind(serde_json::to_value(user.nfts_holding).unwrap())
            .execute(conn)
            .await;

            
            if let Ok(res)=update{
                Some(res)
            }else {
                // Err("update holding nft to user erro".into())
                None
            }


        }else {

            let mut nfts_holding:Vec<Collection>=vec![];
            let mut nfts_transactions:Vec<NFTtransaction>=vec![];
            let mut token_transactions:Vec<TokenSwap>=vec![];
            
            if operate =="add"{
                if let Some(res)=get_contract_info(collection_account.to_string()).await{
                  
                        nfts_holding.push(Collection{
                            collection:collection_account.to_string(),
                            name:res.name,
                            symbol:res.symbol,
                            creator:res.creator,
                            count:res.count,
                            nfts:vec![nft],
                        })
                    
                }
            }
            
            
            let user=User{
                wallet_address:wallet_address.to_string(),
                nfts_holding:nfts_holding,
                nfts_transactions:nfts_transactions,
                token_transactions:token_transactions,
            };

            if let Some(res) =insert_user(user, conn).await {
                Some(res)
            }else {
                // Err("update nfts holding err  || inster user err".into())
                None
            }

        }
}


pub async fn update_nfts_transactions(wallet_address:&str,conn:&mut PgConnection,nfts_transaction:Vec<NFTtransaction>) -> Option<PgQueryResult> {
        
        
        
        
        if let Some(UserData) =search_user(wallet_address, conn).await  {
            
            let mut nfts_transactions=UserData.nfts_transactions;
            nfts_transactions.extend(nfts_transaction.iter().cloned());
            
            let update=sqlx::query(
                 r#"UPDATE "User" SET nfts_transactions =$2 WHERE wallet_address = $1 "#
            )
            .bind(wallet_address)
            .bind(serde_json::to_value(nfts_transactions).unwrap())
            .execute(conn).await;
            
            if let Ok(res) = update {
                Some(res)
            }else {
                // Err("update nfts transaction err ".into())
                None
            }
        }else {

            
            let mut nfts_holding:Vec<Collection>=vec![];
            let mut nfts_transactions:Vec<NFTtransaction>=vec![];
            let mut token_transactions:Vec<TokenSwap>=vec![];

            nfts_transactions.extend(nfts_transaction.iter().cloned());

            let user=User{
                wallet_address:wallet_address.to_string(),
                nfts_holding:nfts_holding,
                nfts_transactions:nfts_transactions,
                token_transactions:token_transactions,
            };


            if let Some(res) =insert_user(user, conn).await {
                Some(res)
            }else {
                // Err("update nfts transaction err  || inster user err".into())
                None
            }

        }
     
}

    
pub async fn update_token_transaction(wallet_address:&str,conn:&mut PgConnection,token_transaction:Vec<TokenSwap>) -> Option<PgQueryResult> {
        
        if let Some(UserData) =search_user(wallet_address, conn).await  {
            
            let mut token_transactions=UserData.token_transactions;
            token_transactions.extend(token_transaction.iter().cloned());
            
            let update=sqlx::query(
                 r#"UPDATE "User" SET token_transactions =$2 WHERE wallet_address = $1 "#
            )
            .bind(wallet_address)
            .bind(serde_json::to_value(token_transactions).unwrap())
            .execute(conn).await;
            
            if let Ok(res) = update {
                Some(res)
            }else {
                // Err("update nfts transaction err ".into())
                None
            }
        }else {

            
            let mut nfts_holding:Vec<Collection>=vec![];
            let mut nfts_transactions:Vec<NFTtransaction>=vec![];
            let mut token_transactions:Vec<TokenSwap>=vec![];

            token_transactions.extend(token_transaction.iter().cloned());

            let user=User{
                wallet_address:wallet_address.to_string(),
                nfts_holding:nfts_holding,
                nfts_transactions:nfts_transactions,
                token_transactions:token_transactions,
            };


            if let Some(res) = insert_user(user, conn).await {
                Some(res)
            }else {
                // Err("update nfts transaction err  || inster user err".into())
                None
            }

        }
     
}



//////////////////////////////////////////////

pub async fn search_contract_createauctions(contract_address:&str,conn:&mut PgConnection) -> Option<ContractCreateAuctions> {
    
    let res=sqlx::query_as::<_,_ContractCreateAuctions>(
        r#"SELECT contract_address ,create_auction_transactions FROM "ContractsCreateAuction" WHERE contract_address = $1"#
    )
    .bind(contract_address)
    .fetch_one(conn).await;

    if let Ok(data)=res  {

        let contract_create_auctions=serde_json::from_value::<Vec<OnlyCreateAuction>>(data.create_auction_transactions).unwrap();
        Some(ContractCreateAuctions { contract_address: data.contract_address, create_auctions: contract_create_auctions })
    }else {
        None
    }
}

pub async fn inster_ContractsCreateAuction(contranct_create_auctions:ContractCreateAuctions,conn:&mut PgConnection) -> Option<PgQueryResult> {
    
    let insert=sqlx::query::<sqlx::Postgres>(
        r#"INSERT INTO "ContractsCreateAuction" (contract_address, create_auction_transactions) VALUES ($1,$2)"#
    )
    .bind(contranct_create_auctions.contract_address)
    .bind(serde_json::to_value(contranct_create_auctions.create_auctions).unwrap())
    .execute(conn).await;
    
    if let Ok(result) =insert  {
        Some(result)
    }else {
        None
    }
}

pub async fn update_contract_create_auctions(contract_address:&str,create_auctions:Vec<OnlyCreateAuction>, conn:&mut PgConnection) -> Option<PgQueryResult> {
    if let Some(contract) =search_contract_createauctions(contract_address, conn).await  {

        let day_now=Utc::now().date_naive();

        let mut contract_create_auctions=contract.create_auctions;

        // 删除非今天        
        contract_create_auctions.retain(|auction|{
            let ts=DateTime::parse_from_rfc3339(&auction.ts).unwrap().with_timezone(&Utc).date_naive();
            ts == day_now
        });

        contract_create_auctions.extend(create_auctions.iter().cloned());


        let update=sqlx::query(
            r#"UPDATE "ContractsCreateAuction" SET create_auction_transactions =$2 WHERE contract_address = $1 "#
        )
        .bind(contract_address)
        .bind(serde_json::to_value(contract_create_auctions).unwrap())
        .execute(conn).await;

        if let Ok(result) =update  {
            Some(result)
        }else {
            None
        }


    }else {
        let contranct_create_auctions=ContractCreateAuctions{
            contract_address:contract_address.to_string(),
            create_auctions:create_auctions.clone()
        };
        if let Some(result) =inster_ContractsCreateAuction(contranct_create_auctions, conn).await {
            Some(result)
        }else {
            None
        }
    }
}




mod tests{

    use sei_client::field_data::field_data_structions::{AcceptBidNft, NftAttribute, TransferNft, _NftTransaction};

    use super::*;
    #[tokio::test]
    async fn test1()  {
       
        let mut conn=client_db().await.unwrap();
        let a=search_user("1111111111", &mut conn).await;
      
        println!("{:?}",a)

        // let usr2=User{
        //     wallet_address:"1111111111".to_string(),
        //     nfts_holding:vec![],
        //     nfts_transactions:vec![],
        //     token_transactions:vec![],
        // };


        // let nft1=NftToken{
        //     token_id:"1".to_string(),
        //     name:"kasserrr".to_string(),
        //     key:"ssssss".to_string(),
        //     image:"abc".to_string(),
        //     royalty_percentage:5,
        //     attributes:vec![NftAttribute{
        //         trait_type:"66".to_string(),
        //         value:"2".to_string()
        //     }]
        // };

        // let nft_transaction1=NFTtransaction{
        //     transaction:_NftTransaction::AcceptBid(AcceptBidNft{
        //         transfer:TransferNft{
        //             collection:"11111".to_string(),
        //             sender:"me".to_string(),
        //             recipient:"you".to_string(),
        //             token_id:"1".to_string(),
        //             ts:"sss".to_string(),
        //             tx:"111111".to_string(),
        //         },
        //         bidder:"me".to_string(),
        //         seller:"me".to_string(),
        //         sale_price:"10000sei".to_string(),
        //         marketplace_fee:"11".to_string(),
        //         royalties:"5".to_string()

        //     }),
        //     _type:"buy_nft".to_string()
        // };



        // let data=UserDb::search_user("1ssssssss", &mut conn).await;
        // let data=UserDb::operate(UserDb::Search("1ssssssss".to_string()), &mut conn).await;
        // println!("{:?}",data);

        // let res=UserDb::operate(UserDb::InsertUser(usr2), &mut conn).await;
        // println!("{:?}",res)

        // let res=UserDb::operate(UserDb::NftHolding { 
        //     wallet_address:"211111111111".to_string(), 
        //     collection_account:"sei1v90ly54qeu7497lzk2mnmp2h29sgtep8hs5ryvfqf8dwq5gc0t9srp6aey".to_string(), 
        //     nft: nft1, operate: "del".to_string() }, 
        //     &mut conn).await;

        // println!("{:?}",res);

        // let res=UserDb::operate(UserDb::NftTransaction { wallet_address:"2111111111111".to_string(), nfts_transaction: vec![nft_transaction1] }, &mut conn).await;
        // println!("{:?}",res);

    


    }
}

