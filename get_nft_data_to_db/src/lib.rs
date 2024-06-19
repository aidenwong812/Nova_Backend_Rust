mod pgsql;
mod get_data;

use tokio_postgres::{NoTls, Error};
use dotenv::dotenv;
use std::env;



#[cfg(test)]
mod tests {
    use super::*;



    #[tokio::test]
    async fn test() -> Result<(), Error> {

        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
    
        // 连接成功，可以在这里执行数据库操作
        println!("Successfully connected to the database");

        

        // // 执行查询
        // let rows = client
        //     .query("SELECT * FROM my_table", &[])
        //     .await?;

        // // 处理查询结果
        // for row in &rows {
        //     let id: i32 = row.get(0);
        //     let name: &str = row.get(1);
        //     println!("id: {}, name: {}", id, name);
        // }

        // 等待连接任务完成
        
        Ok(())
    }

}
