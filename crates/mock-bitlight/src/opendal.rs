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
use opendal::layers::RetryLayer;
use opendal::services::{Fs, S3};
use opendal::Operator;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::runtime::Handle;
use tokio::sync::{Mutex, OnceCell};
use tokio::task::block_in_place;

#[derive(Debug)]
pub struct OpendalOperator {
    inner: Operator,
}

pub static BACKUP_RECOVERY_OPERATOR: OnceCell<OpendalOperator> = OnceCell::const_new();

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub struct OpendalConfig {
    #[serde(flatten)]
    pub backend: StoreBackendConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum StoreBackendConfig {
    Fs { root: PathBuf },
    S3 { bucket: String, region: String },
}

impl Default for StoreBackendConfig {
    fn default() -> Self {
        StoreBackendConfig::Fs {
            root: PathBuf::from("./"),
        }
    }
}

impl OpendalConfig {
    pub async fn opendal_operator(&self) -> &'static OpendalOperator {
        BACKUP_RECOVERY_OPERATOR
            .get_or_init(|| async {
                let inner = match self.backend {
                    StoreBackendConfig::Fs { ref root } => {
                        let path = root;
                        let fs_builder = Fs::default().root(path.display().to_string().as_str());
                        Operator::new(fs_builder)
                            .expect("Failed to create opendal fs operator")
                            .layer(RetryLayer::new())
                            .finish()
                    }
                    StoreBackendConfig::S3 {
                        ref bucket,
                        ref region,
                    } => {
                        let s3_builder = S3::default().bucket(bucket).region(region);
                        Operator::new(s3_builder)
                            .expect("Failed to create opendal s3 operator")
                            .layer(RetryLayer::new())
                            .finish()
                    }
                };

                OpendalOperator { inner }
            })
            .await
    }
}

#[derive(Debug)]
pub struct OpendalContainer {
    operator: &'static OpendalOperator,
    user_id: String,
}

impl OpendalContainer {
    pub async fn make_container(
        user_id: &str,
        config: OpendalConfig,
    ) -> io::Result<OpendalContainer> {
        let operator = config.opendal_operator().await;
        Ok(OpendalContainer {
            operator,
            user_id: user_id.to_string(),
        })
    }
}

impl<K, D: Descriptor<K>, L2D: Layer2Descriptor> StoreProvider<WalletDescr<K, D, L2D>>
    for OpendalContainer
where for<'a> WalletDescr<K, D, L2D>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletDescr<K, D, L2D>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let path = format!("{}/descr.toml", self.user_id);
                let buffer = self.operator.inner.read(&path).await?;
                let string = String::from_utf8(buffer.to_bytes().to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletDescr<K, D, L2D>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");

        block_in_place(move || {
            Handle::current().block_on(async move {
                let path = format!("{}/descr.toml", self.user_id);
                self.operator.inner.write(&path, data.into_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

impl<L2> StoreProvider<WalletData<L2>> for OpendalContainer
where
    for<'a> WalletData<L2>: Serialize + Deserialize<'a>,
    L2: Layer2Data,
{
    fn load(&self) -> Result<WalletData<L2>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let path = format!("{}/data.toml", self.user_id);
                let buffer = self.operator.inner.read(&path).await?;
                let string = String::from_utf8(buffer.to_bytes().to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(toml::from_str(&data)?)
    }

    fn store(&self, object: &WalletData<L2>) -> Result<(), StoreError> {
        let data = toml::to_string_pretty(object).expect("");

        block_in_place(move || {
            Handle::current().block_on(async move {
                let path = format!("{}/data.toml", self.user_id);
                self.operator.inner.write(&path, data.into_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

impl<L2: Layer2Cache> StoreProvider<WalletCache<L2>> for OpendalContainer
where for<'a> WalletCache<L2>: Serialize + Deserialize<'a>
{
    fn load(&self) -> Result<WalletCache<L2>, LoadError> {
        let data = block_in_place(|| {
            Handle::current().block_on(async {
                let path = format!("{}/cache.yaml", self.user_id);
                let buffer = self.operator.inner.read(&path).await.expect("");
                let string = String::from_utf8(buffer.to_bytes().to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok::<_, io::Error>(string)
            })
        })?;
        Ok(serde_yaml::from_str(&data).expect(""))
    }

    fn store(&self, object: &WalletCache<L2>) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object).expect("");

        block_in_place(move || {
            Handle::current().block_on(async move {
                let path = format!("{}/cache.yaml", self.user_id);
                self.operator.inner.write(&path, data.into_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}

impl StoreProvider<NoLayer2> for OpendalContainer
where NoLayer2: Layer2
{
    fn load(&self) -> Result<NoLayer2, LoadError> { Ok(None) }

    fn store(&self, object: &NoLayer2) -> Result<(), StoreError> {
        let data = serde_yaml::to_string(object)?;

        block_in_place(move || {
            Handle::current().block_on(async {
                let path = format!("{}/layer2.toml", self.user_id);
                self.operator.inner.write(&path, data.into_bytes()).await?;
                Ok::<_, io::Error>(())
            })
        })?;
        Ok(())
    }
}
