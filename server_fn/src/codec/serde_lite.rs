use crate::{
    codec::Post, error::ServerFnErrorErr, ContentType, Decodes, Encodes,
};
use bytes::Bytes;
use serde_lite::{Deserialize, Serialize};

/// Pass arguments and receive responses as JSON in the body of a `POST` request.
pub struct SerdeLite;

impl ContentType for SerdeLite {
    const CONTENT_TYPE: &'static str = "application/json";
}

impl<T> Encodes<T> for SerdeLite
where
    T: Serialize,
{
    type Error = ServerFnErrorErr;

    fn encode(value: T) -> Result<Bytes, Self::Error> {
        serde_json::to_vec(
            &value
                .serialize()
                .map_err(|e| ServerFnErrorErr::Serialization(e.to_string()))?,
        )
        .map_err(|e| ServerFnErrorErr::Serialization(e.to_string()))
        .map(Bytes::from)
    }
}

impl<T> Decodes<T> for SerdeLite
where
    T: Deserialize,
{
    type Error = ServerFnErrorErr;

    fn decode(bytes: Bytes) -> Result<T, Self::Error> {
        T::deserialize(
            &serde_json::from_slice(&bytes).map_err(|e| {
                ServerFnErrorErr::Deserialization(e.to_string())
            })?,
        )
        .map_err(|e| ServerFnErrorErr::Deserialization(e.to_string()))
    }
}

/// Pass arguments and receive responses as JSON in the body of a `POST` request.
pub type PostSerdeLite = Post<SerdeLite>;
