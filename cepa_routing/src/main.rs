#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::or_fun_call)]

mod crypto;

use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use cepa_common::{NodeData, NodeList};

use std::thread::sleep;
use std::time::{Duration, Instant};

use std::time::{SystemTime, UNIX_EPOCH};

const CEPA_INDEX_HOST: &str = "cepa.ech0.ch";
const CEPA_INDEX_PORT: &str = "443";
const CEPA_INDEX_PUB_KEY: &str = "keyhere";

const CEPA_ROUTER_PORT: &str = "55505";

#[derive(Debug, Clone)]
struct Message {
    timestamp_received: u64,
    message: String,
}

#[derive(Debug, Clone)]
struct Log {
    list: Vec<Message>,
}

fn forward_message(next_hop: &str, message: &str) {
    let mut n_stream = TcpStream::connect(format!("{next_hop}:{CEPA_ROUTER_PORT}")).unwrap();
    n_stream.write_all(message.as_bytes()).unwrap();
}

fn timed_get_dir(data: Arc<Mutex<NodeList>>) {
    let delta = Duration::from_secs(5);
    let mut next_time = Instant::now() + delta;
    loop {
        get_dir(data.clone());
        sleep(next_time - Instant::now());
        next_time += delta;
    }
}

fn add_host(d: &NodeData) {
    let request = format!(
        "{{\"host\": \"{}\", \"pub_key\": \"{}\"}}",
        d.host, d.pub_key
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("https://{CEPA_INDEX_HOST}:{CEPA_INDEX_PORT}"))
        .header("Content-Type", "application/json")
        .body(request)
        .send()
        .unwrap();

    if response.status() == 200 {
        println!("OK");
    }
}

fn get_dir(data: Arc<Mutex<NodeList>>) {
    let resp = reqwest::blocking::get(format!("https://{CEPA_INDEX_HOST}:{CEPA_INDEX_PORT}/"))
        .unwrap()
        .json::<NodeList>()
        .unwrap();

    let mut d = data.lock().unwrap();
    if resp.timestamp > d.timestamp {
        d.timestamp = resp.timestamp;
        d.list = resp.list;
    }
}

fn handle_connection(mut stream: TcpStream, data: Arc<Mutex<NodeList>>, log: Arc<Mutex<Log>>) {
    // stream.write(b"connection handled").unwrap();

    // Read from tcp stream
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();

    let mut l = log.lock().unwrap();
    l.list.push(Message {
        timestamp_received: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        message: buf.clone(),
    });

    // Check if the message needs to be forwarded
    if buf.starts_with("sendto:") {
        let w: Vec<&str> = buf.split_whitespace().collect();
        let next_hop = w[1];
        let msg: Vec<&str> = w[2..].to_vec();
        let message = msg.join(" ");
        forward_message(next_hop, &message);

        // forward_message(next_hop, message);
    }
}

fn print_help() {
    println!(
        "\x1b[1mCommands:\x1b[0m
  \x1b[1mIndex:\x1b[0m
    \x1b[1;94mget\x1b[0m                HTTP GET NodeList from cepa_index
    \x1b[1;94madd\x1b[0m  HOST PUB_KEY  HTTP POST NodeData to cepa_index

  \x1b[1mNode:\x1b[0m
    \x1b[1;94mls(d)\x1b[0m              (Debug) Print NodeList
    \x1b[1;94mlog(d)\x1b[0m             (Debug) Print Log

    \x1b[1;94msend\x1b[0m HOST MSG      Send message to host
    \x1b[1;94mdate\x1b[0m               Show current unix time
    \x1b[1;94mflush\x1b[0m              Flush Log

    \x1b[1;94mclear\x1b[0m              Clear screen
    \x1b[1;94mexit\x1b[0m               Exit the cepa_router process
    \x1b[1;94mhelp\x1b[0m               Print this help"
    );
}

// Handle user input from stdin
fn handle_user_input(data: Arc<Mutex<NodeList>>, log: Arc<Mutex<Log>>) {
    let stdin = io::stdin();
    loop {
        let mut user_input = String::new();
        print!("cepa_router # ");
        let _ = io::stdout().flush();
        stdin.read_line(&mut user_input).unwrap();
        if !user_input.is_empty() {
            match user_input.split_whitespace().next().unwrap_or("\n") {
                "ls" => {
                    let d = data.lock().unwrap();
                    let width = 24;
                    println!("+-------------------------------------------------+");
                    println!(
                        "| \x1b[1;94mTIMESTAMP:\x1b[0m {n:<width$}             |",
                        n = d.timestamp
                    );
                    println!("+------------------------+------------------------+");
                    println!(
                        "|          \x1b[1;94mHOST\x1b[0m          |         \x1b[1;94mPUB_KEY\x1b[0m        |"
                    );
                    println!("+------------------------+------------------------+");

                    for node in &d.list {
                        println!("|{:width$}|{:width$}|", node.host, node.pub_key);
                    }
                    println!("+------------------------+------------------------+");
                }
                "lsd" => {
                    println!("{:#?}", data.lock().unwrap());
                }
                "get" => {
                    get_dir(data.clone());
                }
                "clear" => {
                    print!("\x1B[2J\x1B[1;1H");
                }
                "exit" => {
                    std::process::exit(0);
                }
                "help" => {
                    print_help();
                }
                "send" => {
                    if user_input.split_whitespace().count() == 3 {
                        let next_hop = user_input.split_whitespace().nth(1).unwrap();
                        let message = user_input.split_whitespace().nth(2).unwrap();
                        forward_message(next_hop, message);
                    } else {
                        println!("Usage: send HOST MESSAGE");
                    }
                }
                "add" => {
                    if user_input.as_str().split_whitespace().count() == 3 {
                        let host = user_input.split_whitespace().nth(1).unwrap().to_string();
                        let pub_key = user_input.split_whitespace().nth(2).unwrap().to_string();
                        add_host(&NodeData { host, pub_key });
                    } else {
                        println!("Usage: add HOST PUB_KEY");
                    }
                }
                "log" => {
                    let l = log.lock().unwrap();
                    for message in l.list.clone() {
                        println!("[{}] {}", message.timestamp_received, message.message);
                    }
                }
                "logd" => {
                    let l = log.lock().unwrap();
                    println!("{:#?}", l);
                }
                "date" => {
                    println!(
                        "{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );
                }
                "flush" => {
                    let mut l = log.lock().unwrap();
                    l.list = Vec::new();
                }
                _ => {
                    if user_input != "\n" {
                        user_input.pop();
                        println!("  Command \x1b[1;31m{user_input}\x1b[0m not found");
                    }
                }
            }
        }
    }
}

fn main() {
    print!("\x1b[92m");
    println!(
        r"
     ▄████▄  ▓█████  ██▓███   ▄▄▄          ██▀███  ▄▄▄█████▓ ██▀███  
    ▒██▀ ▀█  ▓█   ▀ ▓██░  ██▒▒████▄       ▓██ ▒ ██▒▓  ██▒ ▓▒▓██ ▒ ██▒
    ▒▓█    ▄ ▒███   ▓██░ ██▓▒▒██  ▀█▄     ▓██ ░▄█ ▒▒ ▓██░ ▒░▓██ ░▄█ ▒
    ▒▓▓▄ ▄██▒▒▓█  ▄ ▒██▄█▓▒ ▒░██▄▄▄▄██    ▒██▀▀█▄  ░ ▓██▓ ░ ▒██▀▀█▄  
    ▒ ▓███▀ ░░▒████▒▒██▒ ░  ░ ▓█   ▓██▒   ░██▓ ▒██▒  ▒██▒ ░ ░██▓ ▒██▒
    ░ ░▒ ▒  ░░░ ▒░ ░▒▓▒░ ░  ░ ▒▒   ▓▒█░   ░ ▒▓ ░▒▓░  ▒ ░░   ░ ▒▓ ░▒▓░
      ░  ▒    ░ ░  ░░▒ ░       ▒   ▒▒ ░     ░▒ ░ ▒░    ░      ░▒ ░ ▒░
    ░           ░   ░░         ░   ▒        ░░   ░   ░        ░░   ░ 
    ░ ░         ░  ░               ░  ░      ░                 ░     
    ░                                                                
        "
    );
    print!("\x1b[0m");
    match TcpListener::bind(format!("0.0.0.0:{}", CEPA_ROUTER_PORT)) {
        Ok(listener) => {
            let data: Arc<Mutex<NodeList>> = Arc::new(Mutex::new(NodeList {
                timestamp: 0,
                list: Vec::new(),
            }));

            let log: Arc<Mutex<Log>> = Arc::new(Mutex::new(Log { list: Vec::new() }));

            let mut data_clone = data.clone();

            thread::spawn(move || {
                timed_get_dir(data_clone);
            });

            data_clone = data.clone();
            let mut log_clone = log.clone();
            thread::spawn(move || {
                handle_user_input(data_clone, log_clone);
            });

            for stream in listener.incoming() {
                let stream = stream.unwrap();
                data_clone = data.clone();
                log_clone = log.clone();
                thread::spawn(move || {
                    handle_connection(stream, data_clone, log_clone);
                });
            }
        }
        Err(_) => {
            panic!("Could not bind on port {}", CEPA_ROUTER_PORT);
        }
    }
}
