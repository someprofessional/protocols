use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use sha1::{Sha1, Digest};
use base64::encode;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = Vec::new();

    loop {
        let mut chunk = [0; 1024];
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                buffer.extend_from_slice(&chunk[..n]);
                if n < 1024 {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to read from stream: {}", e);
                return;
            }
        }
    }
    request_matcher(&buffer, stream);
}

fn request_matcher(buffer: &[u8], mut stream: TcpStream) {
    let request_str = String::from_utf8_lossy(buffer);
    let mut lines = request_str.lines();
    let first_line = lines.next();

    if first_line == Some("GET /websocketme HTTP/1.1") {
        if let Some(websocket_key_line) = lines.find(|line: &&str| line.starts_with("Sec-WebSocket-Key:")) {
            let websocket_key = websocket_key_line.split_whitespace().nth(1).unwrap();
            handle_websocket_handshake(stream, websocket_key);
        }
    } else {
        let response = match first_line {
            Some("GET / HTTP/1.1") => get_index(),
            Some("GET /about HTTP/1.1") => get_about(),
            _ => get_404(),
        };

        if let Err(e) = response {
            eprintln!("Error: {}", e);
            let error_message = b"HTTP/1.1 500 Internal Server Error\r\n\r\n";
            let _ = stream.write_all(error_message);
        } else {
            let mut file = response.unwrap();
            let mut file_contents = Vec::new();
            if let Err(e) = file.read_to_end(&mut file_contents) {
                eprintln!("Failed to read file: {}", e);
                let error_message = b"HTTP/1.1 500 Internal Error\r\n\r\n";
                let _ = stream.write_all(error_message);
                return;
            }

            let response_header = b"HTTP/1.1 200 OK \r\n\r\n";
            if let Err(e) = stream.write_all(response_header) {
                eprintln!("Failed to write response header: {}", e);
            }
            if let Err(e) = stream.write_all(&file_contents) {
                eprintln!("Failed to write response body: {}", e);
            }
        }
    }
}

fn handle_websocket_handshake(mut stream: TcpStream, websocket_key: &str) {
    let accept_key = generate_websocket_accept_key(websocket_key);

    // Send the WebSocket handshake response
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Accept: {}\r\n\r\n",
        accept_key
    );
    stream.write_all(response.as_bytes()).unwrap();

    // After handshake, enter WebSocket communication loop
    handle_websocket_communication(stream);
}

fn generate_websocket_accept_key(websocket_key: &str) -> String {
    let magic_string = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha1::new();
    hasher.update(websocket_key.as_bytes());
    hasher.update(magic_string.as_bytes());
    let result = hasher.finalize();
    encode(result)
}

fn handle_websocket_communication(mut stream: TcpStream) {
    loop {
        let mut buffer = [0u8; 2]; // Read the first 2 bytes of the WebSocket frame
        if let Err(e) = stream.read_exact(&mut buffer) {
            eprintln!("Failed to read from WebSocket stream: {}", e);
            return;
        }

        let _fin_and_opcode = buffer[0];
        let masked = buffer[1] & 0b10000000 != 0;
        let mut payload_len = buffer[1] & 0b01111111;

        if !masked {
            eprintln!("Invalid WebSocket frame: client-to-server frames must be masked.");
            return;
        }

        // Determine the payload length based on the WebSocket frame encoding
        if payload_len == 126 {
            // Payload length is represented by the next 2 bytes
            let mut extended_len = [0u8; 2];
            if let Err(e) = stream.read_exact(&mut extended_len) {
                eprintln!("Failed to read extended payload length: {}", e);
                return;
            }
            payload_len = u16::from_be_bytes(extended_len) as u8;
        } else if payload_len == 127 {
            // Payload length is represented by the next 8 bytes
            let mut extended_len = [0u8; 8];
            if let Err(e) = stream.read_exact(&mut extended_len) {
                eprintln!("Failed to read extended payload length: {}", e);
                return;
            }
            payload_len = u64::from_be_bytes(extended_len) as u8;
        }

        // Read the masking key (4 bytes) and the payload data
        let mut masking_key = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut masking_key) {
            eprintln!("Failed to read masking key: {}", e);
            return;
        }

        let mut payload = vec![0u8; payload_len as usize];
        if let Err(e) = stream.read_exact(&mut payload) {
            eprintln!("Failed to read payload data: {}", e);
            return;
        }

        // Unmask the payload
        for i in 0..payload.len() {
            payload[i] ^= masking_key[i % 4];
        }

        // Print the received message
        let message = String::from_utf8_lossy(&payload);
        println!("Received WebSocket message: {}", message);

        // You can process the message and send a response
        // For this example, we'll just send back the same message as an echo
        send_websocket_message(&mut stream, &message);
    }
}

fn send_websocket_message(stream: &mut TcpStream, message: &str) {
    let message_bytes = message.as_bytes();
    let message_len = message_bytes.len();

    let mut frame = Vec::new();
    frame.push(0b10000001); // FIN + Text frame (opcode 0x1)

    if message_len <= 125 {
        frame.push(message_len as u8);
    } else if message_len <= 65535 {
        frame.push(126);
        frame.extend_from_slice(&(message_len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(message_len as u64).to_be_bytes());
    }

    frame.extend_from_slice(message_bytes);

    if let Err(e) = stream.write_all(&frame) {
        eprintln!("Failed to send WebSocket message: {}", e);
    }
}

fn main() -> std::io::Result<()> {
    println!("Hello, WebSocket!");

    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}

// Here are the handlers
fn get_index() -> Result<File, std::io::Error> {
    File::open("public/index.html")
}

fn get_about() -> Result<File, std::io::Error> {
    File::open("public/about.html")
}

fn get_404() -> Result<File, std::io::Error> {
    File::open("public/404.html")
}
