use std::fmt::Debug;
use std::path::PathBuf;

use bpstd::Descriptor;
use serde::{Deserialize, Serialize};

use crate::fs::{LoadError, StoreError};
use crate::{
    Layer2, Layer2Cache, Layer2Data, Layer2Descriptor, Save, Wallet, WalletCache, WalletData,
    WalletDescr,
};

pub trait StoreProvider<T>: Send + Debug {
    fn load(&self) -> Result<T, LoadError>;
    fn store(&self, object: &T) -> Result<(), StoreError>;
}

impl<K, D: Descriptor<K>, L2D: Layer2Descriptor> StoreProvider<WalletDescr<K, D, L2D>> for PathBuf
where for<'a> WalletDescr<K, D, L2D>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletDescr<K, D, L2D>, LoadError> {
        let data = std::fs::read_to_string(self)?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletDescr<K, D, L2D>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");
        Ok(std::fs::write(self, data)?)
    }
}

impl<L2: Layer2Data> StoreProvider<WalletData<L2>> for PathBuf
where for<'a> WalletData<L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletData<L2>, LoadError> {
        let data = std::fs::read_to_string(self)?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletData<L2>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");
        Ok(std::fs::write(self, data)?)
    }
}

impl<L2: Layer2Cache> StoreProvider<WalletCache<L2>> for PathBuf
where for<'a> WalletCache<L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletCache<L2>, LoadError> {
        let data = std::fs::read_to_string(self)?;
        Ok(serde_yaml::from_str(&data)?)
    }

    fn store(&self, object: &WalletCache<L2>) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object)?;
        Ok(std::fs::write(self, data)?)
    }
}

impl<L2: Layer2> StoreProvider<L2> for PathBuf
where
    for<'a> L2: Serialize + Deserialize<'a>,
    for<'a> L2::Descr: Serialize + Deserialize<'a> + Layer2Descriptor,
    for<'a> L2::Data: Serialize + Deserialize<'a> + Layer2Data,
    for<'a> L2::Cache: Serialize + Deserialize<'a> + Layer2Cache,
{
    fn load(&self) -> Result<L2, LoadError> {
        let data = std::fs::read_to_string(self)?;
        Ok(serde_yaml::from_str(&data)?)
    }

    fn store(&self, object: &L2) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object)?;
        Ok(std::fs::write(self, data)?)
    }
}

impl<K, D, L2> StoreProvider<Wallet<K, D, L2>> for PathBuf
where
    Wallet<K, D, L2>: Save,
    for<'de> D: serde::Serialize + serde::Deserialize<'de> + Descriptor<K>,
    for<'de> L2: serde::Serialize + serde::Deserialize<'de> + Layer2,
    for<'de> L2::Descr: serde::Serialize + serde::Deserialize<'de> + Layer2Descriptor,
    for<'de> L2::Data: serde::Serialize + serde::Deserialize<'de> + Layer2Data,
    for<'de> L2::Cache: serde::Serialize + serde::Deserialize<'de> + Layer2Cache,
{
    fn load(&self) -> Result<Wallet<K, D, L2>, LoadError> {
        let mut descr = self.to_owned();
        descr.push("descriptor.toml");

        let mut data = self.to_owned();
        data.push("data.toml");

        let mut cache: PathBuf = self.to_owned();
        cache.push("cache.yaml");

        let mut layer2 = self.to_owned();
        layer2.push("layer2");

        let descr = descr.load()?;
        let data = data.load()?;
        let cache = cache.load()?;
        let layer2 = layer2.load()?;
        Ok(Wallet::with(descr, data, cache, layer2))
    }

    fn store(&self, object: &Wallet<K, D, L2>) -> Result<(), StoreError> {
        let mut descr = self.to_owned();
        descr.push("descriptor.toml");

        let mut data = self.to_owned();
        data.push("data.toml");

        let mut cache: PathBuf = self.to_owned();
        cache.push("cache.yaml");

        let mut layer2 = self.to_owned();
        layer2.push("layer2");

        descr.store(object.descr())?;
        data.store(object.data())?;
        cache.store(object.cache())?;
        layer2.store(object.layer2())?;
        Ok(())
    }
}
