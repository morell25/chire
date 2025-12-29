//aof
mod aof;
mod cleaner;
mod operation;

//Types
mod types;
use crate::types::EntryChire;

//crates externos
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::RwLock,
    time,
    time::Duration,
    time::Instant,
};

async fn process_socket(socket: TcpStream, dic: Arc<RwLock<HashMap<String, EntryChire>>>) {
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
                        //operation::set(rest);
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
                            let entry_chire = EntryChire::create(value);
                            map.insert(key, entry_chire.clone());
                            drop(map);
                            let _ = aof::append_result(entry_chire, "db_main".to_string()).await;
                        }

                        let result = "Ok, insetado correctametne\n";
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
                                u.set_expiration(Instant::now() + Duration::from_secs(20))
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
                            Some(_v) => "1\n",
                            None => "0\n",
                        };
                        let _ = write_half.write_all(result.as_bytes()).await;
                    }
                    "ping" => {
                        let result = aof::create_db().await;
                        let pri = format!("PONG, var: {:?}\n", result);
                        let _ = write_half.write_all(pri.as_bytes()).await;
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

pub async fn run_server(name_port: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(name_port).await?;
    let dic_compa: Arc<RwLock<HashMap<String, EntryChire>>> =
        Arc::new(RwLock::new(HashMap::new()));

    {
        let dic = dic_compa.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(20));
            loop {
                interval.tick().await;
                cleaner::cleaner_out_date(&dic).await;
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

#[cfg(test)]
mod tests {
    use super::{cleaner, types::EntryChire};
    use std::{collections::HashMap, sync::Arc};
    use tokio::{
        sync::RwLock,
        time::{Duration, Instant},
    };

    #[test]
    fn entry_chire_create_sets_fields() {
        let entry = EntryChire::create("valor".to_string());
        assert_eq!(entry.data, "valor");
        assert!(entry.expire_at.is_none());
    }

    #[tokio::test]
    async fn cleaner_removes_expired_entries() {
        let dic: Arc<RwLock<HashMap<String, EntryChire>>> =
            Arc::new(RwLock::new(HashMap::new()));

        let mut expired = EntryChire::create("expirado".to_string());
        expired.set_expiration(Instant::now() - Duration::from_secs(1));

        let mut active = EntryChire::create("activo".to_string());
        active.set_expiration(Instant::now() + Duration::from_secs(60));

        {
            let mut guard = dic.write().await;
            guard.insert("exp".to_string(), expired);
            guard.insert("act".to_string(), active);
        }

        cleaner::cleaner_out_date(&dic).await;

        let guard = dic.read().await;
        assert!(!guard.contains_key("exp"));
        assert!(guard.contains_key("act"));
    }
}
