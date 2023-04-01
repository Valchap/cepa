use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};

fn forward_message(next_hop: String, message: String) {
    let mut n_stream = TcpStream::connect(format!("{}:55505", next_hop)).unwrap();
    n_stream.write(&message.into_bytes()).unwrap();
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
fn handle_user_input() {
    let mut user_input: String;
    let stdin = io::stdin();
    loop {
        user_input = "".to_string();
        stdin.read_line(&mut user_input).unwrap();
        if !user_input.is_empty() {
            println!("{}", user_input);
        }
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:55505").unwrap();
    println!("Listening on 55505");

    thread::spawn(|| {
        handle_user_input();
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        // TODO Fix -> Can spawn an infinite number of threads !!!!
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}
