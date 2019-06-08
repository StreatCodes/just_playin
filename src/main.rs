use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::path::Path;
use std::mem::size_of;

use base64;
use sodiumoxide::crypto::secretstream;

fn load_key() -> Option<secretstream::Key> {
    let exists = Path::exists(Path::new("./priv.key"));
    if!exists {
        return None;
    }

    let encoded_key = fs::read_to_string("./priv.key")
        .expect("Couldn't read priv.key");
    let key = base64::decode(encoded_key.as_bytes())
        .expect("Couldn't decode key file");

    secretstream::Key::from_slice(&key)
}

fn handle_client(mut stream: TcpStream, key: &secretstream::Key) {
    let mut reader = BufReader::new(&stream);
    let writer = BufReader::new(&stream);

    let mut head_buf = vec![0; secretstream::HEADERBYTES];
    reader.read_exact(&mut head_buf)
        .expect("Unexpected header message");
    println!("Read header");
    let header = secretstream::Header::from_slice(&head_buf)
        .expect("Couldn't initialize decryption header");

    let mut dec_stream = secretstream::Stream::init_pull(&header, &key)
        .expect("Couldn't initialize decryption stream");

    let mut count = 0;
    'loopy: loop {
        count += 1;
        let mut count_buf = [0; size_of::<usize>()];
        if let Err(e) = reader.read_exact(&mut count_buf) {
            println!("Received invalid message length, Error: {} Disconnecting client", e);
            println!("Received {} messages", count);
            break 'loopy;
        }

        let count = usize::from_be_bytes(count_buf);
        // println!("Reading: {}bytes", count);

        let mut buf = vec![0; usize::from(count)];
        if let Err(_) = reader.read_exact(&mut buf) {
            println!("Received invalid message, Disconnecting client");
            break 'loopy;
        }

        match dec_stream.pull(&buf, None) {
            Ok((decrypted_message, tag)) => {
                let message = String::from_utf8(decrypted_message)
                    .expect("Error extracting text from message");
                // println!("Read {}", message);
            },
            Err(()) => {
                println!("Couldn't validate and decrypt message, Disconnecting client");
                break 'loopy;
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("localhost:3000").expect("Couldn't bind to 3000");
    let key = load_key()
        .expect("Couldn't load encryption key, exiting");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection!");
                handle_client(stream, &key);
            }
            Err(e) => {
                println!("Connection error: {}", e);
            }
        }
    }
}
