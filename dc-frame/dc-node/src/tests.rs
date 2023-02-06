use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use frame_benchmarking::{whitelisted_caller, account};
use testing_utils::*;
use sp_runtime::SaturatedConversion;

#[test]
fn dc_join_storage_node() {
	new_test_ext().execute_with(|| {
		let peer_id = vec![31; 32];
        let total_space = DcNode::max_storage_node_space();
        let free_space = DcNode::max_storage_node_space();
        let ip_address = vec![33; 256];
        let report_number = 10293u32.into();
        let tee_report = vec![38; 512];
        let caller: AccountId = whitelisted_caller();
        
        assert_noop!(
            DcNode::join_storage_node(
                RuntimeOrigin::signed(caller.clone()), 
                peer_id.clone(), 
                total_space+1, 
                free_space, 
                ip_address.clone(), 
                report_number, 
                tee_report.clone()
            ), 
            Error::<Test>::MaxStorageNodeSize
        );

        assert_ok!(
            DcNode::join_storage_node(
                RuntimeOrigin::signed(caller.clone()), 
                peer_id.clone(), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                report_number, 
                tee_report.clone()
            )
        );
        
        assert_eq!(<Peers<Test>>::get(&peer_id).unwrap().status, NODE_STATUS_JOINING);

        let err_caller = user_purchase_storage::<Test>("err_caller");
        assert_noop!(
            DcNode::join_storage_node(
                RuntimeOrigin::signed(err_caller.clone()), 
                peer_id.clone(), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                report_number, 
                tee_report.clone()
            ), 
            Error::<Test>::PeerAccountError
        );

        System::set_block_number(DcNode::tee_report_verify_number());
		assert_ok!(
            DcNode::join_storage_node(
                RuntimeOrigin::signed(caller.clone()), 
                peer_id.clone(), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                report_number, 
                tee_report.clone()
            )
        );
        assert_eq!(<Peers<Test>>::get(&peer_id).unwrap().status, NODE_STATUS_JOINING);
        
		let mut pre_info = <Peers<Test>>::get(&peer_id).unwrap();
        pre_info.status = NODE_STATUS_STAKED;
        <Peers<Test>>::insert(&peer_id, pre_info);
        assert_ok!(
            DcNode::join_storage_node(
                RuntimeOrigin::signed(caller.clone()), 
                peer_id.clone(), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                report_number, 
                tee_report.clone()
            )
        );
        
        let cur_info = <Peers<Test>>::get(&peer_id).unwrap();
		assert_eq!(cur_info.reward_number, 2*DcNode::start_reward_block_number());
		assert_eq!(DcNode::request_account_peer(caller).unwrap(), peer_id);
	});
}

#[test]
fn dc_submit_work_report() {
	new_test_ext().execute_with(|| {
		let peer_id = vec![36; 32];
        let total_space: u64 = 100*1024*1024*1024*1024;
        let free_space: u64 = 100*1024*1024*1024*1024;
        let ip_address = vec![36; 256];
        
        let caller = add_onchain_node::<Test>(peer_id.clone(), "submit_work_report", NODE_STATUS_ONCHAIN);		
        let owner = user_purchase_storage::<Test>("owner");

        let mut miss_files = Vec::new();
        miss_files.push(vec![49,50,51]);
        miss_files.push(vec![50,51,53]);
        let mut miss_accounts = Vec::new();
        let temp_acc = user_purchase_storage::<Test>("miss account");
        miss_accounts.push(temp_acc.clone());
        let _ = DcNode::add_user_peer(RuntimeOrigin::signed(caller.clone()), temp_acc.clone(), 1000u32.into());
        let _ = DcNode::set_stash_peer(RuntimeOrigin::signed(caller.clone()), temp_acc.clone(), peer_id.clone());

        let file_id = vec![49, 50, 51]; 
        let t_file_id = vec![50, 51, 53];
        let file_size = 1000123; 
        let file_type = 1;
        let tee_report = vec![33; 1024];
        let signature = vec![35; 1024];
		
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(), 
                file_size, 
                file_type, 
                1000u32.into(), 
                signature.clone()
            )
        );
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                t_file_id.clone(), 
                file_size, 
                file_type, 
                1000u32.into(), 
                signature
            )
        );
        System::set_block_number(System::block_number() + DcNode::interval_blocks_work_report());
        let report_number = frame_system::Pallet::<Test>::block_number();
        assert_eq!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(), 
                miss_accounts.clone(),
                report_number.saturated_into(), 
                tee_report.clone()
            ),
            Ok(Pays::No.into())
        );

        assert_eq!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(), 
                miss_accounts.clone(),
                report_number.saturated_into(), 
                tee_report.clone()
            ),
            Ok(Pays::Yes.into())
        );

		let pre_info = <Peers<Test>>::get(&peer_id).unwrap();
		assert_eq!(pre_info.reward_number, frame_system::Pallet::<Test>::block_number()+DcNode::start_reward_block_number());

        // test check_request_block_number error 
        System::set_block_number(System::block_number() + 300);
        assert_noop!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(),  
                miss_accounts.clone(),
                99, 
                tee_report.clone()
            ), 
            Error::<Test>::BlockNumberInvalid
        );
        assert_noop!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(),  
                miss_accounts.clone(),
                301, 
                tee_report.clone()
            ), 
            Error::<Test>::BlockNumberInvalid
        );

        // test get_peer_id_with_req_acc_id error
        let t_peer_id = vec![32; 32];
        let t_caller: AccountId = account("t_caller", 0, 0);
        System::set_block_number(150);
        assert_noop!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(t_caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(),  
                miss_accounts.clone(),
                150, 
                tee_report.clone()
            ), 
            Error::<Test>::AccountNotExist
        );
        <RequestAccountPeer<Test>>::insert(&t_caller, &t_peer_id);
        assert_noop!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(t_caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(),  
                miss_accounts.clone(),
                150, 
                tee_report.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
        let node_info = StorageNode {
            req_account: t_caller.clone(),
            stash: t_caller.clone(),
            total_space: 0,
            free_space: 0,
            status: NODE_STATUS_JOINING,
            report_number: 0,
            staked_number: 0,
            reward_number: 0,
            ip_address: ip_address.clone(),
        };
        Peers::<Test>::insert(&t_peer_id, node_info);
        assert_noop!(
            DcNode::submit_work_report(
                RuntimeOrigin::signed(t_caller.clone()), 
                total_space, 
                free_space, 
                ip_address.clone(), 
                miss_files.clone(),  
                miss_accounts.clone(),
                150, 
                tee_report.clone()
            ), 
            Error::<Test>::NodeStatusError
        );
	});
}

#[test]
fn dc_set_stash_peer() {
	new_test_ext().execute_with(|| {
		let peer_id = vec![36; 32];
        let t_caller: AccountId = account("t_caller", 0, 0);
        let stash: AccountId = account("stash", 0, 0);

        assert_noop!(
            DcNode::set_stash_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                stash.clone(),
                peer_id.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
		
        let _ = add_onchain_node::<Test>(peer_id.clone(), "set_stash_peer", NODE_STATUS_OFFCHAIN);		
        assert_ok!(
            DcNode::set_stash_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                stash.clone(),
                peer_id.clone()
            )
        );

        assert_eq!(<StashPeers<Test>>::get(&stash).unwrap().contains(&peer_id), true);
        let pre_info = <Peers<Test>>::get(&peer_id).unwrap();
		assert_eq!(pre_info.status, NODE_STATUS_ONCHAIN);
    });
}

#[test]
fn dc_remove_stash_peer() {
	new_test_ext().execute_with(|| {
		let peer_id = vec![36; 32];
        let t_caller: AccountId = account("t_caller", 0, 0);
        let stash: AccountId = account("stash", 0, 0);

        assert_noop!(
            DcNode::remove_stash_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                stash.clone(),
                peer_id.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
		
        let _ = add_onchain_node::<Test>(peer_id.clone(), "remove_stash_peer", NODE_STATUS_OFFCHAIN);
        assert_ok!(
            DcNode::set_stash_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                stash.clone(),
                peer_id.clone()
            )
        );
        assert_eq!(<StashPeers<Test>>::get(&stash).unwrap().contains(&peer_id), true);		
        assert_ok!(
            DcNode::remove_stash_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                stash.clone(),
                peer_id.clone()
            )
        );

        assert_eq!(<StashPeers<Test>>::get(&stash).unwrap().len(), 0);
        let pre_info = <Peers<Test>>::get(&peer_id).unwrap();
		assert_eq!(pre_info.stash, <Test as Config>::DefaultAccountId::get());
    });
}

#[test]
fn dc_purchase_storage() {
	new_test_ext().execute_with(|| {
        let caller: AccountId = account("caller", 0, 0);

        assert_noop!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                1
            ), 
            Error::<Test>::StoragePackageNotExist
        );
		
        assert_ok!(DcNode::set_storage_package(RuntimeOrigin::root(), 1, 100, 100, 2, 100));
        assert_noop!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                1
            ), 
            Error::<Test>::InsufficientBalance
        );
        let balance: Balance = 3000_000_000_003_000;
        let _ = Balances::make_free_balance_be(&caller, balance);

        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                1
            ), 
        );
        assert_eq!(DcNode::wallet_account_storage(caller.clone()).unwrap().expire_number, 100);
        assert_ok!(DcNode::set_storage_package(RuntimeOrigin::root(), 2, 200, 200, 1, 200));
        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                2
            ), 
        );

        assert_eq!(DcNode::wallet_account_storage(caller.clone()).unwrap().expire_number, 250);
        System::set_block_number(50);
        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                1
            ), 
        );                            
        assert_eq!(DcNode::wallet_account_storage(caller.clone()).unwrap().expire_number, 550);

        System::set_block_number(150);
        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                2
            ), 
        );                            
        assert_eq!(DcNode::wallet_account_storage(caller.clone()).unwrap().expire_number, 550);

        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                2
            ), 
        );                            
        assert_eq!(DcNode::wallet_account_storage(caller.clone()).unwrap().expire_number, 750);

        let file_id = vec![49, 50, 51]; 
        let file_size = 150; 
        let file_type = 1;
        let signature = vec![35; 1024];
		
        let t_caller = add_onchain_node::<Test>(vec![33; 32], "dc_purchase_storage", NODE_STATUS_ONCHAIN);

        System::set_block_number(0);
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(t_caller.clone()), 
                caller.clone(), 
                file_id.clone(), 
                file_size, 
                file_type, 
                10u32.into(), 
                signature.clone()
            )
        );
        assert_noop!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                caller.clone(),
                1
            ), 
            Error::<Test>::ParamErr
        );
    });
}

#[test]
fn dc_update_db_config() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let for_account: AccountId = account("for_account", 0, 0);
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "update_db_config", NODE_STATUS_ONCHAIN);
        
        // Test check_peer_request_with_account
        assert_noop!(
            DcNode::update_db_config(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                vec![32; 32],
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );
		
        System::set_block_number(300);
        assert_ok!(DcNode::set_storage_package(RuntimeOrigin::root(), 1, 100, 100, 1, 100));
        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                1
            ), 
        );
        assert_noop!(
            DcNode::update_db_config(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                vec![32; 32],
                32u32.into(),
                vec![33; 1024]
            ),
            Error::<Test>::BlockNumberInvalid
        );

        System::set_block_number(1);
        // Test other logics
        let other_account: AccountId = account("other_account", 0, 0);
        assert_noop!(
            DcNode::update_db_config(
                RuntimeOrigin::signed(caller.clone()), 
                other_account.clone(),
                vec![32; 32],
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_ok!(DcNode::set_storage_package(RuntimeOrigin::root(), 1, 100, 100, 1, 100));
        assert_ok!(
            DcNode::purchase_storage(
                RuntimeOrigin::signed(caller.clone()), 
                other_account.clone(),
                1
            ), 
        );
        assert_ok!(
            DcNode::update_db_config(
                RuntimeOrigin::signed(caller.clone()), 
                other_account.clone(),
                vec![33; 32],
                33u32.into(),
                vec![33; 1024]
            ),  
        );
        assert_eq!(DcNode::wallet_account_storage(other_account.clone()).unwrap().db_update_number, 33);
        assert_eq!(DcNode::wallet_account_storage(other_account.clone()).unwrap().db_config, vec![33; 32]);
    });
}

#[test]
fn dc_create_sub_account() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let parent_account: AccountId = account("parent_account", 0, 0);
        let sub_account: AccountId = user_purchase_storage::<Test>("sub_account");
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "create_sub_account", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::create_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                parent_account.clone(),
                sub_account.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountAlreadyExist
        );
		
        let sub_account2: AccountId = account("sub2_account", 0, 0);
        assert_noop!(
            DcNode::create_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                parent_account.clone(),
                sub_account2.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );

        let p_account: AccountId = user_purchase_storage::<Test>("p_account");
        let s_account: AccountId = account("s_account", 0, 0);
        let _ = DcNode::create_sub_account(RuntimeOrigin::signed(caller.clone()), p_account.clone(), s_account.clone(), 10u32.into(), vec![33; 1024]);
        assert_noop!(
            DcNode::create_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                s_account.clone(),
                sub_account2.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::IsSubAccount
        );

        assert_ok!(
            DcNode::create_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                p_account.clone(),
                sub_account2.clone(),
                32u32.into(),
                vec![33; 1024]
            )
        );
        assert_eq!(DcNode::wallet_account_storage(sub_account2.clone()).unwrap().parent_account, p_account);
    });
}

#[test]
fn dc_unbind_sub_account() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let parent_account: AccountId = account("parent_account", 0, 0);
        let sub_account: AccountId = user_purchase_storage::<Test>("sub_account");
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "unbind_sub_account", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::unbind_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                parent_account.clone(),
                sub_account.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );
		
        let sub_account2: AccountId = user_purchase_storage::<Test>("sub_account2");
        let p_account: AccountId = user_purchase_storage::<Test>("p_account");
        let s_account: AccountId = account("s_account", 0, 0);
        let _ = DcNode::create_sub_account(RuntimeOrigin::signed(caller.clone()), p_account.clone(), s_account.clone(), 10u32.into(), vec![33; 1024]);
        assert_noop!(
            DcNode::unbind_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                s_account.clone(),
                sub_account2.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::IsSubAccount
        );

        let sub_no_account: AccountId = account("sub_no_account", 0, 0);
        assert_noop!(
            DcNode::unbind_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                p_account.clone(),
                sub_no_account.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );

        let pt_account: AccountId = user_purchase_storage::<Test>("pt_account");
        let st_account: AccountId = account("st_account", 0, 0);
        let _ = DcNode::create_sub_account(RuntimeOrigin::signed(caller.clone()), pt_account.clone(), st_account.clone(), 10u32.into(), vec![33; 1024]);
        assert_noop!(
            DcNode::unbind_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                pt_account.clone(),
                sub_account2.clone(),
                32u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::ParamErr
        );

        assert_ok!(
            DcNode::unbind_sub_account(
                RuntimeOrigin::signed(caller.clone()), 
                pt_account.clone(),
                st_account.clone(),
                32u32.into(),
                vec![33; 1024]
            )
        );
        assert_eq!(DcNode::wallet_account_storage(st_account.clone()).unwrap().parent_account, st_account);
    });
}

#[test]
fn dc_add_user_peer() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let for_account: AccountId = account("for_account", 0, 0);
        let ok_account: AccountId = user_purchase_storage::<Test>("ok_account");
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_user_peer", NODE_STATUS_ONCHAIN);
        assert_noop!(
            DcNode::add_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                32u32.into()
            ), 
            Error::<Test>::AccountNotExist
        );
        
        assert_ok!(
            DcNode::add_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                ok_account.clone(),
                32u32.into()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(ok_account.clone()).unwrap().peers.contains(&peer_id), true);
    });
}

#[test]
fn dc_remove_self_user_peer() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let for_account: AccountId = account("for_account", 0, 0);
        let ok_account: AccountId = user_purchase_storage::<Test>("ok_account");
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "remove_self_user_peer", NODE_STATUS_ONCHAIN);
        assert_noop!(
            DcNode::remove_self_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                32u32.into()
            ), 
            Error::<Test>::AccountNotExist
        );
        
        assert_ok!(
            DcNode::add_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                ok_account.clone(),
                32u32.into()
            )
        );
        assert_ok!(
            DcNode::remove_self_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                ok_account.clone(),
                32u32.into()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(ok_account.clone()).unwrap().peers.contains(&peer_id), false);
    });
}

#[test]
fn dc_remove_other_user_peer() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let peer_remove_id = vec![58; 32];
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
		
        let caller = add_onchain_node::<Test>(peer_id.clone(), "remove_other_user_peer", NODE_STATUS_ONCHAIN);
        assert_noop!(
            DcNode::remove_other_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                peer_remove_id.clone(),
                for_account.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
        
        let _ = add_onchain_node::<Test>(peer_remove_id.clone(), "remove_other_user_peer", NODE_STATUS_ONCHAIN);
        assert_noop!(
            DcNode::remove_other_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                peer_remove_id.clone(),
                for_account.clone()
            ), 
            Error::<Test>::NodeStatusError
        );

        let peer_ok_id = vec![58; 32];
        let t_caller = add_onchain_node::<Test>(peer_ok_id.clone(), "remove_other_user_peer", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::add_user_peer(
                RuntimeOrigin::signed(t_caller.clone()), 
                for_account.clone(),
                32u32.into()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().peers.contains(&peer_ok_id), true);

        let mut node_info = Peers::<Test>::take(&peer_ok_id).unwrap();
        node_info.status = NODE_STATUS_ABNORMAL;
        Peers::<Test>::insert(&peer_ok_id, node_info);
        assert_ok!(
            DcNode::remove_other_user_peer(
                RuntimeOrigin::signed(caller.clone()), 
                peer_ok_id.clone(),
                for_account.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().peers.contains(&peer_ok_id), false);
    });
}

#[test]
fn dc_apply_nft_account() {
	new_test_ext().execute_with(|| {
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
        let nft_account = vec![33; 32];
        let enc_nft_account = vec![32; 32];
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "apply_nft_account", NODE_STATUS_ONCHAIN);

        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().used_space, 10000);
        assert_ok!(
            DcNode::apply_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                for_account.clone(),
                enc_nft_account.clone(),
                vec![38; 32],
                1001u32.into(),
                vec![33; 1024]
            )
        );
        let pre_expire_number: u64 = frame_system::Pallet::<Test>::block_number()+10000u64;
        let user_info = DcNode::wallet_account_storage(for_account.clone()).unwrap();
        assert_eq!(user_info.peers.len(), 1);
        assert_eq!(user_info.peers.contains(&peer_id), true);
        assert_eq!(user_info.expire_number, pre_expire_number - user_info.call_minus_number);
        assert_eq!(user_info.enc_nft_account, enc_nft_account);
        assert_noop!(
            DcNode::apply_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                for_account.clone(),
                enc_nft_account.clone(),
                vec![38; 32],
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::NftAccoutApplied
        );
        
        let f_account: AccountId = account("f_account", 0, 0);
        assert_noop!(
            DcNode::apply_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                f_account.clone(),
                enc_nft_account.clone(),
                vec![38; 32],
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );
    });
}

#[test]
fn dc_transfer_nft_account() {
	new_test_ext().execute_with(|| {
        let from_account: AccountId = user_purchase_storage::<Test>("from_account");
        let to_account: AccountId = account("to_account", 0, 0);
        let nft_account = vec![33; 32];
        let enc_nft_account = vec![32; 32];
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "transfer_nft_account", NODE_STATUS_ONCHAIN);

        assert_noop!(
            DcNode::transfer_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                from_account.clone(),
                to_account.clone(),
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::NftAccoutApplied
        );
        
        assert_ok!(
            DcNode::apply_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                from_account.clone(),
                enc_nft_account.clone(),
                vec![38; 32],
                1001u32.into(),
                vec![33; 1024]
            )
        );

        assert_noop!(
            DcNode::transfer_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                from_account.clone(),
                to_account.clone(),
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );
        
        let t_account: AccountId = user_purchase_storage::<Test>("t_account");
        assert_ok!(
            DcNode::transfer_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                from_account.clone(),
                t_account.clone(),
                1001u32.into(),
                vec![33; 1024]
            )
        );
        
        assert_eq!(DcNode::wallet_account_storage(from_account.clone()).unwrap().peers.len(), 0);
        assert_eq!(DcNode::wallet_account_storage(from_account.clone()).unwrap().enc_nft_account, NftAccount::new());
    });
}

#[test]
fn dc_update_nft_account() {
	new_test_ext().execute_with(|| {
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
        let nft_account = vec![33; 32];
        let enc_nft_account = vec![32; 32];
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "update_nft_account", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::update_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                for_account.clone(),
                enc_nft_account.clone(),
                vec![32; 32],
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::NftAccoutApplied
        );

        assert_ok!(
            DcNode::apply_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                for_account.clone(),
                enc_nft_account.clone(),
                vec![38; 32],
                1001u32.into(),
                vec![33; 1024]
            )
        );
        
        let f_account: AccountId = user_purchase_storage::<Test>("f_account");
        assert_eq!(DcNode::wallet_account_storage(f_account.clone()).unwrap().used_space, 10000);

        assert_ok!(
            DcNode::transfer_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                for_account.clone(),
                f_account.clone(),
                1001u32.into(),
                vec![33; 1024]
            )
        );
        assert_ok!(
            DcNode::update_nft_account(
                RuntimeOrigin::signed(caller.clone()), 
                nft_account.clone(), 
                f_account.clone(),
                enc_nft_account.clone(),
                vec![32; 32],
                1001u32.into(),
                vec![33; 1024]
            )
        );
        
        assert_eq!(DcNode::wallet_account_storage(f_account.clone()).unwrap().nft_update_number, 1001);
        assert_eq!(DcNode::wallet_account_storage(f_account.clone()).unwrap().enc_nft_account, enc_nft_account);
    });
}

#[test]
fn dc_add_file_info() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = account("owner", 0, 0);

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_file_info", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::AccountNotExist
        );
        
        let pt_account: AccountId = user_purchase_storage::<Test>("pt_account");
        let st_account: AccountId = account("st_account", 0, 0);
        let _ = DcNode::create_sub_account(RuntimeOrigin::signed(caller.clone()), pt_account.clone(), st_account.clone(), 10u32.into(), vec![33; 1024]);

        assert_eq!(DcNode::wallet_account_storage(pt_account.clone()).unwrap().used_space, 10000);

        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                pt_account.clone(), 
                file_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );
        
        assert_eq!(DcNode::wallet_account_storage(pt_account.clone()).unwrap().used_space, 10000+1000123);

        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                st_account.clone(), 
                file_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::wallet_account_storage(pt_account.clone()).unwrap().used_space, 10000+1000123*2);
        assert_eq!(DcNode::wallet_account_storage(st_account.clone()).unwrap().used_space, 1000123);
        assert_eq!(DcNode::files(&file_id).unwrap().users.len(), 2);
        assert_eq!(DcNode::files(&file_id).unwrap().peers.len(), 1);
    });
}

#[test]
fn dc_add_file_peer() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = user_purchase_storage::<Test>("owner");

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_file_peer", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::add_file_peer(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                1001u32.into()
            ), 
            Error::<Test>::FileNotExist
        );
        
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                file_size,
                2,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_noop!(
            DcNode::add_file_peer(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                1001u32.into()
            ), 
            Error::<Test>::FileTypeError
        );

        let ok_id = vec![55; 32];

        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                ok_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );
        assert_eq!(DcNode::files(&ok_id).unwrap().peers.len(), 1);
        assert_ok!(
            DcNode::add_file_peer(
                RuntimeOrigin::signed(caller.clone()), 
                ok_id.clone(),
                1001u32.into()
            ),
        );

        assert_eq!(DcNode::files(&ok_id).unwrap().peers.len(), 1);
        assert_ok!(
            DcNode::add_file_peer(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![66; 32], "file_peer", NODE_STATUS_ONCHAIN)), 
                ok_id.clone(),
                1001u32.into()
            ),
        );
        assert_eq!(DcNode::files(&ok_id).unwrap().peers.contains(&vec![66; 32]), true);
        assert_eq!(DcNode::files(&ok_id).unwrap().peers.len(), 2);
    });
}

#[test]
fn dc_remove_self_file_peer() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = user_purchase_storage::<Test>("owner");

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "remove_self_file_peer", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::remove_self_file_peer(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                1u32,
                1001u32.into()
            ), 
            Error::<Test>::FileNotExist
        );
        
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_ok!(
            DcNode::add_file_peer(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![66; 32], "file_peer", NODE_STATUS_ONCHAIN)), 
                file_id.clone(),
                1001u32.into()
            ),
        );

        assert_eq!(DcNode::files(&file_id).unwrap().peers.len(), 2);
        assert_ok!(
            DcNode::remove_self_file_peer(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                1u32,
                1001u32.into()
            ),
        );

        assert_eq!(DcNode::files(&file_id).unwrap().peers.len(), 1);
        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000+1000123);
        assert_ok!(
            DcNode::remove_self_file_peer(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![66; 32], "file_peer", NODE_STATUS_ONCHAIN)), 
                file_id.clone(),
                1u32,
                1001u32.into()
            ),
        );
        assert_eq!(DcNode::files(&file_id).is_none(), true);
        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000);
    });
}

#[test]
fn dc_delete_file_info() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = user_purchase_storage::<Test>("owner");

        let file_id = vec![37; 32]; 
        let file_size = 1000123; 
        let file_type = 1;
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "delete_file_info", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::delete_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(),
                file_id.clone(),
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::FileNotExist
        );
        
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                file_size,
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000+1000123);
        assert_ok!(
            DcNode::delete_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(),
                file_id.clone(),
                file_type,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::files(&file_id).is_none(), true);
        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000);
    });
}

#[test]
fn dc_add_log_to_thread_db() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = user_purchase_storage::<Test>("owner");

        let file_id = vec![37; 32]; 
        let log_id = vec![33; 32];
        let peer_id = vec![56; 32];

        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_log_to_thread_db", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::add_log_to_thread_db(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                log_id.clone(),
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::FileNotExist
        );
        
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                1000123,
                2,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000+1000123);
        assert_ok!(
            DcNode::add_log_to_thread_db(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                log_id.clone(),
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::files(&file_id).unwrap().db_log.contains(&log_id), true);
        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000+1000123);
    });
}

#[test]
fn dc_add_space_to_thread_db() {
	new_test_ext().execute_with(|| {
        let owner: AccountId = user_purchase_storage::<Test>("owner");

        let file_id = vec![37; 32]; 
        let peer_id = vec![56; 32];
        let increase_size = 3782;

        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_space_to_thread_db", NODE_STATUS_ONCHAIN);
        
        assert_noop!(
            DcNode::add_space_to_thread_db(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                increase_size,
                1001u32.into(),
                vec![33; 1024]
            ), 
            Error::<Test>::FileNotExist
        );
        
        assert_ok!(
            DcNode::add_file_info(
                RuntimeOrigin::signed(caller.clone()), 
                owner.clone(), 
                file_id.clone(),
                1000123,
                2,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, 10000+1000123);
        assert_ok!(
            DcNode::add_space_to_thread_db(
                RuntimeOrigin::signed(caller.clone()), 
                file_id.clone(),
                increase_size,
                1001u32.into(),
                vec![33; 1024]
            ),
        );

        assert_eq!(DcNode::files(&file_id).unwrap().file_size, u64::try_from(1000123+increase_size).unwrap());
        assert_eq!(DcNode::wallet_account_storage(owner.clone()).unwrap().used_space, u64::try_from(10000+1000123+increase_size).unwrap());
    });
}

#[test]
fn dc_report_tee_faking() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "report_tee_faking", NODE_STATUS_ONCHAIN);
        let ext_height = 201;
        let ext_num = 32;
        let ext_tee_report_hash = vec![33; 1024];
        let ext_tee_report = vec![33; 1024];
        let block_height = 201;
        let tee_report = vec![33; 1024];
        let report_p_id = vec![36; 32];
        assert_noop!(
            DcNode::report_tee_faking(
                RuntimeOrigin::signed(caller.clone()), 
                report_p_id.clone(),
                ext_height,
                ext_num,
                ext_tee_report_hash.clone(),
                ext_tee_report.clone(),
                block_height,
                tee_report.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
        
        let _ = add_onchain_node::<Test>(report_p_id.clone(), "report_p_id", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::report_tee_faking(
                RuntimeOrigin::signed(caller.clone()), 
                report_p_id.clone(),
                ext_height,
                ext_num,
                ext_tee_report_hash.clone(),
                ext_tee_report.clone(),
                block_height,
                tee_report.clone()
            )
        );

        let report_info = ReportInfo {
            report_type: ReportType::ReportTeeFaking,
            peer_id: report_p_id.clone(),
        };
        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 1);

        assert_ok!(
            DcNode::report_tee_faking(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![61; 32], "file_peer1", NODE_STATUS_ONCHAIN)), 
                report_p_id.clone(),
                ext_height,
                ext_num,
                ext_tee_report_hash.clone(),
                ext_tee_report.clone(),
                block_height,
                tee_report.clone()
            )
        );
        let caller2 = add_onchain_node::<Test>(vec![62; 32], "file_peer2", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::report_tee_faking(
                RuntimeOrigin::signed(caller2.clone()), 
                report_p_id.clone(),
                ext_height,
                ext_num,
                ext_tee_report_hash.clone(),
                ext_tee_report.clone(),
                block_height,
                tee_report.clone()
            )
        );
        assert_noop!(
            DcNode::report_tee_faking(
                RuntimeOrigin::signed(caller2.clone()), 
                report_p_id.clone(),
                ext_height,
                ext_num,
                ext_tee_report_hash.clone(),
                ext_tee_report.clone(),
                block_height,
                tee_report.clone()
            ),
            Error::<Test>::ErrorNodeReport
        );

        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 3);
    });
}

#[test]
fn dc_verify_tee_faking() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "verify_tee_faking", NODE_STATUS_ONCHAIN);
        let block_height = 201;
        let tee_report = vec![33; 1024];
        let report_p_id = vec![36; 32];
        assert_noop!(
            DcNode::verify_tee_faking(
                RuntimeOrigin::signed(caller.clone()), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );
        
        let _ = add_onchain_node::<Test>(report_p_id.clone(), "report_p_id", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::verify_tee_faking(
                RuntimeOrigin::signed(caller.clone()), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );

        let report_info = ReportInfo {
            report_type: ReportType::VerifyTeeFaking,
            peer_id: report_p_id.clone(),
        };
        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 1);

        assert_ok!(
            DcNode::verify_tee_faking(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![61; 32], "file_peer1", NODE_STATUS_ONCHAIN)), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );
        assert_ok!(
            DcNode::verify_tee_faking(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![62; 32], "file_peer2", NODE_STATUS_ONCHAIN)), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );

        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 3);
        assert_eq!(DcNode::peers(&report_p_id).unwrap().status, NODE_STATUS_DISCARD);
    });
}

#[test]
fn dc_report_peer_offchain() {
	new_test_ext().execute_with(|| {
        System::set_block_number(100);
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "report_peer_offchain", NODE_STATUS_ONCHAIN);
        let block_height = 201;
        let tee_report = vec![33; 1024];
        let report_p_id = vec![36; 32];
        
        let _ = add_onchain_node::<Test>(report_p_id.clone(), "report_p_id", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(caller.clone()), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );

        let report_info = ReportInfo {
            report_type: ReportType::ReportPeerOffchain,
            peer_id: report_p_id.clone(),
        };
        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 1);

        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![61; 32], "file_peer1", NODE_STATUS_ONCHAIN)), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );
        let cur_block_num = 200;
        System::set_block_number(cur_block_num);
        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![62; 32], "file_peer2", NODE_STATUS_ONCHAIN)), 
                report_p_id.clone(),
                block_height,
                tee_report.clone()
            )
        );

        assert_eq!(DcNode::reports_in_era(1, report_info.clone()).unwrap().len(), 3);
        assert_eq!(DcNode::peers(&report_p_id).unwrap().status, NODE_STATUS_OFFCHAIN);
        assert_eq!(DcNode::peers(&report_p_id).unwrap().reward_number, cur_block_num+DcNode::start_reward_block_number());

        System::set_block_number(100);
        let report_pr_id = vec![38; 32];
        let _ = add_onchain_node::<Test>(report_pr_id.clone(), "report_pr_id", NODE_STATUS_ONCHAIN);
        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(caller.clone()), 
                report_pr_id.clone(),
                block_height,
                tee_report.clone()
            )
        );

        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![61; 32], "file_peer1", NODE_STATUS_ONCHAIN)), 
                report_pr_id.clone(),
                block_height,
                tee_report.clone()
            )
        );
        System::set_block_number(50);
        assert_ok!(
            DcNode::report_peer_offchain(
                RuntimeOrigin::signed(add_onchain_node::<Test>(vec![62; 32], "file_peer2", NODE_STATUS_ONCHAIN)), 
                report_pr_id.clone(),
                block_height,
                tee_report.clone()
            )
        );
        assert_eq!(DcNode::peers(&report_pr_id).unwrap().reward_number, 100+DcNode::start_reward_block_number());
    });
}

#[test]
fn dc_report_peer_error() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "report_peer_error", NODE_STATUS_ONCHAIN);
        let block_height = 201;
        let tee_report = vec![33; 1024];

        assert_noop!(
            DcNode::report_peer_error(
                RuntimeOrigin::signed(caller.clone()), 
                vec![33; 32],
                block_height,
                tee_report.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );

        let rpt_p_id = vec![58; 32];
        let _ = add_onchain_node::<Test>(rpt_p_id.clone(), "rpt_p_id", NODE_STATUS_OFFCHAIN);
        System::set_block_number(DcNode::blocks_of_offchain_to_abnormal()+1);
        assert_ok!(
            DcNode::report_peer_error(
                RuntimeOrigin::signed(caller.clone()), 
                rpt_p_id.clone(),
                u32::try_from(DcNode::blocks_of_offchain_to_abnormal()+1).unwrap(),
                tee_report.clone()
            )
        );
        assert_eq!(DcNode::peers(&rpt_p_id).unwrap().status, NODE_STATUS_ABNORMAL);
    });
}

#[test]
fn dc_report_spam() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "report_spam", NODE_STATUS_ONCHAIN);
        let report_account: AccountId = user_purchase_storage::<Test>("report_account");
        let report_block_height = 12u32;
        let report_signature = vec![57; 1024];
        let msg_id = vec![51; 32];
        let sender_account: AccountId = user_purchase_storage::<Test>("sender_account");
        let app_id = vec![52; 32];
        let msg_block_height = 10u32;
        let msg_encrypt = vec![58; 1024];
        let msg_signature = vec![59; 1024];

        System::set_block_number(DcNode::interval_blocks_can_not_report()+30u64);
        assert_ok!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                sender_account.clone(),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_report_amount, 0);

        System::set_block_number(0u32.into());
        assert_noop!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                account("t_caller", 0, 0),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                sender_account.clone(),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_noop!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                sender_account.clone(),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );

        let _ = DcNode::add_user_peer(
            RuntimeOrigin::signed(caller.clone()), 
            report_account.clone(),
            0u32.into()
        );
        assert_noop!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                account("t_caller", 0, 0),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        System::set_block_number(30u32.into());
        assert_ok!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                sender_account.clone(),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_report_number, 30);
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_report_amount, 1);

        let mut user_info = WalletAccountStorage::<Test>::get(&sender_account).unwrap();
        user_info.spam_report_amount = DcNode::frozen_report_spam_amount()-1;
        WalletAccountStorage::<Test>::insert(&sender_account, user_info);

        System::set_block_number(100u32.into());
        assert_ok!(
            DcNode::report_spam(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                msg_id.clone(),
                sender_account.clone(),
                app_id.clone(),
                msg_block_height, 
                msg_encrypt.clone(),
                msg_signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_report_amount, DcNode::frozen_report_spam_amount());
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_frozen_status, 1);
        assert_eq!(DcNode::wallet_account_storage(sender_account.clone()).unwrap().spam_report_number, 30);
    });
}

#[test]
fn dc_set_app_account() {
    new_test_ext().execute_with(|| {
        let app_id = vec![56; 32];
        let caller: AccountId = user_purchase_storage::<Test>("caller");
        let rewarded_account: AccountId = account("rewarded_account", 0, 0);

        assert_noop!(
            DcNode::set_app_account(
                RuntimeOrigin::signed(caller.clone()), 
                vec![56; 33],
                rewarded_account.clone()
            ), 
            Error::<Test>::AppIdLengthErr
        );

        assert_ok!(
            DcNode::set_app_account(
                RuntimeOrigin::signed(caller.clone()), 
                app_id.clone(),
                rewarded_account.clone()
            )
        );
        assert_eq!(DcNode::account_of_app(&app_id).unwrap().private_account, caller);
        assert_eq!(DcNode::account_of_app(&app_id).unwrap().rewarded_stash, rewarded_account);

        let rewarded_acc: AccountId = account("rewarded_acc", 0, 0);
        let caller2: AccountId = user_purchase_storage::<Test>("caller2");
        assert_noop!(
            DcNode::set_app_account(
                RuntimeOrigin::signed(caller2.clone()), 
                app_id.clone(),
                rewarded_acc.clone()
            ),
            Error::<Test>::NotController
        );

        assert_ok!(
            DcNode::set_app_account(
                RuntimeOrigin::signed(caller.clone()), 
                app_id.clone(),
                rewarded_acc.clone()
            )
        );
        assert_eq!(DcNode::account_of_app(&app_id).unwrap().rewarded_stash, rewarded_acc);
    });
}

#[test]
fn dc_user_login() {
    new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "user_login", NODE_STATUS_ONCHAIN);
        let login_account: AccountId = user_purchase_storage::<Test>("login_account");
        let mut app_ids = Vec::new();
        app_ids.push(vec![49,50,51]);
        app_ids.push(vec![50,51,53]);
        let block_height = 0u32;

        assert_noop!(
            DcNode::user_login(
                RuntimeOrigin::signed(caller.clone()), 
                account("AccountNotExist", 0, 0),
                app_ids.clone(),
                block_height
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_noop!(
            DcNode::user_login(
                RuntimeOrigin::signed(caller.clone()), 
                login_account.clone(),
                app_ids.clone(),
                block_height
            ), 
            Error::<Test>::PeerIdNotExist
        );

        let _ = DcNode::add_user_peer(
            RuntimeOrigin::signed(caller.clone()), 
            login_account.clone(),
            0u32.into()
        );

        let mut user_info = WalletAccountStorage::<Test>::get(&login_account).unwrap();
        user_info.spam_report_amount = 10;
        user_info.comment_report_amount = 10;
        WalletAccountStorage::<Test>::insert(&login_account, user_info);
        System::set_block_number(System::block_number() + DcNode::interval_blocks_login());
        let _ = DcNode::set_app_account(RuntimeOrigin::signed(caller.clone()), vec![50,51,53], login_account.clone());
        assert_ok!(
            DcNode::user_login(
                RuntimeOrigin::signed(caller.clone()), 
                login_account.clone(),
                app_ids.clone(),
                System::block_number().try_into().unwrap()
            )
        );
        let cur_info = WalletAccountStorage::<Test>::get(&login_account).unwrap();

        let new_spam_report_amount = (10-System::block_number() / DcNode::interval_blocks_reduce_spam()).saturated_into::<u32>();
        assert_eq!(cur_info.spam_report_amount, new_spam_report_amount);
        assert_eq!(cur_info.spam_report_number, (System::block_number() - System::block_number() % DcNode::interval_blocks_reduce_spam()));
        let new_comment_report_amount = (10-System::block_number() / DcNode::interval_blocks_reduce_comment()).saturated_into::<u32>();
        assert_eq!(cur_info.comment_report_amount, new_comment_report_amount);
        assert_eq!(cur_info.comment_report_number, (System::block_number() - System::block_number() % DcNode::interval_blocks_reduce_comment()));
        let times_info: BTreeMap<AppID, AppLoginInfo<AccountId>> = AppsAccountLoginTimes::<Test>::get().unwrap();
        let app_info = times_info.get(&vec![50,51,53]).unwrap();
        assert_eq!(app_info.login_times, 1);

        System::set_block_number(System::block_number() + DcNode::interval_blocks_login());
        assert_ok!(
            DcNode::user_login(
                RuntimeOrigin::signed(caller.clone()), 
                login_account.clone(),
                app_ids.clone(),
                System::block_number().try_into().unwrap()
            )
        );
        assert_eq!(AppsAccountLoginTimes::<Test>::get().unwrap().get(&vec![50,51,53]).unwrap().login_times, 2);
    });
}

#[test]
fn dc_new_theme() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "new_theme", NODE_STATUS_ONCHAIN);
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
        let theme_id = vec![51; 32];
        let app_id = vec![52; 32];
        let comment_space = 12u64;
        let open_flag = 1;
        let block_height = 10u32;
        let signature = vec![59; 1024];

        assert_noop!(
            DcNode::new_theme(
                RuntimeOrigin::signed(caller.clone()), 
                account("t_caller", 0, 0),
                theme_id.clone(),
                app_id.clone(),
                comment_space,
                open_flag,
                block_height, 
                signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_ok!(
            DcNode::new_theme(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                theme_id.clone(),
                app_id.clone(),
                comment_space,
                open_flag,
                block_height, 
                signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().comment_space, comment_space);
    });
}

#[test]
fn dc_add_theme_comment_space() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_theme_comment_space", NODE_STATUS_ONCHAIN);
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
        let theme_id = vec![51; 32];
        let app_id = vec![52; 32];
        let add_space = 12u64;
        let block_height = 10u32;
        let signature = vec![59; 1024];

        assert_noop!(
            DcNode::add_theme_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                account("t_caller", 0, 0),
                theme_id.clone(),
                app_id.clone(),
                add_space,
                block_height, 
                signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_ok!(
            DcNode::add_theme_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                theme_id.clone(),
                app_id.clone(),
                add_space,
                block_height, 
                signature.clone()
            )
        );
        assert_ok!(
            DcNode::add_theme_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                theme_id.clone(),
                app_id.clone(),
                add_space,
                block_height, 
                signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().comment_space, add_space*2);
    });
}

#[test]
fn dc_add_user_comment_space() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "add_user_comment_space", NODE_STATUS_ONCHAIN);
        let for_account: AccountId = user_purchase_storage::<Test>("for_account");
        let block_height = 10u32;
        let signature = vec![59; 1024];

        assert_noop!(
            DcNode::add_user_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                account("t_caller", 0, 0),
                block_height, 
                signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        assert_ok!(
            DcNode::add_user_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                block_height, 
                signature.clone()
            )
        );
        assert_ok!(
            DcNode::add_user_comment_space(
                RuntimeOrigin::signed(caller.clone()), 
                for_account.clone(),
                block_height, 
                signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(for_account.clone()).unwrap().comment_space, DcNode::comment_reduce_space()*2);
    });
}

#[test]
fn dc_report_malicious_comment() {
	new_test_ext().execute_with(|| {
        let peer_id = vec![56; 32];
        let caller = add_onchain_node::<Test>(peer_id.clone(), "report_malicious_comment", NODE_STATUS_ONCHAIN);
        let report_account: AccountId = user_purchase_storage::<Test>("report_account");
        let report_block_height = 10u32;
        let report_signature = vec![59; 1024];
        let theme_id = vec![51; 32];
        let content_id = vec![53; 32];
        let comment_account: AccountId = user_purchase_storage::<Test>("comment_account");
        let app_id = vec![52; 32];
        let comment_block_height = 10u32;
        let refer_comment_key =  vec![55; 32];
        let content_type = 1u32;
        let comment_signature = vec![59; 1024];

        System::set_block_number(DcNode::interval_blocks_can_not_report()+30u64);
        assert_ok!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                comment_account.clone(),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            )
        );
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_report_amount, 0);
        
        System::set_block_number(0u64);
        assert_noop!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                account("t_caller", 0, 0),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                comment_account.clone(),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            ),
            Error::<Test>::AccountNotExist
        );

        assert_noop!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                comment_account.clone(),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            ), 
            Error::<Test>::PeerIdNotExist
        );

        let _ = DcNode::add_user_peer(
            RuntimeOrigin::signed(caller.clone()), 
            report_account.clone(),
            0u32.into()
        );
        assert_noop!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                account("t_caller", 0, 0),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            ), 
            Error::<Test>::AccountNotExist
        );

        System::set_block_number(30u32.into());
        assert_ok!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                comment_account.clone(),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            ),
        );
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_report_number, 30);
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_report_amount, 1);

        let mut user_info = WalletAccountStorage::<Test>::get(&comment_account).unwrap();
        user_info.comment_report_amount = DcNode::frozen_report_comment_amount()-1;
        WalletAccountStorage::<Test>::insert(&comment_account, user_info);

        System::set_block_number(100u32.into());
        assert_ok!(
            DcNode::report_malicious_comment(
                RuntimeOrigin::signed(caller.clone()), 
                report_account.clone(),
                report_block_height,
                report_signature.clone(),
                theme_id.clone(),
                content_id.clone(),
                comment_account.clone(),
                app_id.clone(),
                comment_block_height,
                refer_comment_key.clone(),
                content_type, 
                comment_signature.clone()
            ),
        );
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_report_amount, DcNode::frozen_report_comment_amount());
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_frozen_status, 1);
        assert_eq!(DcNode::wallet_account_storage(comment_account.clone()).unwrap().comment_report_number, 30);
    });
}