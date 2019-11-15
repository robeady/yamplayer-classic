use crate::errors::Try;
use crate::services::ServiceId;
use serde_derive::Serialize;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::FromStr;

pub trait Entity {}

// TODO: we can avoid these derives if we explicitly implement traits on the Id types

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Serialize, Hash)]
pub struct Track {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Serialize, Hash)]
pub struct Album {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Serialize, Hash)]
pub struct Artist {}
#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Serialize, Hash)]
pub struct Playlist {}

impl Entity for Track {}
impl Entity for Album {}
impl Entity for Artist {}
impl Entity for Playlist {}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Id<E: Entity> {
    Library(LibraryId<E>),
    External(ExternalId<E>),
}

impl<E: Entity> FromStr for Id<E> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        match parts.as_slice() {
            [p] => Ok(Self::Library(LibraryId(p.parse()?, PhantomData))),
            [service, id] => Ok(Self::External(ExternalId {
                service: ServiceId((*service).to_owned()),
                id: IdString::new((*id).to_owned()),
            })),
            _ => Err(anyhow!("invalid ID {}", s)),
        }
    }
}

impl<E: Entity> Display for Id<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Library(library_id) => library_id.fmt(f),
            Self::External(external_id) => external_id.fmt(f),
        }
    }
}

impl<T: Entity> serde::Serialize for Id<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serde::string::serialize(self, serializer)
    }
}
impl<'de, T: Entity> serde::Deserialize<'de> for Id<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::serde::string::deserialize(deserializer)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct LibraryId<E: Entity>(pub i64, PhantomData<E>);

impl<E: Entity> LibraryId<E> {
    pub fn new(id: i64) -> Self {
        Self(id, PhantomData)
    }
}

impl<T: Entity> serde::Serialize for LibraryId<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serde::string::serialize(self, serializer)
    }
}
impl<'de, T: Entity> serde::Deserialize<'de> for LibraryId<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::serde::string::deserialize(deserializer)
    }
}

impl<E: Entity> FromStr for LibraryId<E> {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Try<Self> {
        Ok(Self::new(s.parse()?))
    }
}

impl<E: Entity> Display for LibraryId<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct ExternalId<E: Entity> {
    pub service: ServiceId,
    pub id: IdString<E>,
}

impl<E: Entity> Display for ExternalId<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.service.0, self.id.0)
    }
}

impl<T: Entity> serde::Serialize for ExternalId<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::serde::string::serialize(self, serializer)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct IdString<E: Entity>(pub String, PhantomData<E>);

impl<E: Entity> IdString<E> {
    pub fn new(id: String) -> Self {
        Self(id, PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_serialize_as_strings() {
        struct Foo {}
        impl Entity for Foo {}
        assert_eq!(
            ::serde_json::to_string(&LibraryId::<Foo>::new(2)).unwrap(),
            "\"2\""
        );
        assert_eq!(
            ::serde_json::to_string(&Id::<Foo>::Library(LibraryId::new(4))).unwrap(),
            "\"4\""
        );
        assert_eq!(
            ::serde_json::to_string(&Id::<Foo>::External(ExternalId {
                service: ServiceId("foo".to_owned()),
                id: IdString::new("bar")
            }))
            .unwrap(),
            "\"foo:bar\""
        );
    }
}
