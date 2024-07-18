pub mod apis;
pub mod field_data;

#[cfg(test)]
mod tests{
    use super::*;
    use apis::_apis::{get_contract_info, get_last_height, get_transaction_txs_by_event, get_transaction_txs_by_tx, get_txs_by_block_height};

    #[tokio::test]
    async fn test1(){
        
        // let hash="51029AE4B490D51B6F3DAE48D8743D3F252A489F2FDBDBD37023EC3BFACC7D9D";
        // let data1=get_transaction_txs_by_tx(hash).await;
        // let data1=get_last_height().await.unwrap();
        // let data2=get_txs_by_block_height(data1 ).await;
        // println!("{:?}",data1);
        // println!("{:?}",data2);
        
        // let a=query_contract_smart_type::searcher_contract_smrat_info("sei1g2a0q3tddzs7vf7lk45c2tgufsaqerxmsdr2cprth3mjtuqxm60qdmravc".to_string());
        
        // let a=query_contract_smart_type::get_data(a).await.unwrap();
        // println!("{:?}",a);
    
        // let b=query_contract_smart_type::searcher_nft_token_info { contract_address:"sei1g2a0q3tddzs7vf7lk45c2tgufsaqerxmsdr2cprth3mjtuqxm60qdmravc".to_string() , token_id: "22".to_string() };
        // let a=query_contract_smart_type::get_data(b).await.unwrap();
        
        let a=get_contract_info("sei1g2a0q3tddzs7vf7lk45c2tgufsaqerxmsdr2cprth3mjtuqxm60qdmravc".to_string() ).await;
        println!("{:?}",a);
 
        //  sei1ts53rl9eqrdjd82hs2em7hv8g6em4xye67z9wxnhdrn4lnf8649sxtww22

        // let c=query_contract_smart_type::get_all_nft_info_about_collection{contract_address:"sei1g2a0q3tddzs7vf7lk45c2tgufsaqerxmsdr2cprth3mjtuqxm60qdmravc".to_string(),max_tasks:10};
        // let b=query_contract_smart_type::get_data(c).await;
        // println!("{:?}",b);
    }

    


}

