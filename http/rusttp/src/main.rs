use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

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
                eprintln!("Failed to read from stream : {}", e);
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

    let response = match first_line {
        Some("GET /test HTTP/1.1") => get_test_handler(),
        Some("GET /everything HTTP/1.1") => get_everything_handler(),
        _ => get_404(),
    };

    if let Err(e) = response {
        eprintln!("Error : {}", e);
        let error_message = b"HTTP/1.1 500 Internal Server Error\r\n\r\n";
        let _ = stream.write_all(error_message);
    } else {
        let mut file = response.unwrap();
        let mut file_contents = Vec::new();
        if let Err(e) = file.read_to_end(&mut file_contents) {
            eprintln!("Failed to read file : {}", e);
            let error_message = b"HTTP/1.1 500 Internal Server Error\r\n\r\n";
            let _ = stream.write_all(error_message);
            return;
        }

        let response_header = b"HTTP/1.1 200 OK \r\n\r\n";
        if let Err(e) = stream.write_all(response_header) {
            eprintln!("Failed to write response header: {}", e);
            return;
        }
        if let Err(e) = stream.write_all(&file_contents) {
            eprintln!("Failed to write response body : {} ", e);
        }
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        handle_client(stream?)
    }
    Ok(())
}

fn get_test_handler() -> Result<File, std::io::Error> {
    File::open("public/index.html")
}

fn get_everything_handler() -> Result<File, std::io::Error> {
    File::open("public/page.html")
}

fn get_404() -> Result<File, std::io::Error> {
    File::open("public/404.html")
}
