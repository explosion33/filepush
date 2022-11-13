use std::thread;

use reqwest;

use futures::{self, StreamExt};
use tokio;

use std::fs::OpenOptions;
use std::io::prelude::*;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("Error | Could not create runtime");
    rt.block_on(stream("explosion33", "password", ""));
}

async fn stream(username: &str, password: &str, path: &str) {
    let client = reqwest::Client::new();
    let res = match client.get("http://localhost/events")
    .header("username", username)
    .header("password", password)
    .send()
    .await
    {
        Ok(n) => n,
        Err(_) => {
            println!("Error establishing connection with server");
            return;
        } 
    };

    let res = match res.error_for_status() {
        Ok(n) => n,
        Err(n) => {
            println!("server returned an error | {}", n);
            return;
        },
    };

    let mut stream = res.bytes_stream();


    while let Some(item) = stream.next().await {        
        let mut data = match item {
            Ok(n) => {match String::from_utf8(n.to_vec()) {
                Ok(n) => n,
                Err(_) => {continue;},
            }},
            Err(_) => {continue;},
        };

        if !data.starts_with("data:[") {
            continue;
        }

        data.replace_range(0..6, "");
        data.replace_range(data.len()-2.., "");

        for file in data.split(",") {
            let mut name = file.trim().to_string();
            name.replace_range(..1, "");
            name.replace_range(name.len()-1.., "");

            println!("{}", name);

            let username2 = username.to_string();
            let password2 = password.to_string();
            let path2 = path.to_string();
            thread::spawn(move || {
                let res = download_file(name.as_str(), username2.as_str(), password2.as_str(), path2.as_str());
                
                match res {
                    Ok(_) => {},
                    Err(n) => {
                        println!("{:#?}", n);
                    }
                }
                
            });

        }
    }
}

fn download_file(name: &str, username: &str, password: &str, path: &str) -> Result<(), String> {
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("{}{}", path, name)) {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error opening file | {}", n))},
        };

    let client = reqwest::blocking::Client::new();
    let data = match client.get(format!("http://localhost/file/{}", name))
        .header("username", username)
        .header("password", password)
        .send()
        {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error getting file from server | {}", n))} 
        };

    match file.write_all(&match data.bytes() {
        Ok(n) => n,
        Err(_) => {
            return Err("Error getting file bytes".to_string());
        }
    }) {
        Ok(_) => {},
        Err(n) => {
            return Err(format!("Error writing to file | {}", n));
        }
    };

    Ok(())
}