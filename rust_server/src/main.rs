use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

// Define our database type: A Thread-Safe Hash Map
type Db = Arc<Mutex<HashMap<String, String>>>;

// This function processes an individual client's connection
fn handle_client(mut stream: TcpStream, db: Db) {
    let mut buffer = [0; 512];
    
    if let Ok(bytes_read) = stream.read(&mut buffer) {
        if bytes_read == 0 { return; }
        
        let request = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
        let parts: Vec<&str> = request.trim().split_whitespace().collect();
        
        if parts.is_empty() { return; }

        let command = parts[0].to_uppercase();
        let mut response = String::new();

        // Parse the database commands
        match command.as_str() {
            "SET" => {
                if parts.len() >= 3 {
                    let key = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    
                    // Lock the database, insert the data, and automatically unlock
                    let mut map = db.lock().unwrap();
                    map.insert(key, value);
                    response = "OK\n".to_string();
                } else {
                    response = "ERROR: SET requires a key and a value\n".to_string();
                }
            }
            "GET" => {
                if parts.len() == 2 {
                    let key = parts[1];
                    let map = db.lock().unwrap();
                    
                    if let Some(value) = map.get(key) {
                        response = format!("{}\n", value);
                    } else {
                        response = "(nil)\n".to_string();
                    }
                } else {
                    response = "ERROR: GET requires a key\n".to_string();
                }
            }
            _ => {
                response = format!("ERROR: Unknown command '{}'\n", command);
            }
        }
        
        let _ = stream.write(response.as_bytes());
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").expect("Failed to bind");
    
    // Initialize the empty database
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    println!("Mini-Redis Database running on 127.0.0.1:6379...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Clone the reference pointer to the database
                let db_clone = Arc::clone(&db);
                
                // Spawn a new background thread for this specific network request
                thread::spawn(move || {
                    handle_client(stream, db_clone);
                });
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
}