use std::{collections::HashMap, fmt::format, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

async fn process_socket(socket: TcpStream, dic: Arc<RwLock<HashMap<String, String>>>) {
    let (read_half, mut write_half) = socket.into_split();
    let mut buff: String = String::new();
    let mut reader: BufReader<tokio::net::tcp::OwnedReadHalf> = BufReader::new(read_half);
    let dic_cliente: Arc<RwLock<HashMap<String, String>>> = dic;
    println!("Cliente conectado");
    loop {
        buff.clear();
        let res: Result<usize, std::io::Error> = reader.read_line(&mut buff).await;

        match res {
            Ok(0) => {
                println!("clientes desconectado");
                break;
            }
            Ok(_n) => {
                let line: String = buff.trim().to_lowercase();
                let (cmd, rest) = line.split_once(' ').unwrap_or((&line, ""));
                match cmd {
                    "set" => {
                        let (dic_key, dic_value) = rest.split_once(' ').unwrap();
                        let mut map: tokio::sync::RwLockWriteGuard<'_, HashMap<String, String>> =
                            dic_cliente.write().await;
                        map.insert(dic_key.trim().to_string(), dic_value.trim().to_string());
                        let result = format!(
                            "objetos insertados key: {}, value: {}\n",
                            dic_key, dic_value
                        );
                        let _ = write_half.write_all(result.as_bytes()).await;
                    }
                    "get" => {
                        let key: &str = rest.trim();
                        if key.is_empty() {
                            let _ = write_half
                                .write_all(b"ERR wrong number of arguments\n")
                                .await;
                            break;
                        }
                        let map_guard: Option<String> = {
                            let guard: tokio::sync::RwLockReadGuard<'_, HashMap<String, String>> =
                                dic_cliente.read().await;
                            guard.get(key).cloned()
                        };
                        let response: String = match map_guard {
                            Some(v) => format!("objeto encontrado: {v}\n"),
                            None => format!("objeto no encontrado, parametro usado: {key}\n"),
                        };
                        let _ = write_half.write_all(response.as_bytes()).await;
                    }
                    "getall" => {
                        for (ke, val) in dic_cliente.read().await.iter() {
                            let result: String = format!("Objeto -> key: {}, value: {}\n", ke, val);
                            let _ = write_half.write_all(result.as_bytes()).await;
                        }
                    }
                    "del" => {}
                    "ping" => {
                        let _ = write_half.write_all(b"PONG\n").await;
                    }
                    "echo" => {
                        if rest.is_empty() {
                            let _ = write_half.write_all(b"echo sin parametros\n").await;
                        }
                        let _ = write_half.write_all(rest.as_bytes()).await;
                    }
                    "help" => {
                        let _ = write_half
                            .write_all(b"Commands: PING, ECHO, HELP, QUIT\n")
                            .await;
                    }
                    "quit" => {
                        let _ = write_half.write_all(b"BYE\n").await;
                        break;
                    }

                    _ => {
                        let _ = write_half.write_all(b"ERR unknown command\n").await;
                    }
                }
            }
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
    let dic_compa: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));

    loop {
        let (socket, _) = listener.accept().await?;
        let dic_clo = dic_compa.clone();
        tokio::spawn(async move { process_socket(socket, dic_clo).await });
    }
}
