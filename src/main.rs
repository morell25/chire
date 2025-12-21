use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

async fn process_socket(socket: TcpStream, dic: Arc<RwLock<HashMap<String, String>>>) {
    let (read_half, mut write_half) = socket.into_split();
    let mut buff = String::new();
    let mut reader = BufReader::new(read_half);
    let dic_cliente = dic;
    println!("Cliente conectado");
    loop {
        buff.clear();
        let res = reader.read_line(&mut buff).await;

        match res {
            Ok(0) => {
                println!("clientes desconectado");
                break;
            }
            Ok(_n) => {
                let line = buff.trim().to_lowercase();
                let (cmd, rest) = line.split_once(' ').unwrap_or((&line, ""));

                match cmd {
                    "set" => {
                        let (key, value) = match rest.split_once(' ') {
                            Some((k, v)) if !k.trim().is_empty() && !v.trim_start().is_empty() => {
                                (k.trim().to_string(), v.trim_start().to_string())
                            }
                            _ => {
                                let _ = write_half.write_all(b"Error en los parametros \n").await;
                                continue;
                            }
                        };
                        {
                            let mut map = dic_cliente.write().await;
                            map.insert(key, value);
                        }

                        let result = format!("Ok, insetado correctametne\n");
                        let _ = write_half.write_all(result.as_bytes()).await;
                    }
                    "get" => {
                        let mut it = rest.split_whitespace();
                        let key = it.next();
                        let extra_key = it.next();

                        if key.is_none() || extra_key.is_some() {
                            let _ = write_half.write_all(b"parametros errones \n").await;
                            continue;
                        }

                        let map_guard = {
                            let guard = dic_cliente.read().await;
                            guard.get(key.expect("")).cloned()
                        };

                        let response = match map_guard {
                            Some(v) => format!("objeto encontrado: {v}\n"),
                            None => {
                                format!("objeto no encontrado, parametro usado: {:?}\n", extra_key)
                            }
                        };
                        let _ = write_half.write_all(response.as_bytes()).await;
                    }
                    "getall" => {
                        for (ke, val) in dic_cliente.read().await.iter() {
                            let result = format!("Objeto -> key: {}, value: {}\n", ke, val);
                            let _ = write_half.write_all(result.as_bytes()).await;
                        }
                    }
                    "del" => {
                        let mut it = rest.split_whitespace();
                        let key = it.next();
                        let extra_key = it.next();

                        if key.is_none() || extra_key.is_some() {
                            let _ = write_half.write_all(b"parametros errones \n").await;
                            continue;
                        }
                        let h = { dic_cliente.write().await.remove(key.expect("")) };

                        let result = match h {
                            Some(_v) => format!("1\n"),
                            None => format!("0\n"),
                        };
                        let _ = write_half.write_all(result.as_bytes()).await;
                    }
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
