use tokio::fs::{self, File};
//Fichero destinado a append only file (aof)
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tracing::{Level, event};

use crate::types::EntryChire;

trait EntryOperation {
    fn operation(&self) {}
}

impl EntryOperation for EntryChire {
    fn operation(&self) {}
}

//Function para crear la carpeta donde se almacenan los fichero de la bd
//NO es publica porque se llama desde crear fichero/db
async fn create_folder() -> u8 {
    let path_folder: &str = "./db";
    let result_path_create = fs::create_dir(path_folder).await;
    match result_path_create {
        Ok(n) => {
            event!(Level::INFO, "created folder {:?}", n);
            1
        }
        Err(e) => {
            event!(Level::INFO, "folder already exit, {e}");
            0
        }
    }
}

//Crea el fichero donde se almacena los resultados
pub async fn create_db() -> u8 {
    let path_folder: &str = "./db/db_main.json";
    //Da igual el resultado de folder porque ya devolveria error en caso de no poder crear el directorio
    let _ = create_folder().await;
    let result_file_create = fs::File::create_new(path_folder).await;
    match result_file_create {
        Ok(n) => {
            event!(Level::INFO, "created file {:?}", n);
            1
        }
        Err(e) => {
            event!(Level::INFO, "file already exit, {e}");
            0
        }
    }
}

pub async fn append_result(
    Entry: EntryChire,
    db: String,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let path_db = format!("./db/db_main.json");
    println!("{:?}", path_db);
    let mut file = File::options().append(true).open(path_db).await?;
    let jeje = file.write_all(b"src").await;
    let ho = file.flush().await;
    match ho {
        Ok(n) => {
            println!("{:?}", n);
        }
        Err(e) => {
            println!("{e}");
        }
    }
    Ok(())
}
