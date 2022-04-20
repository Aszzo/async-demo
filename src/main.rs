use async_demo::{TimerFuture, new_executor_and_spawner};
use std::time::{ Duration, SystemTime};
use std::io::Error;

fn main() -> Result<(), Error>{
    println!("{:?}",SystemTime::now());
    let (executor, spawner) = new_executor_and_spawner();

    spawner.spawn(async {
        println!("ready");
        // 创建定时器Future，并等待它完成
        TimerFuture::new(Duration::new(2, 0)).await;
        println!("done!");
    });

    drop(spawner);

    executor.run();

    Ok(())
}