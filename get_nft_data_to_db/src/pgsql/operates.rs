use std::error::Error;
use serde_json;
use tokio;
use tokio_postgres::{Client,NoTls};
use super::tables;
use field_blockchain_data::data_structions::{NftHoldTransaction,TokensTranscations,NftToken};

async fn connect_sql(database_url:&str) -> Result<Client,Box<dyn Error>> {

    let (client,connection)=tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Successfully connected to the db");

    Ok(client)
}


async fn create_tables(client:Client) -> Result<(),Box<dyn Error>> {
    
    //create wallet info table 
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS Wallet_Info_Nft (
                wallet_address VARCHAR PRIMARY KEY,
                nft_holding JSONB,
                nfts_transcations JSONB,
                tokens_transcations JSONB,
            )",
            &[],
        )
        .await?;

    // create Collection Info table
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS Collection_Info (
                sei_address VARCHAR PRIMARY KEY,
                evm_address VARCHAR,
                creater VARCHAR,
                name VARCHAR,
                symbol VARCHAR,
                pfp VARCHAR,
                items VARCHAR,
                tokens JSONB
            )", 
            &[])
            .await?;

    Ok(())

}

pub enum Insert_data_to_db {
    
    Wallet_Info_Nft(tables::WalletNftInfo),
    
    Nft_Holding(Vec<tables::HoldingNft>),
    
    Nfts_Transcations(Vec<NftHoldTransaction>),
    
    Tokens_Transactions(Vec<TokensTranscations>),

    Collection_Info_Db(tables::CollectionInfoDb),

    Collection_Tokens(Vec<NftToken>)

}impl Insert_data_to_db {
    pub async fn insert_data(data_type:Insert_data_to_db) -> Result<(),Box<dyn Error>> {
        match data_type {
            Insert_data_to_db::Wallet_Info_Nft(data)=>{},
            Insert_data_to_db::Nft_Holding(data)=>{},
            Insert_data_to_db::Nfts_Transcations(data)=>{},
            Insert_data_to_db::Tokens_Transactions(data)=>{},
            Insert_data_to_db::Collection_Info_Db(data)=>{},
            Insert_data_to_db::Collection_Tokens(data)=>{},
            _=>{panic!("don't have this enmu");},
        }
        Ok(())
    }
}