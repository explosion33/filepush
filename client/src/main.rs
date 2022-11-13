use std::{thread, io::{BufReader, BufWriter}};

use reqwest;

use futures::{self, StreamExt};
use tokio;

use std::fs::OpenOptions;
use std::io::prelude::*;

use serde::{Serialize, Deserialize};
use serde_json;

use dirs::document_dir;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Settings {
    pub username: String,
    pub password: String,
    pub url: String,
    pub path: String
}

impl Settings {
    fn new(path: &str) -> Result<Settings, String> {
        let file = match OpenOptions::new()
            .read(true)
            .open(path)
            {
                Ok(n) => n,
                Err(n) => {return Err(format!("Error opening file | {}", n))},
            };
        let reader  = BufReader::new(file);
        
        match serde_json::from_reader(reader) {
            Ok(n) => Ok(n),
            Err(n) => Err(format!("Error | {}", n)) 
        }
    }
}

macro_rules! input {
    {} => {{
        input!("")
    }};

    ($a:expr) => {{
        use std::io;
        use std::io::Write;

        print!("{}", $a);
        let _ = io::stdout().flush();

        let mut line = String::new();
        io::stdin().read_line(&mut line).expect("Error reading from stdin");
        line.trim().to_string()
    }};
}


fn prompt_settings() -> Settings {
    println!("First Time Setup:");
    let username = input!("Enter Username: ");
    let password = input!("Enter Password: ");
    let url = input!("Enter FilePush server URL: ");
    let path = input!("Enter Path to download files to: ");

    Settings { username, password, url, path}
}

fn main() {
    let settings = match Settings::new(format!("{}\\filepush\\settings.txt", document_dir().unwrap().to_str().unwrap()).as_str()) {
        Ok(n) => n,
        Err(_) => {
            let settings = prompt_settings();

            let file = match OpenOptions::new()
                .create(true)
                .write(true)
                .open(format!("{}\\filepush\\settings.txt", document_dir().unwrap().to_str().unwrap()))
                {
                    Ok(n) => n,
                    Err(n) => {
                        println!("Error getting settings file | {}", n);
                        return;
                    },
                };
            let writer  = BufWriter::new(file);

            serde_json::to_writer(writer, &settings).expect("error writing settings to file");
            settings
        } 
    };

    println!("{:?}", settings);

    let rt = tokio::runtime::Runtime::new().expect("Error | Could not create runtime");
    rt.block_on(stream(&settings));
}

async fn stream(settings: &Settings) {
    let client = reqwest::Client::new();
    let res = match client.get(format!("{}/events", settings.url))
    .header("username", &settings.username)
    .header("password", &settings.password)
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

            let settings_clone = settings.clone();
            let name_clone = name.to_string() ;
            thread::spawn(move || {
                let res = download_file(settings_clone, name_clone.as_str());
                
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

fn download_file(settings: Settings, name: &str) -> Result<(), String> {
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .open(format!("{}\\{}", settings.path, name)) {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error opening file | {}", n))},
        };

    println!("{}/{}", settings.url, name);
    let client = reqwest::blocking::Client::new();
    let data = match client.get(format!("{}/file/{}", settings.url, name))
        .header("username", settings.username)
        .header("password", settings.password)
        .send()
        {
            Ok(n) => n,
            Err(n) => {return Err(format!("Error getting file from server | {}", n))} 
        };

    let data = match data.error_for_status() {
        Ok(n) => n,
        Err(n) => {return Err(format!("Error | {}", n))}
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