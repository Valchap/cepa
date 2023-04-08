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

fn forward_message(next_hop: String, message: String) {
    let mut n_stream = TcpStream::connect(format!("{}:55505", next_hop)).unwrap();
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

fn get_dir(data: Arc<Mutex<NodeList>>) {
    if let Ok(mut n_stream) = TcpStream::connect("127.0.0.1:8000") {
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
                        println!("hey");
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
    \x1b[1;94mdir\x1b[0m      Print the local list of announced cepa nodes
    \x1b[1;94mget\x1b[0m      Ask the cepa_index for the list of nodes
    \x1b[1;94mclear\x1b[0m    Clear screen
    \x1b[1;94mexit\x1b[0m     Exit the cepa_router process
    \x1b[1;94mhelp\x1b[0m     Print this help"
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
            match user_input.as_str() {
                "dir\n" => {
                    println!("{:#?}", data.lock().unwrap())
                }
                "get\n" => {
                    get_dir(data.clone());
                }
                "clear\n" => {
                    print!("\x1B[2J\x1B[1;1H");
                }
                "exit\n" => {
                    std::process::exit(0);
                }
                "help\n" => {
                    print_help();
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
    match TcpListener::bind("0.0.0.0:55505") {
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
            panic!("Could not bind on port 55505");
        }
    }
}
