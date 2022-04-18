use anyhow::Result;
use serde_yaml::Value;
// use std::fs;
use toml::from_str;
use tokio::{fs, try_join};

// 1. 同步形式 time = 同步任务耗时之和
// fn main() -> Result<()> {
//     let content1 = fs::read_to_string("Cargo.toml")?;
//     let content2 = fs::read_to_string("Cargo.lock")?;
//
//     let value1 = toml2yml(&content1)?;
//     let value2 = toml2yml(&content2)?;
//
//     fs::write("tmp/Cargo.yml", &value1)?;
//     fs::write("tmp/Cargo.lock.yml", &value2)?;
//
//     println!("执行完毕");
//
//     Ok(())
//
//
// }

// 2. 异步 time = 耗时最长的任务
#[tokio::main]
async fn main() -> Result<()>{
    let f1 = fs::read_to_string("Cargo.toml");
    let f2 = fs::read_to_string("Cargo.lock");

    let (content1, content2) = try_join!(f1,f2)?;

    let value1 = toml2yml(&content1)?;
    let value2 = toml2yml(&content2)?;

    let f3 = fs::write("tmp/Cargo.yml", &value1);
    let f4 = fs::write("tmp/Cargo.lock.yml", &value2);

    try_join!(f3, f4)?;

    println!("执行完毕");

    Ok(())

}

fn toml2yml(content: &str) -> Result<String> {
    let value = from_str::<Value>(content)?;
    Ok(serde_yaml::to_string(&value)?)
}
