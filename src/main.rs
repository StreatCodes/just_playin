use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

fn handle_client(mut stream: TcpStream) {
    let mut buf = vec![0; 1048576];
    let mut byte_count: usize = 0;
    let mut print_at = Instant::now() + Duration::from_secs(1);

    loop {
        let count = stream.read_exact(&mut buf);
        match count {
            Ok(()) => {
                byte_count += 1048576;
                if Instant::now() > print_at {
                    println!("{}MB/s", byte_count / 1024 / 1024);
                    byte_count = 0;
                    print_at = Instant::now() + Duration::from_secs(1);
                }
            }, Err(e) => {
                println!("Error reading from socket: {}", e)
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("192.168.20.4:3000").expect("Couldn't bind to 3000");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection!");
                handle_client(stream);
            }
            Err(e) => { /* connection failed */ }
        }
    }
}
