use std::fmt::Debug;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use bpstd::Descriptor;
use bpwallet::fs::{LoadError, StoreError};
use bpwallet::persistence::StoreProvider;
use bpwallet::{
    Layer2, Layer2Cache, Layer2Data, Layer2Descriptor, NoLayer2, Save, Wallet, WalletCache,
    WalletData, WalletDescr,
};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::task::block_in_place;

#[derive(Debug)]
pub struct AsyncFileContainer {
    file: Arc<Mutex<File>>,
}

impl AsyncFileContainer {
    pub async fn make_container(
        user_id: String,
    ) -> io::Result<(AsyncFileContainer, AsyncFileContainer, AsyncFileContainer)> {
        let user_path = PathBuf::from(format!("./{}", user_id));
        let descr_path = user_path.join("descr.toml");
        let data_path = user_path.join("data.toml");
        let cache_path = user_path.join("cache.toml");

        let descr_file = File::open(descr_path).await?;
        let data_file = File::open(data_path).await?;
        let cache_file = File::open(cache_path).await?;

        let descr_container = AsyncFileContainer {
            file: Arc::new(Mutex::new(descr_file)),
        };
        let data_container = AsyncFileContainer {
            file: Arc::new(Mutex::new(data_file)),
        };
        let cache_container = AsyncFileContainer {
            file: Arc::new(Mutex::new(cache_file)),
        };

        Ok((descr_container, data_container, cache_container))
    }
}

impl<K, D: Descriptor<K>, L2: Layer2Descriptor> StoreProvider<WalletDescr<K, D, L2>>
    for AsyncFileContainer
where for<'a> WalletDescr<K, D, L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletDescr<K, D, L2>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let mut string = String::new();
                let mut file = self.file.lock().await;
                file.read_to_string(&mut string).await?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletDescr<K, D, L2>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");

        block_in_place(|| {
            Handle::current().block_on(async {
                let mut file = self.file.lock().await;
                file.write_all(data.as_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

impl<L2: Layer2Data> StoreProvider<WalletData<L2>> for AsyncFileContainer
where for<'a> WalletData<L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletData<L2>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let mut string = String::new();
                let mut file = self.file.lock().await;
                file.read_to_string(&mut string).await?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletData<L2>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");

        block_in_place(|| {
            Handle::current().block_on(async {
                let mut file = self.file.lock().await;
                file.write_all(data.as_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

impl<L2: Layer2Cache> StoreProvider<WalletCache<L2>> for AsyncFileContainer
where for<'a> WalletCache<L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletCache<L2>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let mut string = String::new();
                let mut file = self.file.lock().await;
                file.read_to_string(&mut string).await?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(serde_yaml::from_str(&data)?)
    }

    fn store(&self, object: &WalletCache<L2>) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object)?;

        block_in_place(|| {
            Handle::current().block_on(async {
                let mut file = self.file.lock().await;
                file.write_all(data.as_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

// TODO: error, from compiler limit
// upstream crates may add a new impl of trait `bpwallet::Layer2`
// for type `bpwallet::WalletDescr<_, _, _>` in future versions
// impl<L2: Layer2> StoreProvider<L2> for AsyncFileContainer
// where
//     for<'a> L2: Serialize + Deserialize<'a>,
//     for<'a> L2::Descr: Serialize + Deserialize<'a> + Layer2Descriptor,
//     for<'a> L2::Data: Serialize + Deserialize<'a> + Layer2Data,
//     for<'a> L2::Cache: Serialize + Deserialize<'a> + Layer2Cache,
// {
//     fn load(&self) -> Result<L2, LoadError> {
//         let data = block_in_place(|| {
//             Handle::current().block_on(async {
//                 let mut string = String::new();
//                 let mut file = self.file.lock().await;
//                 file.read_to_string(&mut string).await?;
//                 Ok::<_, io::Error>(string)
//             })
//         })?;
//         Ok(serde_yaml::from_str(&data)?)
//     }

//     fn store(&self, object: &L2) -> Result<(), StoreError> {
//         let data = serde_yaml::to_string(object)?;

//         block_in_place(|| {
//             Handle::current().block_on(async {
//                 let mut file = self.file.lock().await;
//                 file.write_all(data.as_bytes()).await?;
//                 Ok::<_, io::Error>(())
//             })
//         })?;
//         Ok(())
//     }
// }

impl StoreProvider<NoLayer2> for AsyncFileContainer
where NoLayer2: Layer2
{
    fn load(&self) -> Result<NoLayer2, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let mut string = String::new();
                let mut file = self.file.lock().await;
                file.read_to_string(&mut string).await?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(serde_yaml::from_str(&data)?)
    }

    fn store(&self, object: &NoLayer2) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object)?;

        block_in_place(|| {
            Handle::current().block_on(async {
                let mut file = self.file.lock().await;
                file.write_all(data.as_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}
