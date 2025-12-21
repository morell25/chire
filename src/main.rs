use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

async fn process_socket(socket: TcpStream) {
    let (read_half, mut write_half) = socket.into_split();
    let mut buff: String = String::new();
    let mut reader: BufReader<tokio::net::tcp::OwnedReadHalf> = BufReader::new(read_half);
    let resulta: Result<usize, std::io::Error> = reader.read_line(&mut buff).await;

    if resulta.unwrap() > 0 {
        println!("Cliente conectado");
        match buff.trim() {
            "PING" => {
                let _ = write_half.write_all(b"PONG\n").await;
            }
            _ => {
                let _ = write_half.write_all(b"expected PING\n").await;
            }
        }
    } else {
        println!("Cliente desconectado")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await;
    }
}
