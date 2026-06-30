// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{State, Manager};
use redb::{Database, TableDefinition};
use std::sync::Arc;
use std::path::PathBuf;

const BOARDS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("boards_v2");

struct AppState {
    db: Arc<Database>,
    app_data_dir: PathBuf,
}

#[tauri::command]
fn save_board(state: State<'_, AppState>, payload: Vec<u8>) -> Result<(), String> {
    let write_txn = state.db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = write_txn.open_table(BOARDS_TABLE).map_err(|e| e.to_string())?;
        table.insert("default_board", payload.as_slice()).map_err(|e| e.to_string())?;
    }
    write_txn.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_board(state: State<'_, AppState>) -> Result<Option<Vec<u8>>, String> {
    let read_txn = state.db.begin_read().map_err(|e| e.to_string())?;
    let table = read_txn.open_table(BOARDS_TABLE).map_err(|e| e.to_string())?;
    
    match table.get("default_board").map_err(|e| e.to_string())? {
        Some(guard) => Ok(Some(guard.value().to_vec())),
        None => Ok(None),
    }
}

#[tauri::command]
fn save_asset(state: State<'_, AppState>, id: String, base64_data: String) -> Result<(), String> {
    let assets_dir = state.app_data_dir.join("assets");
    std::fs::create_dir_all(&assets_dir).map_err(|e| e.to_string())?;
    
    let file_path = assets_dir.join(&id);
    std::fs::write(file_path, base64_data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn load_asset(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let file_path = state.app_data_dir.join("assets").join(&id);
    std::fs::read_to_string(file_path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path_resolver().app_data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&app_data_dir).unwrap();
            let db_path = app_data_dir.join("aerial_store_os.redb");
            let db = Database::create(db_path).expect("Failed to create redb database");
            
            let write_txn = db.begin_write().expect("Failed to begin write txn");
            {
                let _ = write_txn.open_table(BOARDS_TABLE).expect("Failed to open table");
            }
            write_txn.commit().expect("Failed to commit txn");
            
            app.manage(AppState {
                db: Arc::new(db),
                app_data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_board,
            load_board,
            save_asset,
            load_asset,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
