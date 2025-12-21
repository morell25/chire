use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

async fn process_socket(socket: TcpStream) {
    let (read_half, mut write_half) = socket.into_split();
    let mut buff: String = String::new();
    let mut reader: BufReader<tokio::net::tcp::OwnedReadHalf> = BufReader::new(read_half);
    println!("Cliente conectado");
    loop {
        buff.clear();
        let res: Result<usize, std::io::Error> = reader.read_line(&mut buff).await;
        match res {
            Ok(0) => {
                println!("clientes desconectado");
                break;
            }
            Ok(_n) => match buff.trim() {
                "PING" => {
                    let _ = write_half.write_all(b"PONG\n").await;
                }
                "echo" => {
                    let _ = write_half.write_all(b"hola mundo\n").await;
                }
                _ => {
                    let _ = write_half.write_all(b"Err unknow command\n").await;
                }
            },
            Err(_) => {
                println!("Error"); //Futuro log
                break;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(async move { process_socket(socket).await });
    }
}
