mod async_file_container;
mod opendal;
use std::path::PathBuf;
use std::str::FromStr;

use async_file_container::AsyncFileContainer;
use bpstd::{Network, StdDescr, TrKey, XpubDerivable};
use bpwallet::indexers::esplora::Client;
use bpwallet::persistence::StoreProvider;
use bpwallet::{FsConfig, NoLayer2, Save, Wallet, WalletCache, WalletData, WalletDescr};
use opendal::{OpendalConfig, OpendalContainer, OpendalOperator};

#[tokio::main]
async fn main() {
    let user_id = "clvi67aav0009i996zkh6vdlv".to_string();
    let descr = XpubDerivable::from_str("[893ab691/86h/1h/0h]tpubDC3FhuA5HwZWPxRYygVugxmsfqSqqhpLoQi6Bz4YUHFD1eXYqn6bJk9nbDe2DRws82GgGePNEL5XWFtNkSzi4eKAbFpkGcxJX2181CwL6vi/<0;1;9;10>/*").expect("");
    let std_descr: StdDescr = TrKey::from(descr.clone()).into();
    let mut wallet = Wallet::new_layer1(std_descr, Network::Testnet3);
    dbg!(&wallet);
    dbg!(wallet.balance());

    // DO NOT USE THIS
    // wallet
    //     .set_fs_config(FsConfig {
    //         // useless path
    //         path: PathBuf::from_str("").expect(""),
    //         autosave: false,
    //     })
    //     .expect("");

    let (descr, data, cache) = AsyncFileContainer::make_container(&user_id).await.expect("");
    wallet.make_descr_store_provider(descr);
    wallet.make_data_store_provider(data);
    wallet.make_cache_store_provider(cache);

    let client = Client::new_esplora("https://bitcoin-testnet-api.bitlightdev.info").expect("");
    wallet.update(&client);

    let balance = wallet.balance();
    dbg!(balance);

    wallet.save().expect("");
    drop(wallet);

    let config = OpendalConfig {
        backend: Default::default(),
    };
    let container = OpendalContainer::make_container(&user_id, config).await.expect("");

    let descr: WalletDescr<_, StdDescr, NoLayer2> = container.load().expect("");
    let data: WalletData<NoLayer2> = container.load().expect("");
    let cache: WalletCache<NoLayer2> = container.load().expect("");
    let layer2: NoLayer2 = container.load().expect("");
    let wallet = Wallet::with(descr, data, cache, layer2);
    dbg!(wallet.balance());
    
    assert!(balance == wallet.balance());
}
