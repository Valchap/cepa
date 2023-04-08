use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use cepa_common::{NodeData, NodeList, NodeListPointer};

use serde_json;

use std::thread::sleep;
use std::time::{Duration, Instant};

const CEPA_INDEX_HOST: &str = "127.0.0.1";
const CEPA_INDEX_PORT: &str = "8000";
const CEPA_INDEX_PUB_KEY: &str = "keyhere";

const CEPA_ROUTER_PORT: &str = "55505";

fn forward_message(next_hop: String, message: String) {
    let mut n_stream = TcpStream::connect(format!("{}:{}", next_hop, CEPA_ROUTER_PORT)).unwrap();
    n_stream.write(&message.into_bytes()).unwrap();
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

fn add_host(d: NodeData) {
    if let Ok(mut n_stream) = TcpStream::connect(format!("{}:{}", CEPA_INDEX_HOST, CEPA_INDEX_PORT))
    {
        let req = format!(
            "
POST / HTTP/1.1
Content-Length: {}
Content-Type: application/json
Connection: Close

{{\"host\": \"{}\", \"pub_key\": \"{}\"}} 
        ",
            (d.host.as_bytes().len() + d.pub_key.as_bytes().len() + 27),
            d.host,
            d.pub_key
        );
        match n_stream.write(req.as_bytes()) {
            Ok(_) => {
                let mut buf = String::new();
                match n_stream.read_to_string(&mut buf) {
                    Ok(_) => {
                        println!("{}", buf);
                    }
                    Err(_) => {
                        panic!("Could not read from stream")
                    }
                }
            }
            Err(_) => {
                panic!("Could not send request to cepa_index")
            }
        }
    }
}

fn get_dir(data: Arc<Mutex<NodeList>>) {
    if let Ok(mut n_stream) = TcpStream::connect(format!("{}:{}", CEPA_INDEX_HOST, CEPA_INDEX_PORT))
    {
        match n_stream.write("GET / HTTP/1.1\nConnection: Close\n\n".as_bytes()) {
            Ok(_) => {
                let mut dir_string = String::new();
                match n_stream.read_to_string(&mut dir_string) {
                    Ok(_) => {
                        let json_txt = dir_string.split("\n").last().unwrap();
                        let t: NodeList = serde_json::from_str(json_txt).unwrap();
                        match data.lock() {
                            Ok(mut d) => {
                                if t.timestamp > d.timestamp {
                                    d.timestamp = t.timestamp;
                                    d.list = t.list;
                                }
                            }
                            Err(_) => {
                                panic!("Could not access data")
                            }
                        }
                    }
                    Err(_) => {
                        panic!("Can't read from tcp stream")
                    }
                }
            }
            Err(_) => {
                panic!("Could not send request")
            }
        }
    } else {
        panic!("Could not connect to index")
    };
}

fn handle_connection(mut stream: TcpStream, data: Arc<Mutex<NodeList>>) {
    // stream.write(b"connection handled").unwrap();

    // Read from tcp stream
    let mut buf: String = "".to_string();
    stream.read_to_string(&mut buf).unwrap();

    println!("received: {}", buf);

    // Check if the message needs to be forwarded
    if buf.starts_with("sendto:") {
        let w: Vec<&str> = buf.split_whitespace().collect();
        let next_hop = w[1];
        let msg: Vec<&str> = w[2..].to_vec();
        let message = msg.join(" ");
        forward_message(next_hop.to_string(), message);

        // forward_message(next_hop, message);
    } else {
        println!("{}", buf);
    }
}

fn print_help() {
    println!(
        "\x1b[1mCommands:\x1b[0m
    \x1b[1;94mdir\x1b[0m                Print the local list of announced cepa nodes
    \x1b[1;94mget\x1b[0m                Ask the cepa_index for the list of nodes
    \x1b[1;94msend\x1b[0m HOST MSG      Send message to host
    \x1b[1;94madd\x1b[0m  HOST PUB_KEY  Add a node to cepa_index
    \x1b[1;94mclear\x1b[0m              Clear screen
    \x1b[1;94mexit\x1b[0m               Exit the cepa_router process
    \x1b[1;94mhelp\x1b[0m               Print this help"
    );
}

// Handle user input from stdin
fn handle_user_input(data: Arc<Mutex<NodeList>>) {
    let mut user_input: String;
    let stdin = io::stdin();
    loop {
        user_input = "".to_string();
        print!("cepa_router # ");
        let _ = io::stdout().flush();
        stdin.read_line(&mut user_input).unwrap();
        if !user_input.is_empty() {
            match user_input
                .as_str()
                .split_whitespace()
                .into_iter()
                .nth(0)
                .unwrap_or("\n")
            {
                "dir" => {
                    println!("{:#?}", data.lock().unwrap())
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
                    if user_input.as_str().split_whitespace().count() != 3 {
                        println!("Usage: send HOST MESSAGE");
                    } else {
                        let next_hop = user_input
                            .as_str()
                            .split_whitespace()
                            .into_iter()
                            .nth(1)
                            .unwrap()
                            .to_string();
                        let message = user_input
                            .as_str()
                            .split_whitespace()
                            .into_iter()
                            .nth(2)
                            .unwrap()
                            .to_string();
                        forward_message(next_hop, message);
                    }
                }
                "add" => {
                    if user_input.as_str().split_whitespace().count() != 3 {
                        println!("Usage: add HOST PUB_KEY");
                    } else {
                        let host = user_input
                            .as_str()
                            .split_whitespace()
                            .into_iter()
                            .nth(1)
                            .unwrap()
                            .to_string();
                        let pub_key = user_input
                            .as_str()
                            .split_whitespace()
                            .into_iter()
                            .nth(2)
                            .unwrap()
                            .to_string();
                        add_host(NodeData { host, pub_key });
                    }
                }
                _ => {
                    if user_input != "\n" {
                        user_input.pop();
                        println!("  Command \x1b[1;31m{}\x1b[0m not found", user_input);
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
            // println!("Listening on 55505");

            let data: Arc<Mutex<NodeList>> = Arc::new(Mutex::new(NodeList {
                timestamp: 0,
                list: Vec::new(),
            }));

            let mut data_clone = data.clone();

            thread::spawn(move || {
                timed_get_dir(data_clone);
            });

            data_clone = data.clone();
            thread::spawn(move || {
                handle_user_input(data_clone);
            });

            for stream in listener.incoming() {
                let stream = stream.unwrap();
                data_clone = data.clone();
                thread::spawn(|| {
                    handle_connection(stream, data_clone);
                });
            }
        }
        Err(_) => {
            panic!("Could not bind on port {}", CEPA_ROUTER_PORT);
        }
    }
}
