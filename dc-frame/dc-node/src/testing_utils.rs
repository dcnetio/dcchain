use crate::*;
use frame_benchmarking::{account};
use sp_std::{
    prelude::*,
    collections::{btree_set::BTreeSet},
};

pub fn add_onchain_node<T: Config>(peer_id: PeerId, name: &'static str, status: u32) -> T::AccountId {
    let total_space = 100*1024*1024*1024*1024;
    let free_space = 100*1024*1024*1024*1024;
    let ip_address = vec![36; 256];
    let report_number = frame_system::Pallet::<T>::block_number();
    let caller: T::AccountId = account(name, 0, 0);
    let balance = T::Currency::minimum_balance()*100u32.into() + 40_1000_1000u32.into();
    let _ = T::Currency::make_free_balance_be(&caller, balance);

    let node_info = StorageNode {
        req_account: caller.clone(),
        stash: T::DefaultAccountId::get(),
        total_space: total_space,
        free_space: free_space,
        status: status,
        report_number: report_number,
        staked_number: Zero::zero(),
        reward_number: report_number,
        ip_address: ip_address.clone(),
    };
    Peers::<T>::insert(&peer_id, node_info);
    <RequestAccountPeer<T>>::insert(&caller, &peer_id);
    if status == NODE_STATUS_ONCHAIN {
        OnchainPeerNumber::<T>::mutate(|n| *n += 1);
    }
    caller
}

pub fn user_purchase_storage<T: Config>(name: &'static str) -> T::AccountId {
    user_purchase_storage_index::<T>(name, 0)
}

pub fn user_purchase_storage_index<T: Config>(name: &'static str, index: u32) -> T::AccountId {
    let for_account: T::AccountId = account(name, index, 0);
    let balance = T::Currency::minimum_balance()*100u32.into() + 40_1000_1000u32.into();
    let _ = T::Currency::make_free_balance_be(&for_account, balance);

    let mut packages = BTreeSet::<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>::new();
    packages.insert((1, 60*1024*1024*1024, 1000u32.into(), 10000u32.into(), 10u32.into()));
    packages.insert((2, 120*1024*1024*1024, 2000u32.into(), 10000u32.into(), 5u32.into()));
    <StoragePackages::<T>>::put(packages);

    let new_user = UserStorage {
        peers: BTreeSet::new(),
        used_space: 10000,
        subscribe_space: 20000,
        subscribe_price: 1000u32.into(),
        call_minus_number: 10u32.into(),
        nft_update_number: 1000u32.into(),
        db_update_number: 1000u32.into(),
        expire_number: frame_system::Pallet::<T>::block_number()+10000u32.into(),
        db_config: DcString::new(),
        enc_nft_account: NftAccount::new(),
        parent_account: for_account.clone(),
        spam_frozen_status: 0,
        spam_report_amount: 0,
        spam_report_number: 0u32.into(),
        comment_frozen_status: 0,
        comment_report_amount: 0,
        comment_report_number: 0u32.into(),
        login_number: 0u32.into(),
        comment_space: 0,
        request_peers: BTreeSet::new(),
    };
    <WalletAccountStorage<T>>::insert(&for_account, new_user);
    for_account
}

