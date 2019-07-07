use serde_derive::Serialize;

pub type Try<T> = std::result::Result<T, Erro>;

#[derive(Debug, Serialize)]
pub enum Erro {
    StringError(String),
}

pub fn string_err(s: impl Into<String>) -> Erro {
    Erro::StringError(s.into())
}

impl<E: std::error::Error> From<E> for Erro {
    fn from(e: E) -> Self {
        Erro::StringError(e.to_string())
    }
}
