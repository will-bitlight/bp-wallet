mod async_file_container;
mod opendal;
use std::str::FromStr;

use async_file_container::AsyncFileContainer;
use bpstd::{Network, StdDescr, TrKey, XpubDerivable};
use bpwallet::indexers::esplora::Client;
use bpwallet::Wallet;
use opendal::{OpendalConfig, OpendalOperator};

#[tokio::main]
async fn main() {
    let descr = XpubDerivable::from_str("[893ab691/86h/1h/0h]tpubDC3FhuA5HwZWPxRYygVugxmsfqSqqhpLoQi6Bz4YUHFD1eXYqn6bJk9nbDe2DRws82GgGePNEL5XWFtNkSzi4eKAbFpkGcxJX2181CwL6vi/<0;1;9;10>/*").expect("");
    let std_descr: StdDescr = TrKey::from(descr.clone()).into();
    let mut wallet = Wallet::new_layer1(std_descr, Network::Testnet3);
    dbg!(&wallet);
    dbg!(wallet.balance());

    let client = Client::new_esplora("https://bitcoin-testnet-api.bitlightdev.info").expect("");
    wallet.update(&client);

    dbg!(wallet.balance());
}
