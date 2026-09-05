use crate::db::Db;
use crate::transcribe::Transcriber;
use std::sync::Mutex;

pub struct AppState {
    pub db: Db,
    pub transcriber: Mutex<Option<Transcriber>>,
}
