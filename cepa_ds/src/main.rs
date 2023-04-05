use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

use serde;
use serde_json;

fn handle_connection(mut stream: TcpStream, dir: &Arc<Mutex<Vec<String>>>) {
    // Read from tcp stream
    let mut buf: String = "".to_string();
    stream.read_to_string(&mut buf).unwrap();

    println!("received: {}", buf);

    // let mut d = Arc::clone(&dir);
    // let (tx, rx) = Arc::channel();
    // let (data, tx) = (Arc::clone(&dir), tx.clone());

    if buf.starts_with("PUT") {
        let w: Vec<&str> = buf.split_whitespace().collect();
        let host = w[1];
        let msg: Vec<&str> = w[2..].to_vec();
        let message = msg.join(" ");

        println!("host {}", host);
        println!("message {}", message);

        let push_string: String = format!("{} {}", host, message);

        dir.lock().unwrap().push(push_string);
        println!("Added {} {}", host, message);

        // forward_message(next_hop, message);
    } else if buf.starts_with("GET") {
        let tx_txt = dir.lock().unwrap().clone();
        println!("{:?}", tx_txt);

        stream
            .write(serde_json::to_string(&tx_txt).unwrap().as_bytes())
            .unwrap();
    }
}

fn main() {
    // let mut dir: HashMap<String, String> = HashMap::new();
    let dir = Arc::new(Mutex::new(Vec::new()));

    let listener = TcpListener::bind("0.0.0.0:8080").unwrap();
    println!("Listening on 8080");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        let new_dir = Arc::clone(&dir);
        thread::spawn(move || {
            handle_connection(stream, &new_dir);
        });
    }
}
