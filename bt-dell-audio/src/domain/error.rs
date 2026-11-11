#![allow(dead_code)]

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Erro de E/S no caminho '{path}': {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Variável de ambiente necessária não encontrada: {0}")]
    EnvVarMissing(String),

    #[error("Falha ao executar o comando '{command}': {details}")]
    CommandExecution {
        command: String,
        details: String,
    },

    #[error("Falha ao analisar a saída de '{command}': {details}")]
    ParseError {
        command: String,
        details: String,
    },

    #[error("Permissão insuficiente: Esta operação exige privilégios de superusuário (root)")]
    PermissionDenied,
}

pub type AppResult<T> = Result<T, AppError>;
