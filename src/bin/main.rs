use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::thread;
use mini_httpd::ThreadPool;
use std::time::Duration;


fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    //println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";
    let (status_line, filename) = if buffer.starts_with(get){
        thread::sleep(Duration::from_secs(2));
        ("HTTP/1.1 200 OK \r\n\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep){
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK \r\n\r\n\r\n", "hello.html")
    } else {
        thread::sleep(Duration::from_secs(10));
        ("HTTP/1.1 404 NOT FOUND \r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let response = format!("{}{}", status_line, contents);
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    let pool = ThreadPool::new(4);
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        //spawn thread for each request
        pool.execute(|| {
            handle_connection(stream);
        });
    }
    //shutting down is just for demo purpose
    println!("Shutting down.");

}
