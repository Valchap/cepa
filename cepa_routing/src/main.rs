#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::or_fun_call)]

mod crypto;

use std::{
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use cepa_common::{NodeData, NodeList};
use crypto::{unwrap_layer, wrap_layer};
use rsa::{RsaPrivateKey, RsaPublicKey};

use std::thread::sleep;
use std::time::{Duration, Instant};

use std::time::{SystemTime, UNIX_EPOCH};

const CEPA_INDEX_HOST: &str = "cepa.ech0.ch";
const CEPA_INDEX_PORT: &str = "443";
const CEPA_INDEX_PUB_KEY: &str = "keyhere";

const CEPA_ROUTER_PORT: u16 = 55505;

const RSA_KEY_BITS: usize = 2048;

const AUTO_REFRESH_RATE: u64 = 5;

#[derive(Debug, Clone)]
struct Message {
    timestamp_received: u64,
    message: String,
}

#[derive(Debug, Clone)]
struct MessageLog {
    list: Vec<Message>,
}

impl MessageLog {
    pub const fn new() -> Self {
        Self { list: Vec::new() }
    }
}

struct SharedData {
    priv_key: RsaPrivateKey,
    node_list: NodeList,
    message_log: MessageLog,
}

fn send_message(data: &[u8], destination: [u8; 4], shared_data: &Arc<Mutex<SharedData>>) {
    let prepared_data = wrap_layer(
        &(
            [127, 0, 0, 1],
            &RsaPublicKey::from(&shared_data.lock().unwrap().priv_key),
        ),
        data,
    );

    let socket_addr = SocketAddr::from((destination, CEPA_ROUTER_PORT));

    let mut stream = TcpStream::connect(socket_addr).unwrap();

    stream.write_all(&prepared_data).unwrap();
}

fn send_message_command(next_hop: &str, message: &str, shared_data: &Arc<Mutex<SharedData>>) {
    let address = next_hop.parse::<Ipv4Addr>().unwrap();

    send_message(message.as_bytes(), address.octets(), shared_data);
}

fn timed_get_dir(shared_data: &Arc<Mutex<SharedData>>) {
    let delta = Duration::from_secs(AUTO_REFRESH_RATE);
    let mut next_time = Instant::now() + delta;
    loop {
        {
            let mut node_list = shared_data.lock().unwrap();
            get_dir(&mut node_list.node_list);
        }
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

fn get_dir(node_list: &mut NodeList) {
    let resp = reqwest::blocking::get(format!("https://{CEPA_INDEX_HOST}:{CEPA_INDEX_PORT}/"))
        .unwrap()
        .json::<NodeList>()
        .unwrap();

    if resp.timestamp > node_list.timestamp {
        node_list.timestamp = resp.timestamp;
        node_list.list = resp.list;
    }
}

fn handle_connection(mut stream: TcpStream, shared_data: &Arc<Mutex<SharedData>>) {
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).unwrap();

    let (dest, decrypted) = unwrap_layer(&shared_data.lock().unwrap().priv_key, &buf);

    if dest[0] == 127 {
        shared_data.lock().unwrap().message_log.list.push(Message {
            timestamp_received: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message: (*String::from_utf8_lossy(&decrypted)).to_owned(),
        });
    } else {
        send_message(&decrypted, dest, shared_data);
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
fn handle_user_input(shared_data: &Arc<Mutex<SharedData>>) {
    let stdin = io::stdin();
    loop {
        let mut user_input = String::new();
        print!("cepa_router # ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut user_input).unwrap();

        let mut inputs_iterator = user_input.split_whitespace();

        let Some(command) = inputs_iterator.next() else {
            continue;
        };

        let parameters = inputs_iterator.collect::<Vec<&str>>();
        match command {
            "ls" => {
                let d = shared_data.lock().unwrap();
                let width = 24;
                println!("+-------------------------------------------------+");
                println!(
                    "| \x1b[1;94mTIMESTAMP:\x1b[0m {n:<width$}             |",
                    n = d.node_list.timestamp
                );
                println!("+------------------------+------------------------+");
                println!(
                        "|          \x1b[1;94mHOST\x1b[0m          |         \x1b[1;94mPUB_KEY\x1b[0m        |"
                    );
                println!("+------------------------+------------------------+");

                for node in &d.node_list.list {
                    println!("|{:width$}|{:width$}|", node.host, node.pub_key);
                }
                println!("+------------------------+------------------------+");
            }
            "lsd" => {
                println!("{:#?}", shared_data.lock().unwrap().node_list);
            }
            "get" => {
                get_dir(&mut shared_data.lock().unwrap().node_list);
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
                if parameters.len() == 2 {
                    let next_hop = parameters[0];
                    let message = parameters[1];
                    send_message_command(next_hop, message, shared_data);
                } else {
                    println!("Usage: send HOST MESSAGE");
                }
            }
            "add" => {
                if parameters.len() == 2 {
                    let host = parameters[0].to_owned();
                    let pub_key = parameters[1].to_owned();
                    add_host(&NodeData { host, pub_key });
                } else {
                    println!("Usage: add HOST PUB_KEY");
                }
            }
            "log" => {
                let l = &shared_data.lock().unwrap().message_log;
                for message in &l.list {
                    println!("[{}] {}", message.timestamp_received, message.message);
                }
            }
            "logd" => {
                let l = &shared_data.lock().unwrap().message_log;
                println!("{l:#?}");
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
                let l = &mut shared_data.lock().unwrap().message_log;
                l.list.clear();
            }
            _ => {
                println!("  Command \x1b[1;31m{command}\x1b[0m not found");
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

    let mut rng = rand::thread_rng();

    let shared_data = Arc::new(Mutex::new(SharedData {
        priv_key: RsaPrivateKey::new(&mut rng, RSA_KEY_BITS).expect("failed to generate a key"),
        node_list: NodeList::new(),
        message_log: MessageLog::new(),
    }));

    let data_clone = shared_data.clone();

    thread::spawn(move || {
        timed_get_dir(&data_clone);
    });

    let data_clone = shared_data.clone();
    thread::spawn(move || {
        handle_user_input(&data_clone);
    });

    match TcpListener::bind(format!("0.0.0.0:{CEPA_ROUTER_PORT}")) {
        Ok(listener) => {
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                let data_clone = shared_data.clone();
                thread::spawn(move || {
                    handle_connection(stream, &data_clone);
                });
            }
        }
        Err(err_msg) => {
            panic!("Could not bind on port {CEPA_ROUTER_PORT} : {err_msg}");
        }
    }
}
