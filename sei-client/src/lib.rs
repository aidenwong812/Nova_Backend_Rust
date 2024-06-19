pub mod apis;

#[cfg(test)]
mod tests{
    use super::*;
    use apis::_apis::{get_transaction_txs_by_event, get_transaction_txs_by_tx,get_last_height,get_txs_by_block_height};

    #[tokio::test]
    async fn test1(){
        
        // let hash="51029AE4B490D51B6F3DAE48D8743D3F252A489F2FDBDBD37023EC3BFACC7D9D";
        // let data1=get_transaction_txs_by_tx(hash).await;
        let data1=get_last_height().await.unwrap();
        let data2=get_txs_by_block_height(data1 ).await;
        println!("{:?}",data1);
        println!("{:?}",data2);
        
    }


}