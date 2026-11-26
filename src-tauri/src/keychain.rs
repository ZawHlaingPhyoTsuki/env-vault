use std::sync::Mutex;

use crate::crypto::{self, MasterKey, KEY_LEN};
use crate::error::{AppError, AppResult};

pub const SERVICE: &str = "envvault";
pub const ACCOUNT: &str = "master-key";

pub trait Keychain: Send + Sync {
    fn get_master_key(&self) -> AppResult<Option<MasterKey>>;
    fn set_master_key(&self, key: &[u8; KEY_LEN]) -> AppResult<()>;
}

pub struct OsKeychain;

impl OsKeychain {
    fn entry(&self) -> AppResult<keyring::Entry> {
        keyring::Entry::new(SERVICE, ACCOUNT)
            .map_err(|e| AppError::Keychain(format!("failed to access OS keychain: {e}")))
    }
}

impl Keychain for OsKeychain {
    fn get_master_key(&self) -> AppResult<Option<MasterKey>> {
        let entry = self.entry()?;
        match entry.get_secret() {
            Ok(bytes) => {
                let hex = String::from_utf8(bytes)
                    .map_err(|_| AppError::Keychain("stored master key is invalid".into()))?;
                let key = crypto::decode_key(hex.trim())?;
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Keychain(format!("failed to read master key: {e}"))),
        }
    }

    fn set_master_key(&self, key: &[u8; KEY_LEN]) -> AppResult<()> {
        let entry = self.entry()?;
        let hex = crypto::encode_key(key);
        entry
            .set_secret(hex.as_bytes())
            .map_err(|e| AppError::Keychain(format!("failed to store master key: {e}")))
    }
}

#[derive(Debug, Default)]
pub struct MockKeychain {
    key: Mutex<Option<String>>,
}

impl MockKeychain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_key(key: &[u8; KEY_LEN]) -> Self {
        Self {
            key: Mutex::new(Some(crypto::encode_key(key))),
        }
    }
}

impl Keychain for MockKeychain {
    fn get_master_key(&self) -> AppResult<Option<MasterKey>> {
        let guard = self
            .key
            .lock()
            .map_err(|_| AppError::Keychain("mock keychain lock poisoned".into()))?;
        match guard.as_deref() {
            Some(hex) => Ok(Some(crypto::decode_key(hex)?)),
            None => Ok(None),
        }
    }

    fn set_master_key(&self, key: &[u8; KEY_LEN]) -> AppResult<()> {
        let mut guard = self
            .key
            .lock()
            .map_err(|_| AppError::Keychain("mock keychain lock poisoned".into()))?;
        *guard = Some(crypto::encode_key(key));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_key;

    #[test]
    fn mock_keychain_round_trip() {
        let keychain = MockKeychain::new();
        assert!(keychain.get_master_key().unwrap().is_none());
        let key = generate_master_key();
        keychain.set_master_key(&key).unwrap();
        let loaded = keychain.get_master_key().unwrap().unwrap();
        assert_eq!(key.as_ref(), loaded.as_ref());
    }

    #[test]
    fn mock_keychain_with_key() {
        let key = generate_master_key();
        let keychain = MockKeychain::with_key(&key);
        let loaded = keychain.get_master_key().unwrap().unwrap();
        assert_eq!(key.as_ref(), loaded.as_ref());
    }
}
