use serde::{Deserialize, Serialize};

use crate::components::containers::kvpair::KVPair;
use crate::utils::ApiError;

#[derive(Serialize, Deserialize)]
pub struct SymbolConfig {
    pub symbols: Vec<SymbolSerde>,
}
#[derive(Serialize, Deserialize)]
pub struct SymbolSerde(pub KVPair<String, String>);

pub type SymbolKVPair = KVPair<Option<String>, String>;
impl TryFrom<SymbolKVPair> for SymbolSerde {
    type Error = ApiError;

    fn try_from(pair: SymbolKVPair) -> Result<Self, Self::Error> {
        Ok(SymbolSerde(KVPair {
            key: pair
                .key
                .ok_or::<Self::Error>("Could not unwrap {pair:?}".into())?,
            value: pair.value,
        }))
    }
}
impl From<SymbolSerde> for SymbolKVPair {
    fn from(pair: SymbolSerde) -> Self {
        Self {
            key: Some(pair.0.key),
            value: pair.0.value,
        }
    }
}
