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

fn get_directory(data: Arc<Mutex<NodeList>>) {
    let delta = Duration::from_secs(5);
    let mut next_time = Instant::now() + delta;
    loop {
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
                                    d.timestamp = t.timestamp;
                                    d.list = t.list;
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

        sleep(next_time - Instant::now());
        next_time += delta;
    }
}

fn handle_connection(mut stream: TcpStream) {
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

// Handle user input from stdin
fn handle_user_input(data: Arc<Mutex<NodeList>>) {
    let mut user_input: String;
    let stdin = io::stdin();
    loop {
        user_input = "".to_string();
        stdin.read_line(&mut user_input).unwrap();
        if !user_input.is_empty() {
            match user_input.as_str() {
                "dir\n" => {
                    println!("{:?}", data.lock())
                }
                _ => {
                    println!("{}", user_input);
                }
            }
        }
    }
}

fn main() {
    match TcpListener::bind("0.0.0.0:55505") {
        Ok(listener) => {
            // println!("Listening on 55505");

            let data: Arc<Mutex<NodeList>> = Arc::new(Mutex::new(NodeList {
                timestamp: 0,
                list: Vec::new(),
            }));

            let mut data_clone = data.clone();

            thread::spawn(move || {
                get_directory(data_clone);
            });

            data_clone = data.clone();
            thread::spawn(move || {
                handle_user_input(data_clone);
            });

            // for stream in listener.incoming() {
            //     let stream = stream.unwrap();

            //     // TODO Fix -> Can spawn an infinite number of threads !!!!
            //     thread::spawn(|| {
            //         handle_connection(stream);
            //     });
            // }
        }
        Err(_) => {
            panic!("Could not bind on port 55505");
        }
    }
}
