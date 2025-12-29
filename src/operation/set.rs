/*
pub async fn set(rest: &str) {
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
        let entry_chire = Entry_chire::create(value);
        map.insert(key, entry_chire.clone());
        drop(map);
        let _ = aof::append_result(entry_chire, "db_main".to_string()).await;
    }

    let result = format!("Ok, insetado correctametne\n");
    let _ = write_half.write_all(result.as_bytes()).await;
}
    */
