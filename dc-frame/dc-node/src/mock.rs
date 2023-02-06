use crate::{self as pallet_dc_node, *};

use frame_support::{
	parameter_types,
	traits::{
		GenesisBuild, 
	},
	weights::constants::RocksDbWeight,
	pallet_prelude::Weight,
};

use sp_core::{H256};
use sp_runtime::{
	testing::{Header},
	traits::{IdentityLookup, IdentifyAccount, Verify},
	MultiSignature,
};

use pallet_transaction_payment::{Multiplier, NextMultiplier};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub(crate) type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
pub(crate) type Balance = u128;
pub(crate) type AccountIndex = u64;
pub(crate) type BlockNumber = u64;
pub(crate) type SessionIndex = u64;

pub struct StakingMock<T: Config> {
	pub account_id: T::AccountId,
}

impl<T: Config> StakingProvider for StakingMock<T> {
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    
	/// Get staking active of stash.
    fn get_staking_active(_stash: &Self::AccountId) -> Self::Balance {
		20000u32.into()
	}

	/// Get current era index
    fn get_current_era_index() -> EraIndex {
		1
	}

	/// Check the controller account mapped by the "stash" account
    fn is_bonded_controller(_stash: &Self::AccountId, _controller: &Self::AccountId) -> bool {
		true
	}

	fn report_offence(
		_stash: &Self::AccountId,
		_slash: Self::Balance,
	) {
	}
}

pub struct PaymentMock {
}

impl NextMultiplier for PaymentMock
{
	/// Get the next multiplier
	fn get_next_fee_multiplier() -> Multiplier {
		Multiplier::from_inner(1000000000000000000u128)
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		DcNode: pallet_dc_node::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			Weight::from_parts(frame_support::weights::constants::WEIGHT_REF_TIME_PER_SECOND * 2, u64::MAX),
		);
	pub static SessionsPerEra: SessionIndex = 3;
	pub static ExistentialDeposit: Balance = 1;
	pub static SlashDeferDuration: EraIndex = 0;
	pub static Period: BlockNumber = 5;
	pub static Offset: BlockNumber = 0;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = RocksDbWeight;
	type RuntimeOrigin = RuntimeOrigin;
	type Index = AccountIndex;
	type BlockNumber = BlockNumber;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = frame_support::traits::ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}
impl pallet_balances::Config for Test {
	type MaxLocks = frame_support::traits::ConstU32<1024>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}


parameter_types! {
	pub DefaultAccountId: AccountId = hex_literal::hex!("2bb43fdff91b4d6adfe15c48cccc71ef92eafbf19a791bf6ee5927dfd2a59890").into();
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Currency = Balances;
	type AccountStore = frame_system::Pallet<Test>;
	type DefaultAccountId = DefaultAccountId;
	type StakingProvider = StakingMock<Self>;
	type WeightInfo = pallet_dc_node::weights::SubstrateWeight<Test>;
	type BlockMultiplier = PaymentMock;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let account1: AccountId = hex_literal::hex!("2bb43fdff91b4d6adfe15c48cccc71ef92eafbf19a791bf6ee5927dfd2a51235").into();
	let account2: AccountId = hex_literal::hex!("2bb43fdff91b4d6adfe15c48cccc71ef92eafbf19a791bf6ee5927dfd2a51236").into();
	pallet_balances::GenesisConfig::<Test> { balances: vec![(account1, 100), (account2, 98)], }
		.assimilate_storage(&mut t)
		.unwrap();
	pallet_dc_node::GenesisConfig::<Test> {
		onchain_peer_number: 0u32,
		app_reward_percent: 60,
		app_reward_total: Default::default(),
		storage_reward_total: Default::default(),
		min_staking_amount: 200u32.into(),
		faking_report_number: 3,
		abnormal_report_number: 3,
		blocks_of_offchain_to_abnormal: 10000u32.into(),
		comment_reduce_space: 20*1024*1024,
		start_reward_block_number: 2000u32.into(),
		max_storage_node_space: 100*1024*1024*1024*1024,
		valid_call_block_number: 200u32.into(),
		frozen_report_spam_amount: 100,
		interval_blocks_reduce_spam: 14400u32.into(),
		frozen_report_comment_amount: 100,
		interval_blocks_reduce_comment: 14400u32.into(),
		interval_blocks_can_not_report: 14400u32.into(),
		interval_blocks_work_report: 28800u32.into(),
		interval_blocks_login: 28800u32.into(),
		tee_report_verify_number: 300u32.into(),
	}
	.assimilate_storage(&mut t)
	.unwrap();
	t.into()
}
