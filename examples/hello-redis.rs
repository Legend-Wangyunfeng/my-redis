use mini_redis::{client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // 连接到 Redis 服务器
    let mut client = client::connect("127.0.0.1:6379").await?;

    // 设置键值对
    let res = client.set("hello", "world".into()).await?;


    // 获取
    let result = client.get("hello").await?;

    println!("{:?}", result.unwrap());
    Ok(())
}
