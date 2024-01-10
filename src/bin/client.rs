
use bytes::Bytes;
use tokio::sync::{oneshot, mpsc};
use mini_redis::client;

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        value: Bytes,
        resp: Responder<()>,
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);

    let tx2 = tx.clone();
    // 启动一个异步任务来处理命令
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: String::from("hello"),
            resp: resp_tx,
        };
        tx.send(cmd).await.unwrap();

        let res = resp_rx
            .await;
        println!("{:?}", res);

    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: String::from("hello"),
            value: Bytes::from("world"),
            resp: resp_tx,
        };

        tx2.send(cmd).await.unwrap();
    });

    // 等待接收到的命令
    while let Some(msg) = rx.recv().await {
        println!("Received message: {:?}", msg);
    }

    let manager = tokio::spawn(async move {
        //建立连接
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        //开始接收消息
        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key , resp} => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, value , resp } => {
                    let res = client.set(&key, value).await;
                    let _ = resp.send(res);
                }
            }

        }
    });
    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}