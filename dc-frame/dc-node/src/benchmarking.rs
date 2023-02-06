

#![cfg(feature = "runtime-benchmarks")]

use crate::*;
use testing_utils::*;
use frame_benchmarking::{benchmarks, whitelisted_caller, account};
use frame_system::RawOrigin;
use sp_std::{
    fmt::Debug,
    vec::Vec,
    prelude::*,
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
};
use sp_runtime::SaturatedConversion;

benchmarks! {
    where_clause { where
        T::AccountId: AsRef<[u8]>,
    }

    join_storage_node {
        let peer_id = vec![31; 32];
        let total_space = 100*1024*1024*1024*1024;
        let free_space = 100*1024*1024*1024*1024;
        let ip_address = vec![33; 256];
        let report_number = 10293u32.into();
        let tee_report = vec![38; 512];
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), peer_id.clone(), total_space, free_space, ip_address, report_number, tee_report)
    verify {
        assert!(Peers::<T>::contains_key(peer_id));
    }

    submit_work_report {
        let n in 1 .. MISSING_FILES_MAX_NUM;
        let m in 1 .. MISSING_FILES_MAX_NUM;
        let peer_id = vec![36; 32];
        let total_space = 100*1024*1024*1024*1024;
        let free_space = 100*1024*1024*1024*1024;
        let ip_address = vec![36; 256];
    
        let caller = add_onchain_node::<T>(peer_id.clone(), "submit_work_report", NODE_STATUS_ONCHAIN);

        let owner = user_purchase_storage::<T>("owner");
        let file_size = 1000123; 
        let file_type = 1;
        let tee_report = vec![33; 1024];
        let signature = vec![35; 1024];

        let mut miss_files = Vec::new();
        for i in 0 .. n {
            miss_files.push(vec![32; usize::try_from(i).unwrap()]);
            let file_id = vec![32; usize::try_from(i).unwrap()]; 
            let _ = Pallet::<T>::add_file_info(
                RawOrigin::Signed(caller.clone()).into(), 
                owner.clone(), 
                file_id.clone(), 
                file_size, 
                file_type, 
                1000u32.into(), 
                signature.clone()
            );
        }
        let mut miss_accounts = Vec::new();
        for i in 0 .. m {
            let temp_acc: T::AccountId = user_purchase_storage_index::<T>("miss account", i);
            miss_accounts.push(temp_acc.clone());
            let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), temp_acc.clone(), 1000u32.into());
            let _ = Pallet::<T>::set_stash_peer(RawOrigin::Signed(caller.clone()).into(), temp_acc.clone(), peer_id.clone());
        }

        frame_system::Pallet::<T>::set_block_number(frame_system::Pallet::<T>::block_number() + Pallet::<T>::interval_blocks_work_report());
        let report_number = frame_system::Pallet::<T>::block_number();
    }: _(RawOrigin::Signed(caller), total_space, free_space, ip_address, miss_files, miss_accounts, report_number.saturated_into(), tee_report)
    
    purchase_storage {
        let for_account = user_purchase_storage::<T>("purchase_storage");
    }: _(RawOrigin::Signed(for_account.clone()), for_account.clone(), 1)
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.subscribe_space == 60*1024*1024*1024);
    }

    add_request_peer_id_to_user {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_request_peer_id_to_user", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let signature = vec![33; 1024];
    }: _(RawOrigin::Signed(caller), for_account.clone(), 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.request_peers.len() == 1);
    }

    update_db_config {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "update_db_config", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let db_config = vec![31; 256];
        let signature = vec![33; 1024];
    }: _(RawOrigin::Signed(caller), for_account.clone(), db_config.clone(), 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.db_config == db_config);
    }
    
    create_sub_account {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "create_sub_account", NODE_STATUS_ONCHAIN);
        let parent_account = user_purchase_storage::<T>("parent_account");
        let sub_account: T::AccountId = account("sub_account", 0, 0);
        let signature = vec![33; 1024];
    }: _(RawOrigin::Signed(caller), parent_account.clone(), sub_account.clone(), 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(sub_account).unwrap();
        assert!(info.parent_account == parent_account);
    }

    unbind_sub_account {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "unbind_sub_account", NODE_STATUS_ONCHAIN);
        let parent_account = user_purchase_storage::<T>("parent_account");
        let sub_account: T::AccountId = account("sub_account", 0, 0);
        let signature = vec![33; 1024];
        let _ = Pallet::<T>::create_sub_account(RawOrigin::Signed(caller.clone()).into(), parent_account.clone(), sub_account.clone(), 1000u32.into(), signature.clone());
        let info = WalletAccountStorage::<T>::get(&sub_account).unwrap();
        assert!(info.parent_account == parent_account);
    }: _(RawOrigin::Signed(caller), parent_account.clone(), sub_account.clone(), 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&sub_account).unwrap();
        assert!(info.parent_account == sub_account);
    }

    add_user_peer {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_user_peer", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
    }: _(RawOrigin::Signed(caller), for_account.clone(), 1000u32.into())
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.peers.len() == 1);
    }

    remove_self_user_peer {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "remove_self_user_peer", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), for_account.clone(), 1000u32.into());
        let info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(info.peers.len() == 1);
    }: _(RawOrigin::Signed(caller), for_account.clone(), 1000u32.into())
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.peers.len() == 0);
    }

    remove_other_user_peer {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id.clone(), "remove_other_user_peer", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), for_account.clone(), 1000u32.into());
        let info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(info.peers.len() == 1);
        let caller2 = add_onchain_node::<T>(peer_id.clone(), "remove_other_user_peer", NODE_STATUS_ABNORMAL);
    }: _(RawOrigin::Signed(caller2), peer_id, for_account.clone())
    verify {
        let info = WalletAccountStorage::<T>::get(for_account).unwrap();
        assert!(info.peers.len() == 0);
    }

    apply_nft_account {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "apply_nft_account", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let signature = vec![33; 1024];
        let nft_account = vec![36; 32];
        let enc_nft_account = vec![36; 32];
        let private_key_enc_hash = vec![36; 32];
    }: _(RawOrigin::Signed(caller), nft_account.clone(), for_account.clone(), enc_nft_account, private_key_enc_hash, 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(info.peers.len() == 1);
        assert!(NftToWalletAccount::<T>::get(nft_account).unwrap() == for_account);
    }
    
    transfer_nft_account {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "transfer_nft_account", NODE_STATUS_ONCHAIN);
        let from_account = user_purchase_storage::<T>("from_account");
        let to_account = user_purchase_storage::<T>("to_account");
        let signature = vec![33; 1024];
        let nft_account = vec![36; 32];
        let enc_nft_account = vec![36; 32];
        let private_key_enc_hash = vec![36; 32];
        let _ = Pallet::<T>::apply_nft_account(RawOrigin::Signed(caller.clone()).into(), nft_account.clone(), from_account.clone(),
                                               enc_nft_account, private_key_enc_hash, 1000u32.into(), signature.clone());
        let info = WalletAccountStorage::<T>::get(&from_account).unwrap();
        assert!(info.peers.len() == 1);
        assert!(NftToWalletAccount::<T>::get(nft_account.clone()).unwrap() == from_account.clone());
    }: _(RawOrigin::Signed(caller), nft_account.clone(), from_account.clone(), to_account.clone(), 1000u32.into(), signature)
    verify {
        assert!(NftToWalletAccount::<T>::get(nft_account).unwrap() == to_account);
        let info = WalletAccountStorage::<T>::get(&from_account).unwrap();
        assert!(info.peers.len() == 0);
    }
    
    update_nft_account {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "transfer_nft_account", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let signature = vec![33; 1024];
        let nft_account = vec![36; 32];
        let enc_nft_account = vec![36; 32];
        let private_key_enc_hash = vec![36; 32];
        let _ = Pallet::<T>::apply_nft_account(RawOrigin::Signed(caller.clone()).into(), nft_account.clone(), for_account.clone(),
                                               enc_nft_account.clone(), private_key_enc_hash.clone(), 1000u32.into(), signature.clone());
        let info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(info.peers.len() == 1);
        let new_enc_nft_account = vec![38; 32];
    }: _(RawOrigin::Signed(caller), nft_account.clone(), for_account.clone(), new_enc_nft_account.clone(), private_key_enc_hash, 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(info.peers.len() == 1);
        assert!(info.enc_nft_account == new_enc_nft_account);
    }

    add_file_info {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_file_info", NODE_STATUS_ONCHAIN);
        let owner: T::AccountId = account("owner", 0, 0);
        let parent_account = user_purchase_storage::<T>("parent_account");

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![33; 1024];
        let _ = Pallet::<T>::create_sub_account(RawOrigin::Signed(caller.clone()).into(), parent_account.clone(), owner.clone(), 1000u32.into(), signature.clone());
    
        let user_storage = <WalletAccountStorage<T>>::get(&owner).unwrap(); 
    }: _(RawOrigin::Signed(caller), owner.clone(), file_id, file_size, file_type, 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&owner).unwrap();
        assert!(info.used_space == user_storage.used_space+file_size);
    }

    add_file_peer {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_file_peer", NODE_STATUS_ONCHAIN);
        let new_peer_id = vec![39; 32];
        let new_caller = add_onchain_node::<T>(new_peer_id, "new_add_file_peer", NODE_STATUS_ONCHAIN);
        let owner = user_purchase_storage::<T>("owner");
        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![35; 1024];

        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(caller.clone()).into(), owner.clone(), file_id.clone(), file_size, file_type, 1000u32.into(), signature);
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1);
    }: _(RawOrigin::Signed(new_caller), file_id.clone(), 1000u32.into())
    verify {
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 2);
    }

    remove_self_file_peer {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "remove_self_file_peer", NODE_STATUS_ONCHAIN);
        let owner = user_purchase_storage::<T>("owner");
        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![35; 1024];
        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(caller.clone()).into(), owner.clone(), file_id.clone(), file_size, file_type, 1000u32.into(), signature);
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1);
        let user_storage = <WalletAccountStorage<T>>::get(&owner).unwrap(); 
    }: _(RawOrigin::Signed(caller), file_id.clone(), 1u32, 1000u32.into())
    verify {
        assert!(Files::<T>::get(&file_id).is_none());
        let info = WalletAccountStorage::<T>::get(&owner).unwrap();
        assert!(info.used_space == user_storage.used_space-file_size);
    }

    delete_file_info {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_file_info", NODE_STATUS_ONCHAIN);
        let owner: T::AccountId = account("owner", 0, 0);
        let parent_account = user_purchase_storage::<T>("parent_account");

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![33; 1024];
        let _ = Pallet::<T>::create_sub_account(RawOrigin::Signed(caller.clone()).into(), parent_account.clone(), owner.clone(), 1000u32.into(), signature.clone());

        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(caller.clone()).into(), owner.clone(), file_id.clone(), file_size, file_type, 1000u32.into(), signature.clone());
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1);
        let user_storage = <WalletAccountStorage<T>>::get(&owner).unwrap(); 
    }: _(RawOrigin::Signed(caller), owner.clone(), file_id.clone(), file_type, 1000u32.into(), signature.clone())
    verify {
        assert!(Files::<T>::get(&file_id).is_none());
        let info = WalletAccountStorage::<T>::get(&owner).unwrap();
        assert!(info.used_space == user_storage.used_space-file_size);
    }

    add_log_to_thread_db {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_log_to_thread_db", NODE_STATUS_ONCHAIN);
        let owner = user_purchase_storage::<T>("owner");
        let file_id = vec![37; 32]; 
        let log_id = vec![37; 32];
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![33; 1024];

        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(
            caller.clone()).into(), 
            owner.clone(), 
            file_id.clone(), 
            file_size, 
            file_type, 
            1000u32.into(), 
            signature.clone()
        );
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1); 
        let user_storage = <WalletAccountStorage<T>>::get(&owner).unwrap(); 
    }: _(RawOrigin::Signed(caller), file_id, log_id, 1000u32.into(), signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&owner).unwrap();
        assert!(info.used_space == user_storage.used_space);
    }

    add_space_to_thread_db {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_space_to_thread_db", NODE_STATUS_ONCHAIN);
        let owner = user_purchase_storage::<T>("owner");
        let file_id = vec![37; 32]; 
        let increase_size = 67890;
        let signature = vec![33; 1024];
        let file_size = 1000123; 
        let file_type = 1;

        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(
            caller.clone()).into(), 
            owner.clone(), 
            file_id.clone(), 
            file_size, 
            file_type, 
            1000u32.into(), 
            signature.clone()
        );
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1); 
        let user_storage = <WalletAccountStorage<T>>::get(&owner).unwrap();  
    }: _(RawOrigin::Signed(caller), file_id, increase_size, 1000u32.into(), signature.clone())
    verify {
        let info = WalletAccountStorage::<T>::get(&owner).unwrap();
        assert!(info.used_space == user_storage.used_space+u64::try_from(increase_size).unwrap());
    }

    report_file_miss {
        let peer_id = vec![33; 32];
        let peer_id2 = vec![35; 32];
        let caller = add_onchain_node::<T>(peer_id, "report_file_miss", NODE_STATUS_ONCHAIN);
        let caller2 = add_onchain_node::<T>(peer_id2, "report_file_miss2", NODE_STATUS_ONCHAIN);
        let owner: T::AccountId = user_purchase_storage::<T>("owner");

        let file_id = vec![32; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let signature = vec![33; 1024];
        let tee_report = vec![35; 1024];

        let _ = Pallet::<T>::add_file_info(RawOrigin::Signed(caller.clone()).into(), owner.clone(), file_id.clone(), file_size, file_type, 1000u32.into(), signature);
        let _ = Pallet::<T>::add_file_peer(RawOrigin::Signed(caller2).into(), file_id.clone(), 1000u32.into());
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 2);
    }: _(RawOrigin::Signed(caller),file_id.clone(), file_type, 1000u32.into())
    verify {
        let info = Files::<T>::get(&file_id).unwrap();
        assert!(info.peers.len() == 1);
    }

    report_login_info_miss {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id.clone(), "report_login_info_miss", NODE_STATUS_ONCHAIN);
        let owner: T::AccountId = user_purchase_storage::<T>("owner");

        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), owner.clone(), 1000u32.into());
        let _ = Pallet::<T>::set_stash_peer(RawOrigin::Signed(caller.clone()).into(), owner.clone(), peer_id.clone());
        let info = WalletAccountStorage::<T>::get(owner.clone()).unwrap();
        assert!(info.peers.len() == 1);
    }: _(RawOrigin::Signed(caller), owner.clone())
    verify {
        let info = WalletAccountStorage::<T>::get(owner).unwrap();
        assert!(info.peers.len() == 0);
    }

    report_tee_faking {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id.clone(), "report_tee_faking", NODE_STATUS_ONCHAIN);
        let ext_height = 1000u32.into();
        let ext_num = 1;
        let ext_tee_report_hash = vec![36; 32];
        let ext_tee_report = vec![35; 1024];
        let block_height = 1000u32.into();
        let tee_report = vec![38; 1024];
    }: _(RawOrigin::Signed(caller), 
        peer_id.clone(), 
        ext_height, 
        ext_num, 
        ext_tee_report_hash.clone(),
        ext_tee_report.clone(),
        block_height,
        tee_report.clone()
    )
    verify {
        let report_info = ReportInfo {
            report_type: ReportType::ReportTeeFaking,
            peer_id: peer_id.to_vec(),
        };
        let era_index = T::StakingProvider::get_current_era_index();
        let accounts = <ReportsInEra<T>>::get(era_index, report_info).unwrap();
        assert!(accounts.len() == 1);
    }

    verify_tee_faking {
        let peer_id = vec![38; 32];
        let _ = add_onchain_node::<T>(peer_id.clone(), "faking_peer", NODE_STATUS_ONCHAIN);
        let caller = add_onchain_node::<T>(vec![33; 32], "verify_tee_faking", NODE_STATUS_ONCHAIN);
        let report1 = add_onchain_node::<T>(vec![32; 32], "report1", NODE_STATUS_ONCHAIN);
        let report2 = add_onchain_node::<T>(vec![31; 32], "report2", NODE_STATUS_ONCHAIN);
        
        let block_height = 1000u32.into();
        let tee_report = vec![38; 1024];

        let _ = Pallet::<T>::verify_tee_faking(
            RawOrigin::Signed(report1).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
        let _ = Pallet::<T>::verify_tee_faking(
            RawOrigin::Signed(report2).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
    }: _(RawOrigin::Signed(caller), 
        peer_id.clone(), 
        block_height,
        tee_report.clone()
    )
    verify {
        let report_info = ReportInfo {
            report_type: ReportType::VerifyTeeFaking,
            peer_id: peer_id.to_vec(),
        };
        let era_index = T::StakingProvider::get_current_era_index();
        let accounts = <ReportsInEra<T>>::get(era_index, report_info).unwrap();
        assert!(accounts.len() == 3);
        let info = Peers::<T>::get(peer_id).unwrap();
        assert!(info.status == NODE_STATUS_DISCARD);
    }

    report_peer_offchain {
        let peer_id = vec![38; 32];
        let _ = add_onchain_node::<T>(peer_id.clone(), "faking_peer", NODE_STATUS_ONCHAIN);
        let caller = add_onchain_node::<T>(vec![33; 32], "report_peer_offchain", NODE_STATUS_ONCHAIN);
        let report1 = add_onchain_node::<T>(vec![32; 32], "report1", NODE_STATUS_ONCHAIN);
        let report2 = add_onchain_node::<T>(vec![31; 32], "report2", NODE_STATUS_ONCHAIN);
        
        let block_height = 1000u32.into();
        let tee_report = vec![38; 1024];

        let _ = Pallet::<T>::report_peer_offchain(
            RawOrigin::Signed(report1).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
        let _ = Pallet::<T>::report_peer_offchain(
            RawOrigin::Signed(report2).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
    }: _(RawOrigin::Signed(caller), 
        peer_id.clone(), 
        block_height,
        tee_report.clone()
    )
    verify {
        let report_info = ReportInfo {
            report_type: ReportType::ReportPeerOffchain,
            peer_id: peer_id.to_vec(),
        };
        let era_index = T::StakingProvider::get_current_era_index();
        let accounts = <ReportsInEra<T>>::get(era_index, report_info).unwrap();
        assert!(accounts.len() == 3);
        let info = Peers::<T>::get(peer_id).unwrap();
        assert!(info.status == NODE_STATUS_OFFCHAIN);
    }

    report_peer_no_response {
        let peer_id = vec![38; 32];
        let _ = add_onchain_node::<T>(peer_id.clone(), "faking_peer", NODE_STATUS_ONCHAIN);
        let caller = add_onchain_node::<T>(vec![33; 32], "report_peer_no_response", NODE_STATUS_ONCHAIN);
        let report1 = add_onchain_node::<T>(vec![32; 32], "report1", NODE_STATUS_ONCHAIN);
        let report2 = add_onchain_node::<T>(vec![31; 32], "report2", NODE_STATUS_ONCHAIN);
        
        let block_height = 1000u32.into();
        let tee_report = vec![38; 1024];

        let _ = Pallet::<T>::report_peer_no_response(
            RawOrigin::Signed(report1).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
        let _ = Pallet::<T>::report_peer_no_response(
            RawOrigin::Signed(report2).into(), 
            peer_id.clone(), 
            block_height,
            tee_report.clone()
        );
    }: _(RawOrigin::Signed(caller), 
        peer_id.clone(), 
        block_height,
        tee_report.clone()
    )
    verify {
        let report_info = ReportInfo {
            report_type: ReportType::ReportPeerNoResponse,
            peer_id: peer_id.to_vec(),
        };
        let era_index = T::StakingProvider::get_current_era_index();
        let accounts = <ReportsInEra<T>>::get(era_index, report_info).unwrap();
        assert!(accounts.len() == 3);
        let info = Peers::<T>::get(peer_id).unwrap();
        assert!(info.status == NODE_STATUS_OFFCHAIN);
    }

    report_spam {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "report_spam", NODE_STATUS_ONCHAIN);
        let report_account = user_purchase_storage::<T>("report_account");
        let report_block_height = 1000u32.into();
        let report_signature = vec![35; 1024];
        let msg_id = vec![32; 32];
        let sender_account = user_purchase_storage::<T>("sender_account");
        let app_id = vec![31; 32];
        let msg_block_height = 0u32.into();
        let msg_encrypt = vec![37; 1024];
        let msg_signature = vec![38; 1024];

        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), report_account.clone(), 1000u32.into());
 
    }: _(RawOrigin::Signed(caller), report_account, report_block_height, report_signature, msg_id, sender_account.clone(), app_id, msg_block_height, msg_encrypt, msg_signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&sender_account).unwrap();
        assert!(info.spam_report_amount == 1);
    }

    user_login {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "user_login", NODE_STATUS_ONCHAIN);
        let login_account = user_purchase_storage::<T>("login_account");
        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), login_account.clone(), 1000u32.into());
        
        let n in 1 .. 1000 as u32;
        let mut app_ids = Vec::new();
        for i in 0 .. n {
            app_ids.push(vec![32; usize::try_from(i).unwrap()]);
            let _ = Pallet::<T>::set_app_account(RawOrigin::Signed(caller.clone()).into(), vec![32; usize::try_from(i).unwrap()], login_account.clone());
        }

        let mut user_info = WalletAccountStorage::<T>::get(&login_account).unwrap();
        user_info.spam_report_amount = 1;
        user_info.comment_report_amount = 1;
        WalletAccountStorage::<T>::insert(&login_account, user_info);

        frame_system::Pallet::<T>::set_block_number(frame_system::Pallet::<T>::block_number() + Pallet::<T>::interval_blocks_login());
    }: _(RawOrigin::Signed(caller), login_account.clone(), app_ids, Pallet::<T>::interval_blocks_login().saturated_into::<u32>())
    verify {
        let cur_info = WalletAccountStorage::<T>::get(&login_account).unwrap();
        assert!(cur_info.spam_report_amount == 0);
        assert!(cur_info.comment_report_amount == 0);
    }
    
    new_theme {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "new_theme", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let theme_id = vec![32; 32];
        let app_id = vec![31; 32];
        let comment_space = 2000000;
        let open_flag = 1;
        let block_height = 1000u32.into();
        let signature = vec![35; 1024];
 
        let pre_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
    }: _(RawOrigin::Signed(caller), for_account.clone(), theme_id, app_id, comment_space, open_flag, block_height, signature)
    verify {
        let cur_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(pre_info.comment_space+comment_space == cur_info.comment_space);
    }

    add_theme_comment_space {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_theme_comment_space", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let theme_id = vec![32; 32];
        let app_id = vec![31; 32];
        let add_space = 2000000;
        let block_height = 1000u32.into();
        let signature = vec![35; 1024];
 
        let pre_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
    }: _(RawOrigin::Signed(caller), for_account.clone(), theme_id, app_id, add_space, block_height, signature)
    verify {
        let cur_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(pre_info.comment_space+add_space == cur_info.comment_space);
    }

    add_user_comment_space {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "add_user_comment_space", NODE_STATUS_ONCHAIN);
        let for_account = user_purchase_storage::<T>("for_account");
        let block_height = 1000u32.into();
        let signature = vec![35; 1024];
 
        let pre_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
    }: _(RawOrigin::Signed(caller), for_account.clone(), block_height, signature)
    verify {
        let cur_info = WalletAccountStorage::<T>::get(&for_account).unwrap();
        assert!(pre_info.comment_space < cur_info.comment_space);
    }

    report_malicious_comment {
        let peer_id = vec![33; 32];
        let caller = add_onchain_node::<T>(peer_id, "report_spam", NODE_STATUS_ONCHAIN);
        let report_account = user_purchase_storage::<T>("report_account");
        let report_block_height = 14600u32.into();
        let report_signature = vec![35; 1024];
        let theme_id = vec![32; 32];
        let content_id = vec![33; 32];
        let comment_account = user_purchase_storage::<T>("comment_account");
        let app_id = vec![31; 32];
        let comment_block_height = 500u32.into();
        let refer_comment_key = vec![37; 1024];
        let content_type = 1;
        let comment_signature = vec![38; 1024];

        let _ = Pallet::<T>::add_user_peer(RawOrigin::Signed(caller.clone()).into(), report_account.clone(), 1000u32.into());
        frame_system::Pallet::<T>::set_block_number(14600u32.into());
    }: _(RawOrigin::Signed(caller), report_account, report_block_height, report_signature, theme_id, content_id, comment_account.clone(), app_id, comment_block_height, refer_comment_key, content_type, comment_signature)
    verify {
        let info = WalletAccountStorage::<T>::get(&comment_account).unwrap();
        assert!(info.comment_report_amount == 1);
    }

    impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test)
}
