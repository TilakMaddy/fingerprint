use std::str::FromStr;

use alloy::{
    primitives::{Address, Bytes},
    providers::ProviderBuilder,
};

use crate::config;

use super::CHAT;

pub async fn are_friends(person1: Address, person2: &str) -> bool {
    let conf = config::config();

    let person2_address = Address::from_str(person2).expect("invalid fren");

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let status = contract
        .getFriendshipStatus(person1, person2_address)
        .call()
        .await
        .unwrap()
        ._0;

    status == 2
}

pub async fn get_dh_pubkey(person1: Address, person2: &str) -> Option<Bytes> {
    let conf = config::config();

    let person2_address = Address::from_str(person2).expect("invalid fren");

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let dh_keys = contract
        .getDHPublicKeys(person1, person2_address)
        .call()
        .await
        .ok()?;

    Some(dh_keys.f2Key)
}

pub async fn get_incoming_friend_requests(who: Address) -> Option<Vec<Address>> {
    let conf = config::config();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let incoming_requests = contract
        .getIncomingPendingRequests(who)
        .call()
        .await
        .ok()?
        ._0;

    Some(incoming_requests)
}

pub async fn get_outgoing_friend_requests(who: Address) -> Option<Vec<Address>> {
    let conf = config::config();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let incoming_requests = contract.getOutgoingRequests(who).call().await.ok()?;
    let adds = incoming_requests._0;
    let stai = incoming_requests._1;

    let mut pending_requests = vec![];

    for i in 0..adds.len() {
        let add = adds[i];
        let sta = stai[i];

        //Pending - 1
        //Accepted - 2
        //Rejected - 3

        if sta == 1 {
            pending_requests.push(add);
        }
    }

    Some(pending_requests)
}

pub async fn get_friend_list(who: Address) -> Option<Vec<Address>> {
    let conf = config::config();

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .on_http(conf.rpc.parse().expect("invalid rpc"));

    let contract = CHAT::new(conf.contract_address.parse().unwrap(), provider);

    let fll = contract.getFriendList(who).call().await.ok()?;
    let adds = fll._0;

    Some(adds)
}
