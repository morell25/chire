use std::{collections::HashMap, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time::Instant,
};

#[derive(Debug, Clone)]
struct Entry {
    data: String,
    expire_at: Option<Instant>,
}

impl Entry {
    fn create(value: String) -> Self {
        Self {
            data: value,
            expire_at: None,
        }
    }
    fn set_expiration(&mut self, expire_at: Instant) {
        self.expire_at = Some(expire_at);
    }
}

async fn cleaner(dic: &Arc<RwLock<HashMap<String, Entry>>>) {
    let dic_cli = dic;
    let mut expired: Vec<String> = Vec::new();
    let now = tokio::time::Instant::now();

    for (key, val) in dic_cli.read().await.iter() {
        if let Some(t) = val.expire_at {
            if t >= now {
                expired.push(key.to_owned())
            }
        }
    }

    if !expired.is_empty() {
        {
            for x in expired {
                let mut cli_guard = dic_cli.write().await;
                let _ = cli_guard.remove(&x);
            }
        }
    }
}

async fn process_socket(socket: TcpStream, dic: Arc<RwLock<HashMap<String, Entry>>>) {
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
                            map.insert(key, Entry::create(value));
                        }

                        let result = format!("Ok, insetado correctametne\n");
                        let _ = write_half.write_all(result.as_bytes()).await;
                    }
                    "expire" => {
                        let mut string_white = rest.split_whitespace();
                        let key = string_white.next();
                        let value = string_white.next();

                        if key.is_none() || value.is_some() {
                            let _ = write_half.write_all(b"Error en los parametros\n").await;
                        }

                        {
                            let mut guard = dic_cliente.write().await;
                            if let Some(u) = guard.get_mut(key.expect("")) {
                                u.set_expiration(
                                    tokio::time::Instant::now()
                                        + tokio::time::Duration::from_secs(20),
                                )
                            }
                        };
                        let _ = write_half.write_all(b"contadoro fijado\n").await;
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
                            Some(v) => format!("objeto encontrado {:?}\n", v),
                            None => {
                                format!("objeto no encontrado, parametro usado: {:?}\n", extra_key)
                            }
                        };
                        let _ = write_half.write_all(response.as_bytes()).await;
                    }
                    "getall" => {
                        for (ke, val) in dic_cliente.read().await.iter() {
                            let result = format!("Objeto -> key: {}, value: {:?}\n", ke, val);
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
    let dic_compa: Arc<RwLock<HashMap<String, Entry>>> = Arc::new(RwLock::new(HashMap::new()));

    {
        let dic = dic_compa.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(20));
                loop {
                    interval.tick().await;
                    cleaner(&dic).await;
                    println!("limpieza terminada\n");
                }
        });
    }

    loop {
        let (socket, _) = listener.accept().await?;
        let dic_clo = dic_compa.clone();
        tokio::spawn(async move { process_socket(socket, dic_clo).await });
    }
}
