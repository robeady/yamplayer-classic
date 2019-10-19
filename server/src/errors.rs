use serde_derive::Serialize;
use std::fmt;

pub type Try<T> = anyhow::Result<T>;

pub type Erro = anyhow::Error;
