use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::types::EntryChire;

///Funcion para eliminar las entradas que no pueden ser usadas porque ya ha pasado la fecha expiracion
pub async fn cleaner_out_date(dic: &Arc<RwLock<HashMap<String, EntryChire>>>) {
    let dic_cli = dic;
    let mut expired: Vec<String> = Vec::new();
    let now = Instant::now();

    for (key, val) in dic_cli.read().await.iter() {
        if let Some(t) = val.expire_at {
            if t <= now {
                expired.push(key.clone())
            }
        }
    }

    if !expired.is_empty() {
        //Se saca el guard fuera del for porque no tiene sentido bloquear en cada iteración
        //Aqui se podria volver a comprobar si junto entre alguien ha actualizado su hora de borrado
        //Pero es poco probleba por lo que de momento no lo voy a hacer
        let mut cli_guard = dic_cli.write().await;
        for x in expired {
            let _ = cli_guard.remove(&x);
        }
    }
}
