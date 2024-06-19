pub mod nft_apis;
use nft_apis::user::get_holding_nfts;
#[cfg(test)]
mod tests{
    use super::*;
   

    #[tokio::test]
    async fn test1(){
        //  sei1krvjk3r790dcsqkr96ymd44v04w9zz5dlr66z7 sei13m2p7l52gksmnguzz76ga52sxr8f70aqwjwkqr
        let wallet_address="sei1wfpu46r5vjhth7akfu09yc0qtzm93c2z9mv3jh".to_string();
        get_holding_nfts(wallet_address).await;
       
        
    }


}