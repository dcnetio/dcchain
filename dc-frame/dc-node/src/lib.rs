#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>

use frame_support::{
    dispatch::{DispatchResult, Pays},
    traits::{
        Currency, Get,
        LockableCurrency, ExistenceRequirement, StoredMap,
    },
};
 
use codec::{Codec, Decode, Encode, MaxEncodedLen};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, MaybeSerializeDeserialize, Zero, Saturating,
    },
    Perbill, RuntimeDebug, SaturatedConversion, FixedPointNumber,
};

use scale_info::TypeInfo;
use scale_info::prelude::string::String;
use sp_std::{
    fmt::Debug,
    vec::Vec,
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
};
use sp_staking::EraIndex;

use pallet_balances::{AccountData};
use pallet_transaction_payment::{NextMultiplier};
use sp_core::{ed25519};
use sp_io::{crypto::{ed25519_verify}};
// use lite_json::json::JsonValue;
pub use pallet::*;
pub mod weights;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod testing_utils;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

const ONE_G_BYTE: u64 = 1024*1024*1024;

pub type SpaceSize = u64;
pub type LoginTimes = u64;
type PeerId = Vec<u8>;
type NftAccount = Vec<u8>;
type FileID = Vec<u8>;
type AppID = Vec<u8>;
pub type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

type DcString = Vec<u8>;
type PackageId = u32;

/// Status of an Storage node
/// Offchain: peer report offline
const NODE_STATUS_OFFCHAIN: u32 = 1;
/// joining
const NODE_STATUS_JOINING: u32 = 2;
/// Onchain
const NODE_STATUS_ONCHAIN: u32 = 3;
/// Staked
const NODE_STATUS_STAKED: u32 = 4;
/// Abnormal: peer request error
const NODE_STATUS_ABNORMAL: u32 = 5;
/// Closed: user close/quit the project, other storage node backup the file on the user's node
const NODE_STATUS_CLOSED: u32 = 6;
/// Discard: tee fake
const NODE_STATUS_DISCARD: u32 = 7;

/// The max length of app id
const APPID_MAX_LENGTH: u32 = 32;
/// The max number of storage nodes that user can request to
const USER_REQUEST_NODE_MAX_NUM: usize = 5;
/// The max number of missing files/accounts 
pub const MISSING_FILES_MAX_NUM: u32 = 10;
 

/// Information of an Storage node.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct StorageNode<AccountId, BlockNumber> {
    /// The account associated with the Storage node, which is responsible for communicating with the chain.
    pub req_account: AccountId,
    /// Stash account associated with the Storage node.
    pub stash: AccountId,
    /// Total space size unit byte.
    pub total_space: SpaceSize,
    /// Free space size unit byte.
    pub free_space: SpaceSize,
    /// Status of an Storage node.
    pub status: u32,
    /// The block number that storage node report.
    pub report_number: BlockNumber,
    /// The block number that storage node completed staking.
    pub staked_number: BlockNumber,
    /// The block number that can be rewarded.
    pub reward_number: BlockNumber,
    /// The Ip Addess of Storage node
    pub ip_address: DcString,
    /// The SGX version
    pub sgx_version_number: u8,
}

/// Storage information of user.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct UserStorage<AccountId, BlockNumber, Balance> {
    /// List of node IDs where login information is stored.
    pub peers: BTreeSet<PeerId>,
    /// Used space size unit byte.
    pub used_space: SpaceSize,
    /// Subscribed space size unit byte.
    pub subscribe_space: SpaceSize,
    /// Subscribed package price
    pub subscribe_price: Balance,
    /// block numbers deducted for interface calls
    pub call_minus_number: BlockNumber,
    /// The update block number for nft account.
    pub nft_update_number: BlockNumber,
    /// The update block number for db config.
    pub db_update_number: BlockNumber,
    /// The block number that will be expireed.
    pub expire_number: BlockNumber,
    /// The thread db config infomation of wallet account
    pub db_config: DcString,
    /// The encrypted NFT account.
    pub enc_nft_account: NftAccount,
    /// The parent account
    pub parent_account: AccountId,
    /// The spam frozen status: 0:no freeze 1:frozen
    pub spam_frozen_status: u8,
    /// The amount of spam reports
    pub spam_report_amount: u32,
    /// The block number of spam reports, the starting point for reducing the number of reports
    pub spam_report_number: BlockNumber,
    /// The comment frozen status: 0:no freeze 1:frozen
    pub comment_frozen_status: u8,
    /// The amount of comment reports
    pub comment_report_amount: u32,
    /// The block number of comment reports, the starting point for reducing the number of reports
    pub comment_report_number: BlockNumber,
    /// The number of blocks when the user is logged in
    pub login_number: BlockNumber,
    /// The comment space size of user.
    pub comment_space: SpaceSize,
    /// List of node IDs where the user can request to.
    pub request_peers: BTreeSet<PeerId>,
}

/// File type
/// common file
// const FILE_TYPE_COMMON: u32 = 1;
/// threaddb file
const FILE_TYPE_THREAD_DB:u32 =2;

/// Information of file.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct FileInfo<AccountId> {
    /// List of node IDs where the file is stored.
    pub peers: BTreeSet<PeerId>,
    /// List of users who own the file.
    pub users: BTreeSet<AccountId>,
    /// File size unit byte.
    pub file_size: SpaceSize,
    /// File type.
    pub file_type: u32,
    /// thread db information
    pub db_log: BTreeSet<DcString>,
}

/// The report information type.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo, Copy)]
pub enum ReportType {
    /// Report tee faking of storage node.
    ReportTeeFaking = 1,
    /// Verify tee faking of storage node.
    VerifyTeeFaking = 2,
    /// Report a storage node offchain to the chain.
    ReportPeerOffchain = 3,
    /// Report a storage node no response to the chain.
    ReportPeerNoResponse = 4,
}

/// Report information.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct ReportInfo {
    /// The report information type.
    pub report_type: ReportType,
    /// node ID.
    pub peer_id: PeerId,
}

/// Account information of app.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct AppAccountInfo<AccountId> {
    /// The private account.
    pub private_account: AccountId,
    /// Rewarded stash account.
    pub rewarded_stash: AccountId,
}

/// Login information of app.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct AppLoginInfo<AccountId> {
    /// Rewarded stash account.
    pub rewarded_stash: AccountId,
    /// Login times of app.
    pub login_times: LoginTimes,
}

/// DC program information.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode, TypeInfo)]
pub struct DcProgramInfo {
    /// Program download url
    pub origin_url: DcString,
    /// Program download mirror url
    pub mirror_url: DcString,
    /// Enclave id.
    pub enclave_id: DcString,
    /// The version of program.
    pub version: DcString,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The balance of an account.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Codec
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + MaxEncodedLen
            + TypeInfo;
         
        /// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The DC balance.
        type Currency: LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
        /// The means of storing the balances of an account.
        type AccountStore: StoredMap<Self::AccountId, AccountData<Self::Balance>>;
        // A type that can deliver a single account id value to the pallet.
        type DefaultAccountId: Get<<Self as frame_system::Config>::AccountId>;
        /// Something that provides the staking functionality.
		type StakingProvider: StakingProvider<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

        // Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
        type BlockMultiplier: NextMultiplier;
    }

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    /// The peers.
    #[pallet::storage]
    #[pallet::getter(fn peers)]
    pub type Peers<T: Config> = StorageMap<_, Twox64Concat, PeerId, StorageNode<T::AccountId, T::BlockNumber>>;

    /// The Storage node associated with request account.
    #[pallet::storage]
    #[pallet::getter(fn request_account_peer)]
    pub type RequestAccountPeer<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, PeerId>;

    /// The Storage nodes associated with stash account.
    #[pallet::storage]
    #[pallet::getter(fn stash_peers)]
    pub type StashPeers<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BTreeSet<PeerId>>;

    /// Number of onchain peers.
    #[pallet::storage]
    #[pallet::getter(fn onchain_peer_number)]
    pub type OnchainPeerNumber<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// NFT accounts.
    #[pallet::storage]
    #[pallet::getter(fn nft_to_wallet_account)]
    pub type NftToWalletAccount<T: Config> = StorageMap<_, Twox64Concat, NftAccount, T::AccountId>;

    /// Storage information of wallet accounts.
    #[pallet::storage]
    #[pallet::getter(fn wallet_account_storage)]
    pub type WalletAccountStorage<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, UserStorage<T::AccountId, T::BlockNumber, BalanceOf<T>>>;

    /// The files of user.
    #[pallet::storage]
    #[pallet::getter(fn files)]
    pub type Files<T: Config> = StorageMap<_, Twox64Concat, FileID, FileInfo<T::AccountId>>;

    /// The percent of storage rewards in the total(app rewards + storage rewards)
	#[pallet::storage]
	#[pallet::getter(fn app_reward_percent)]
	pub type AppRewardPercent<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The total amount of the app's share of the total amount of currency required by the user to purchase storage.
    #[pallet::storage]
    #[pallet::getter(fn app_reward_total)]
    pub type AppRewardTotal<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The total amount of the storage node's share of the total amount of currency required by the user to purchase storage.
    #[pallet::storage]
    #[pallet::getter(fn storage_reward_total)]
    pub type StorageRewardTotal<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// The storage packages. (id, space size unit byte, package price, valid block numbers, block numbers deducted for interface calls)
    #[pallet::storage]
    #[pallet::getter(fn storage_packages)]
    pub type StoragePackages<T: Config> = StorageValue<_, BTreeSet<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>>;

    /// The minimum staking amount of storage nodes that can obtain storage rewards.
    #[pallet::storage]
    #[pallet::getter(fn min_staking_amount)]
    pub type MinStakingAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

    /// Report the number of storage node tee frauds, when the number is exceeded, the storage node will be punished 
    #[pallet::storage]
    #[pallet::getter(fn faking_report_number)]
    pub type FakingReportNumber<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Report the number of abnormal storage node , when the number is exceeded, the storage node will be punished 
    #[pallet::storage]
    #[pallet::getter(fn abnormal_report_number)]
    pub type AbnormalReportNumber<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// The number of blocks in the interval between node status offchain and abnormal
    #[pallet::storage]
    #[pallet::getter(fn blocks_of_offchain_to_abnormal)]
    pub type BlocksOfOffchainToAbnormal<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// Storage space deducted for adding comment space.
    #[pallet::storage]
    #[pallet::getter(fn comment_reduce_space)]
    pub type CommentReduceSpace<T: Config> = StorageValue<_, SpaceSize, ValueQuery>;

    /// Block number to start rewarding.
    #[pallet::storage]
    #[pallet::getter(fn start_reward_block_number)]
    pub type StartRewardBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The max space size of storage node.
    #[pallet::storage]
    #[pallet::getter(fn max_storage_node_space)]
    pub type MaxStorageNodeSpace<T: Config> = StorageValue<_, SpaceSize, ValueQuery>;

    /// The effective call block numbers of the interface.
    #[pallet::storage]
    #[pallet::getter(fn valid_call_block_number)]
    pub type ValidCallBlockNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// After the number of reported spam messages is exceeded, the account will be frozen, and the account will not be able to send messages.
    #[pallet::storage]
    #[pallet::getter(fn frozen_report_spam_amount)]
    pub type FrozenReportSpamAmount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Interval blocks to reduce the number of spam reports.
    #[pallet::storage]
    #[pallet::getter(fn interval_blocks_reduce_spam)]
    pub type IntervalBlocksReduceSpam<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// After the number of reported comments is exceeded, the account will be frozen, and the account will not be able to comment.
    #[pallet::storage]
    #[pallet::getter(fn frozen_report_comment_amount)]
    pub type FrozenReportCommentAmount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Interval blocks to reduce the number of comment reports.
    #[pallet::storage]
    #[pallet::getter(fn interval_blocks_reduce_comment)]
    pub type IntervalBlocksReduceComment<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The number of interval blocks that cannot be reported.
    #[pallet::storage]
    #[pallet::getter(fn interval_blocks_can_not_report)]
    pub type IntervalBlocksCanNotReport<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The number of interval blocks for the maximum interval of the work report.
    #[pallet::storage]
    #[pallet::getter(fn interval_blocks_work_report)]
    pub type IntervalBlocksWorkReport<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The number of interval blocks for the maximum interval of the user login.
    #[pallet::storage]
    #[pallet::getter(fn interval_blocks_login)]
    pub type IntervalBlocksLogin<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The number of interval blocks that tee report verified.
    #[pallet::storage]
    #[pallet::getter(fn tee_report_verify_number)]
    pub type TeeReportVerifyNumber<T: Config> = StorageValue<_, T::BlockNumber, ValueQuery>;

    /// The list of enclave ids for storage node.
    #[pallet::storage]
    #[pallet::getter(fn enclave_ids)]
    pub type EnclaveIds<T: Config> = StorageValue<_, BTreeSet<(T::BlockNumber, DcString, DcString)>>;

    /// The Dc program infomation.
    #[pallet::storage]
    #[pallet::getter(fn dc_program)]
    pub type DcProgram<T: Config> = StorageValue<_, DcProgramInfo>;

    /// The list of blockchain proxy nodes.
    #[pallet::storage]
    #[pallet::getter(fn proxy_nodes)]
    pub type ProxyNodes<T: Config> = StorageValue<_, BTreeSet<(DcString, DcString)>>;

    /// The list of trusted storage nodes.
    #[pallet::storage]
    #[pallet::getter(fn trusted_storage_nodes)]
    pub type TrustedStorageNodes<T: Config> = StorageValue<_, BTreeSet<(DcString, DcString)>>;

    /// All slashing events on nominators, mapped by era to the highest slash value of the era.
	#[pallet::storage]
    #[pallet::getter(fn reports_in_era)]
	pub(crate) type ReportsInEra<T: Config> = StorageDoubleMap<_, Twox64Concat, EraIndex, Twox64Concat, ReportInfo, BTreeSet<T::AccountId>>;

    /// The account information associated with the app id.
	#[pallet::storage]
    #[pallet::getter(fn account_of_app)]
	pub(crate) type AccountOfApp<T: Config> = StorageMap<_, Twox64Concat, AppID, AppAccountInfo<T::AccountId>>;

    /// The times of logins for each app.<app stash account, longin times>
	#[pallet::storage]
    #[pallet::getter(fn apps_account_login_times)]
	pub(crate) type AppsAccountLoginTimes<T: Config> = StorageValue<_, BTreeMap<AppID, AppLoginInfo<T::AccountId>>>;


    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub onchain_peer_number: u32,
        pub app_reward_percent: u32,
        pub app_reward_total: BalanceOf<T>,
        pub storage_reward_total: BalanceOf<T>,
        pub min_staking_amount: BalanceOf<T>,
        pub faking_report_number: u32,
        pub abnormal_report_number: u32,
        pub blocks_of_offchain_to_abnormal: T::BlockNumber,
        pub comment_reduce_space: SpaceSize,
        pub start_reward_block_number: T::BlockNumber,
        pub max_storage_node_space: SpaceSize,
        pub valid_call_block_number: T::BlockNumber,
        pub frozen_report_spam_amount: u32,
        pub interval_blocks_reduce_spam: T::BlockNumber,
        pub frozen_report_comment_amount: u32,
        pub interval_blocks_reduce_comment: T::BlockNumber,
        pub interval_blocks_can_not_report: T::BlockNumber,
        pub interval_blocks_work_report: T::BlockNumber,
        pub interval_blocks_login: T::BlockNumber,
        pub tee_report_verify_number: T::BlockNumber,
        pub dev_config: Vec<(T::AccountId, T::AccountId, T::AccountId, PeerId)>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> 
    {
        fn default() -> Self {
            GenesisConfig {
                onchain_peer_number: 0,
                app_reward_percent: 60,
                app_reward_total: Default::default(),
                storage_reward_total: Default::default(),
                min_staking_amount: Default::default(),
                faking_report_number: 3,
                abnormal_report_number: 3,
                blocks_of_offchain_to_abnormal: 10000u32.into(),
                comment_reduce_space: 20*1024*1024,
                start_reward_block_number: 2000u32.into(),
                max_storage_node_space: 200*1024*1024*1024*1024,
                valid_call_block_number: 200u32.into(),
                frozen_report_spam_amount: 100,
                interval_blocks_reduce_spam: 14400u32.into(),
                frozen_report_comment_amount: 100,
                interval_blocks_reduce_comment: 14400u32.into(),
                interval_blocks_can_not_report: 14400u32.into(),
                interval_blocks_work_report: 28800u32.into(),
                interval_blocks_login: 28800u32.into(),
                tee_report_verify_number: 300u32.into(),
                dev_config: Default::default(),
            }
        }
    }
    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> 
    where 
        T::AccountId: AsRef<[u8]>,
    {
        fn build(&self) {
            <OnchainPeerNumber<T>>::put(self.onchain_peer_number);
            AppRewardPercent::<T>::put(self.app_reward_percent);
            AppRewardTotal::<T>::put(self.app_reward_total);
            StorageRewardTotal::<T>::put(self.storage_reward_total);
            MinStakingAmount::<T>::put(self.min_staking_amount);
            FakingReportNumber::<T>::put(self.faking_report_number);
            AbnormalReportNumber::<T>::put(self.abnormal_report_number);
            BlocksOfOffchainToAbnormal::<T>::put(self.blocks_of_offchain_to_abnormal);
            CommentReduceSpace::<T>::put(self.comment_reduce_space);
            StartRewardBlockNumber::<T>::put(self.start_reward_block_number);
            MaxStorageNodeSpace::<T>::put(self.max_storage_node_space);
            ValidCallBlockNumber::<T>::put(self.valid_call_block_number);
            FrozenReportSpamAmount::<T>::put(self.frozen_report_spam_amount);
            IntervalBlocksReduceSpam::<T>::put(self.interval_blocks_reduce_spam);
            FrozenReportCommentAmount::<T>::put(self.frozen_report_comment_amount);
            IntervalBlocksReduceComment::<T>::put(self.interval_blocks_reduce_comment);
            IntervalBlocksCanNotReport::<T>::put(self.interval_blocks_can_not_report);
            IntervalBlocksWorkReport::<T>::put(self.interval_blocks_work_report);
            IntervalBlocksLogin::<T>::put(self.interval_blocks_login);
            TeeReportVerifyNumber::<T>::put(self.tee_report_verify_number);
            // For private devlopment chain
            for &(ref controller, ref stash, ref req_account, ref peer_id) in &self.dev_config {
                frame_support::assert_ok!(<Pallet<T>>::join_storage_node(
                    T::RuntimeOrigin::from(Some(req_account.clone()).into()),
                    peer_id.clone(),
                    100_000_000_000_000,
                    100_000_000_000_000,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ));
                frame_support::assert_ok!(<Pallet<T>>::set_stash_peer(
                    T::RuntimeOrigin::from(Some(controller.clone()).into()),
                    stash.clone(),
                    peer_id.clone(),
                ));
                frame_support::assert_ok!(<Pallet<T>>::join_storage_node(
                    T::RuntimeOrigin::from(Some(req_account.clone()).into()),
                    peer_id.clone(),
                    100_000_000_000_000,
                    100_000_000_000_000,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    Default::default(),
                ));
                if <StoragePackages::<T>>::get().is_none() {
                    let mut packages = BTreeSet::<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>::new();
                    packages.insert((1, 60*1024*1024*1024, 10000u32.into(), 1000000u32.into(), 10u32.into()));
                    packages.insert((2, 120*1024*1024*1024, 20000u32.into(), 1000000u32.into(), 5u32.into()));
                    <StoragePackages::<T>>::put(packages);
                }
                if <EnclaveIds::<T>>::get().is_none() {
                    let mut enclave_ids: BTreeSet<(T::BlockNumber, DcString, DcString)> = BTreeSet::<(T::BlockNumber, DcString, DcString)>::new();
                    enclave_ids.insert((
                        frame_system::Pallet::<T>::block_number(), 
                        "72c3b468cd159809cb521885deb3ddce8a6f8a0b23b43785990c7f4b5bc7b8cf".as_bytes().to_vec(), 
                        "bxbmtgfh5wc7cwzlof5hifsv5rgswkmnvkaamfchcaaniqqclzxmc2z7yaald7qejho6s4thcy3uc6dhtfexp44qsmrn32jpkzfccsca".as_bytes().to_vec()));
                    <EnclaveIds::<T>>::put(enclave_ids);
                }
            }
        }
    }

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        JoinStorageNode(T::AccountId, PeerId, SpaceSize, SpaceSize, DcString, u8, T::BlockNumber, DcString),
        PurchaseStorage(PackageId, T::AccountId),
        SetSlashPeer(PeerId, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
        /// Param error.
        ParamErr,
        /// NFT account has been applied
        NftAccoutApplied,
        /// Account does not exist 
        AccountNotExist,
        /// Peer id does not exist 
        PeerIdNotExist,
        /// File does not exist 
        FileNotExist,
        /// File type error 
        FileTypeError,
        /// Node status error
        NodeStatusError,
        /// Account already exists 
        AccountAlreadyExist,
        /// A sub account cannot be used as a parent account  
        IsSubAccount,
        /// Data signature error
        DataSignatureVerify,
        /// Block number invalid
        BlockNumberInvalid,
        /// User's package expired
        UserPackageExpired,
        /// Max storage node size invalid
        MaxStorageNodeSize,
        /// The account for peer is error
        PeerAccountError,
        /// Insufficient balance
        InsufficientBalance,
        /// Storage package does not exist 
        StoragePackageNotExist,
        /// App id length exceeds the limit
        AppIdLengthErr,
        /// Not the controller account.
        NotController,
        /// The invalid percent
        InvalidPercent,
        /// Not a stash account.
		NotStash,
        /// Error node report 
        ErrorNodeReport,
        /// Error work report 
        ErrorWorkReport,
        /// Excessive login
        ExcessiveLogin,
    }

    // #[pallet::hooks]
    // impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
    //     /// Called when a block is initialized. Will rotate session if it is the last
    //     /// block of the current session.
    //     fn on_initialize(n: T::BlockNumber) -> Weight {
    //         log::info!("aaaa---------------ccc-----: {:?}--{:?}--{:?}", n, NodeStatus::Abnormal, NodeStatus::Offchain as u8);
    //         0
    //     }
    // }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> 
    where 
        T::AccountId: AsRef<[u8]>,
    {
        /// Add information of an Storage node.
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::join_storage_node())]
        pub fn join_storage_node(
            origin: OriginFor<T>, 
            peer_id: PeerId, 
            total_space: SpaceSize,
            free_space: SpaceSize,
            ip_address: DcString,
            sgx_version_number: u8,
            block_height: T::BlockNumber,
            tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if total_space > Self::max_storage_node_space() {
                Err(Error::<T>::MaxStorageNodeSize)?
            }

            let cur_block_num = frame_system::Pallet::<T>::block_number();
            let is_exist = <Peers<T>>::contains_key(&peer_id);
            if !is_exist {
                let new_info = StorageNode {
                    req_account: who.clone(),
                    stash: T::DefaultAccountId::get(),
                    total_space: total_space,
                    free_space: free_space,
                    status: NODE_STATUS_JOINING,
                    report_number: cur_block_num,
                    staked_number: Zero::zero(),
                    reward_number: cur_block_num.saturating_add(Self::start_reward_block_number()),
                    ip_address: ip_address.clone(),
                    sgx_version_number: sgx_version_number,
                };
                // Add storage node.
                <Peers<T>>::insert(&peer_id, new_info);
                // Set peer ID of the request account
                <RequestAccountPeer<T>>::insert(&who, peer_id.clone());
            } else {
                let pre_info = <Peers<T>>::get(&peer_id).unwrap();
                if pre_info.req_account != who {
                    Err(Error::<T>::PeerAccountError)?
                }
                if (pre_info.status == NODE_STATUS_STAKED 
                    && cur_block_num.saturating_sub(pre_info.staked_number) < Self::tee_report_verify_number())
                    || pre_info.status == NODE_STATUS_JOINING
                    || pre_info.status == NODE_STATUS_DISCARD {
                    return Ok(().into());
                } else {
                    let staking_active = T::StakingProvider::get_staking_active(&pre_info.stash);
                    // Set the status of the nodes based on the amount of stake
                    Self::update_peers_of_stash(&pre_info.stash, staking_active);

                    let mut cur_info = <Peers<T>>::get(&peer_id).unwrap();
                    let mut begin_number = cur_info.reward_number;
                    if cur_block_num > cur_info.reward_number {
                        begin_number = cur_block_num;
                    }
                    cur_info.report_number = cur_block_num;
                    cur_info.total_space = total_space;
                    cur_info.reward_number = begin_number.saturating_add(Self::start_reward_block_number());
                    cur_info.ip_address = ip_address.clone();
                    cur_info.sgx_version_number = sgx_version_number;
                    <Peers<T>>::insert(&peer_id, cur_info);
                }
            }
            Self::deposit_event(Event::JoinStorageNode(who, peer_id, total_space, free_space, ip_address, sgx_version_number, block_height, tee_report));
            Ok(().into())
        }

        /// Submit work report of storage node. 
        #[pallet::call_index(1)]
        #[pallet::weight((T::WeightInfo::submit_work_report(miss_files.len().try_into().unwrap(), miss_accounts.len().try_into().unwrap()), DispatchClass::Operational))]
        pub fn submit_work_report(
            origin: OriginFor<T>, 
            total_space: SpaceSize,
            free_space: SpaceSize,
            ip_address: DcString,
            miss_files: Vec<DcString>,
            miss_accounts: Vec<T::AccountId>,
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;
            let mut is_repeat_report = false;
            let is_exist = <Peers<T>>::contains_key(&peer_id);
            if is_exist {
                let mut pre_info = <Peers<T>>::take(&peer_id).unwrap();
                
                let cur_num = frame_system::Pallet::<T>::block_number();
                if cur_num.saturating_sub(pre_info.report_number) < Self::interval_blocks_work_report().saturating_sub(300u32.into()) {
                    is_repeat_report = true;
                }

                // Delete file from peer
                let mut file_miss_len = 0;
                let mut index_x = 0;
                for file_id in &miss_files {
                    if index_x >= MISSING_FILES_MAX_NUM {
                        break;
                    }
                    index_x += 1;
                    let r_ret = Self::remove_file_peer(&peer_id, &file_id);
                    if r_ret.is_ok() {
                        file_miss_len += 1;
                    }
                }

                let mut account_miss_len = 0;
                let mut index_y = 0;
                for for_account in &miss_accounts {
                    if index_y >= MISSING_FILES_MAX_NUM {
                        break;
                    }
                    index_y += 1;
                    let r_ret = Self::remove_account_peer(&peer_id, &for_account);
                    if r_ret.is_ok() {
                        account_miss_len += 1;
                    }
                }

                // File loss penalty
                if file_miss_len > 0 || account_miss_len > 0 {
                    if pre_info.reward_number > cur_num {
                        pre_info.reward_number = pre_info.reward_number.saturating_add(Self::start_reward_block_number());
                    } else {
                        pre_info.reward_number = cur_num.saturating_add(Self::start_reward_block_number());
                    }
                }
                pre_info.total_space = total_space;
                pre_info.free_space = free_space;
                pre_info.ip_address = ip_address.clone();
                pre_info.report_number = cur_num;
                <Peers<T>>::insert(peer_id.clone(), pre_info);
            }
            if is_repeat_report {
                Ok(Pays::Yes.into())
            } else {
                Ok(Pays::No.into())
            }
        }

        /// Set peer ID of the stash
        #[pallet::call_index(2)]
        #[pallet::weight(T::DbWeight::get().reads_writes(5, 3))]
        pub fn set_stash_peer(origin: OriginFor<T>, stash: T::AccountId, peer_id: PeerId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if !T::StakingProvider::is_bonded_controller(&stash, &who) {
                return Err(Error::<T>::NotStash)?;
            }
            let mut pre_info = <Peers<T>>::get(&peer_id).ok_or(Error::<T>::PeerIdNotExist)?;
            
            // The stash account is did not setted
            if pre_info.stash == T::DefaultAccountId::get() {
                pre_info.stash = stash.clone();
                <Peers<T>>::insert(&peer_id, pre_info);
                let mut peer_id_set = Self::stash_peers(&stash).unwrap_or(BTreeSet::<PeerId>::new());
                peer_id_set.insert(peer_id.clone());
                <StashPeers<T>>::insert(&stash, peer_id_set);
            }
            let staking_active = T::StakingProvider::get_staking_active(&stash);
            // Set the status of the nodes based on the amount of stake
            Self::update_peers_of_stash(&stash, staking_active);
            Self::deposit_event(Event::SetSlashPeer(peer_id, stash));
            Ok(())
        }

        /// Remove peer ID of the stash
        #[pallet::call_index(3)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 2))]
        pub fn remove_stash_peer(origin: OriginFor<T>, stash: T::AccountId, peer_id: PeerId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if !T::StakingProvider::is_bonded_controller(&stash, &who) {
                return Err(Error::<T>::NotStash)?;
            }
            let mut pre_info = <Peers<T>>::get(&peer_id).ok_or(Error::<T>::PeerIdNotExist)?;
            // The stash account is did not setted
            if pre_info.stash == stash {
                pre_info.stash = T::DefaultAccountId::get();
                if pre_info.status == NODE_STATUS_ONCHAIN {
                    <OnchainPeerNumber<T>>::mutate(|n| *n -= 1);
                    pre_info.status = NODE_STATUS_JOINING;
                }
                <Peers<T>>::insert(&peer_id, pre_info);
                let mut peer_id_set = Self::stash_peers(&stash).unwrap_or(BTreeSet::<PeerId>::new());
                peer_id_set.remove(&peer_id);
                <StashPeers<T>>::insert(&stash, peer_id_set);
            }
            Ok(())
        }

        /// Stop peer of the stash
        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(2, 1))]
        pub fn stop_stash_peer(origin: OriginFor<T>, stash: T::AccountId, peer_id: PeerId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            if !T::StakingProvider::is_bonded_controller(&stash, &who) {
                return Err(Error::<T>::NotStash)?;
            }
            let mut pre_info = <Peers<T>>::get(&peer_id).ok_or(Error::<T>::PeerIdNotExist)?;
            // The stash account is did not setted
            if pre_info.stash == stash && pre_info.status != NODE_STATUS_CLOSED && pre_info.status != NODE_STATUS_DISCARD {
                if pre_info.status == NODE_STATUS_ONCHAIN {
                    <OnchainPeerNumber<T>>::mutate(|n| *n -= 1);
                }

                pre_info.status = NODE_STATUS_CLOSED;
                <Peers<T>>::insert(&peer_id, pre_info);
            }
            Ok(())
        }

        /// Set the percent of app rewards in the total(app rewards + storage rewards).
        #[pallet::call_index(5)]
		#[pallet::weight(T::DbWeight::get().reads_writes(0,1))]
		pub fn set_app_reward_percent(
			origin: OriginFor<T>,
			percent: u32, 
		) -> DispatchResult {
			ensure_root(origin)?;
			ensure!(percent < 100, Error::<T>::InvalidPercent,);
			<AppRewardPercent::<T>>::put(percent);
			Ok(())
		}

        /// Set the minimum staking amount of storage nodes that can obtain storage rewards.
        #[pallet::call_index(6)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_min_staking(
            origin: OriginFor<T>,
            amount: BalanceOf<T>, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <MinStakingAmount::<T>>::put(amount);
            Ok(())
        }

        /// Set the number of faking report.
        #[pallet::call_index(7)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_faking_report_number(
            origin: OriginFor<T>,
            num: u32, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <FakingReportNumber::<T>>::put(num);
            Ok(())
        }

        /// Set the number of abnormal report.
        #[pallet::call_index(8)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_abnormal_report_number(
            origin: OriginFor<T>,
            num: u32, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <AbnormalReportNumber::<T>>::put(num);
            Ok(())
        }
        
        /// Set the number of blocks in the interval between node status offchain and abnormal.
        #[pallet::call_index(9)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_blocks_of_offchain_to_abnormal(
            origin: OriginFor<T>,
            block_num: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <BlocksOfOffchainToAbnormal::<T>>::put(block_num);
            Ok(())
        }

        /// Set storage space deducted for adding comment sapce.
        #[pallet::call_index(10)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_comment_reduce_space(
            origin: OriginFor<T>,
            reduce_space: SpaceSize, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <CommentReduceSpace::<T>>::put(reduce_space);
            Ok(())
        }

        /// Set block number to start rewarding.
        #[pallet::call_index(11)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_start_reward_block_number(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <StartRewardBlockNumber::<T>>::put(block_number);
            Ok(())
        }

        /// Set the max space size of storage node.
        #[pallet::call_index(12)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_max_storage_node_space(
            origin: OriginFor<T>,
            max_space: SpaceSize, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <MaxStorageNodeSpace::<T>>::put(max_space);
            Ok(())
        }

        /// Set the effective call block numbers of the interface.
        #[pallet::call_index(13)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_valid_call_block_number(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <ValidCallBlockNumber::<T>>::put(block_number);
            Ok(())
        }

        /// Set the spam number of reports that were frozen.
        #[pallet::call_index(14)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_frozen_report_spam_amount(
            origin: OriginFor<T>,
            report_number: u32, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <FrozenReportSpamAmount::<T>>::put(report_number);
            Ok(())
        }

        /// Set the Interval blocks to reduce the number of spam reports.
        #[pallet::call_index(15)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_interval_blocks_reduce_spam(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <IntervalBlocksReduceSpam::<T>>::put(block_number);
            Ok(())
        }

        /// Set the comment number of reports that were frozen.
        #[pallet::call_index(16)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_frozen_report_comment_amount(
            origin: OriginFor<T>,
            report_number: u32, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <FrozenReportCommentAmount::<T>>::put(report_number);
            Ok(())
        }

        /// Set the Interval blocks to reduce the number of comment reports.
        #[pallet::call_index(17)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_interval_blocks_reduce_comment(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <IntervalBlocksReduceComment::<T>>::put(block_number);
            Ok(())
        }

        /// Set the number of interval blocks that cannot be reported.
        #[pallet::call_index(18)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_interval_blocks_can_not_report(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <IntervalBlocksCanNotReport::<T>>::put(block_number);
            Ok(())
        }
        
        /// Set the number of interval blocks for the maximum interval of the work report.
        #[pallet::call_index(19)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_interval_blocks_work_report(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <IntervalBlocksWorkReport::<T>>::put(block_number);
            Ok(())
        }

        /// Set the number of interval blocks for the maximum interval of the user login.
        #[pallet::call_index(20)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_interval_blocks_login(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <IntervalBlocksLogin::<T>>::put(block_number);
            Ok(())
        }

        /// Set the number of interval blocks that tee report verified.
        #[pallet::call_index(21)]
        #[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
        pub fn set_tee_report_verify_number(
            origin: OriginFor<T>,
            block_number: T::BlockNumber, 
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            <TeeReportVerifyNumber::<T>>::put(block_number);
            Ok(())
        }

        /// Set the enclave id for storage node
        #[pallet::call_index(22)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_enclave_id(
            origin: OriginFor<T>,
            enclave_id: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let mut enclave_ids: BTreeSet<(T::BlockNumber, DcString, DcString)> = match Self::enclave_ids() {
                Some(p) => p,
                None => BTreeSet::<(T::BlockNumber, DcString, DcString)>::new(),
            };
            enclave_ids.insert((frame_system::Pallet::<T>::block_number(), enclave_id, signature));
            <EnclaveIds::<T>>::put(enclave_ids);
            Ok(())
        }

        /// Remove the enclave id for storage node
        #[pallet::call_index(23)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_enclave_id(
            origin: OriginFor<T>,
            enclave_id: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut enclave_ids: BTreeSet<(T::BlockNumber, DcString, DcString)> = match Self::enclave_ids() {
                Some(p) => p,
                None => BTreeSet::<(T::BlockNumber, DcString, DcString)>::new(),
            };
            if !enclave_ids.is_empty() {
                for info in enclave_ids.clone().iter() {
                    if info.1 == enclave_id && info.2 == signature {
                        enclave_ids.remove(info);
                    }
                }
                <EnclaveIds::<T>>::put(enclave_ids);
            }
            
            Ok(())
        }

        /// Set the infomation of dc program
        #[pallet::call_index(24)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_dc_program(
            origin: OriginFor<T>,
            origin_url: DcString,
            mirror_url: DcString,
            enclave_id: DcString,
            version: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let pro_info = DcProgramInfo {
                origin_url: origin_url,
                mirror_url: mirror_url,
                enclave_id: enclave_id,
                version: version,
            };
            <DcProgram::<T>>::put(pro_info);
            Ok(())
        }

        /// Set the blockchain proxy node
        #[pallet::call_index(25)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_proxy_node(
            origin: OriginFor<T>,
            node_url: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let mut proxy_nodes: BTreeSet<(DcString, DcString)> = match Self::proxy_nodes() {
                Some(p) => p,
                None => BTreeSet::<(DcString, DcString)>::new(),
            };
            proxy_nodes.insert((node_url, signature));
            <ProxyNodes::<T>>::put(proxy_nodes);
            Ok(())
        }

        /// Remove the blockchain proxy node
        #[pallet::call_index(26)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_proxy_node(
            origin: OriginFor<T>,
            node_url: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut proxy_nodes: BTreeSet<(DcString, DcString)> = match Self::proxy_nodes() {
                Some(p) => p,
                None => BTreeSet::<(DcString, DcString)>::new(),
            };
            if !proxy_nodes.is_empty() {
                for info in proxy_nodes.clone().iter() {
                    if info.0 == node_url && info.1 == signature {
                        proxy_nodes.remove(info);
                    }
                }
                <ProxyNodes::<T>>::put(proxy_nodes);
            }
            
            Ok(())
        }

        /// Set the trusted storage node
        #[pallet::call_index(27)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_trusted_storage_node(
            origin: OriginFor<T>,
            node_url: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let mut storage_nodes: BTreeSet<(DcString, DcString)> = match Self::trusted_storage_nodes() {
                Some(p) => p,
                None => BTreeSet::<(DcString, DcString)>::new(),
            };
            storage_nodes.insert((node_url, signature));
            <TrustedStorageNodes::<T>>::put(storage_nodes);
            Ok(())
        }

        /// Remove the trusted storage node
        #[pallet::call_index(28)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_trusted_storage_node(
            origin: OriginFor<T>,
            node_url: DcString,
            signature: DcString,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut storage_nodes: BTreeSet<(DcString, DcString)> = match Self::trusted_storage_nodes() {
                Some(p) => p,
                None => BTreeSet::<(DcString, DcString)>::new(),
            };
            if !storage_nodes.is_empty() {
                for info in storage_nodes.clone().iter() {
                    if info.0 == node_url && info.1 == signature {
                        storage_nodes.remove(info);
                    }
                }
                <TrustedStorageNodes::<T>>::put(storage_nodes);
            }
            
            Ok(())
        }

        /// Set the storage package
        #[pallet::call_index(29)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_storage_package(
            origin: OriginFor<T>,
            package_id: PackageId, 
            subscribe_space: SpaceSize, 
            subscribe_price: BalanceOf<T>, 
            call_minus_number: T::BlockNumber,
            expire_number: T::BlockNumber,
        ) -> DispatchResult {
            ensure_root(origin)?;
            let mut packages: BTreeSet<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)> = match Self::storage_packages() {
                Some(p) => p,
                None => BTreeSet::<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>::new(),
            };
            packages.insert((package_id, subscribe_space, subscribe_price, expire_number, call_minus_number));
            <StoragePackages::<T>>::put(packages);
            Ok(())
        }

        /// Remove the storage package
        #[pallet::call_index(30)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn remove_storage_package(
            origin: OriginFor<T>,
            package_id: PackageId,
        ) -> DispatchResult {
            ensure_root(origin)?;
            
            let mut packages: BTreeSet<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)> = match Self::storage_packages() {
                Some(p) => p,
                None => BTreeSet::<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>::new(),
            };
            if !packages.is_empty() {
                for info in packages.clone().iter() {
                    if info.0 == package_id {
                        packages.remove(info);
                    }
                }
                <StoragePackages::<T>>::put(packages);
            }
            
            Ok(())
        }

        /// Purchase storage.
        #[pallet::call_index(31)]
        #[pallet::weight(T::WeightInfo::purchase_storage())]
        pub fn purchase_storage(
            origin: OriginFor<T>, 
            for_account: T::AccountId, 
            package_id: PackageId
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let package_option = Self::get_package(package_id);
            if package_option.is_none() {
                return Err(Error::<T>::StoragePackageNotExist)?;
            }
            
            let buy_package = package_option.unwrap();
            // check that the user's balance is sufficient 
            if T::Currency::can_slash(&who, buy_package.2) == false {
                return Err(Error::<T>::InsufficientBalance)?;
            }
            
            let is_exist = <WalletAccountStorage<T>>::contains_key(&for_account);
            let cur_number = frame_system::Pallet::<T>::block_number();
            // The first purchase 
            if is_exist {
                let mut pre_user = <WalletAccountStorage<T>>::get(&for_account).unwrap();
                if pre_user.parent_account != for_account {
                    pre_user = <WalletAccountStorage<T>>::get(&pre_user.parent_account).unwrap();
                }
                // Used space exceeds new subscription space size 
                if pre_user.used_space > buy_package.1 {
                    return Err(Error::<T>::ParamErr)?;
                }
                let mut from_number = pre_user.expire_number;
                if pre_user.expire_number < cur_number {
                    from_number = cur_number;
                }
                let mut new_user = UserStorage {
                    peers: pre_user.peers,
                    used_space: pre_user.used_space,
                    subscribe_space: buy_package.1,
                    subscribe_price: buy_package.2,
                    call_minus_number: buy_package.4,
                    nft_update_number: pre_user.nft_update_number,
                    db_update_number: pre_user.db_update_number,
                    expire_number: from_number.saturating_add(buy_package.3),
                    db_config: pre_user.db_config,
                    enc_nft_account: pre_user.enc_nft_account,
                    parent_account: pre_user.parent_account,
                    spam_frozen_status: pre_user.spam_frozen_status,
                    spam_report_amount: pre_user.spam_report_amount,
                    spam_report_number: pre_user.spam_report_number,
                    comment_frozen_status: pre_user.comment_frozen_status,
                    comment_report_amount: pre_user.comment_report_amount,
                    comment_report_number: pre_user.comment_report_number,
                    login_number: pre_user.login_number,
                    comment_space: pre_user.comment_space,
                    request_peers: pre_user.request_peers,
                };
                // The original subscription has not expired and has changed the storage package
                if pre_user.expire_number > cur_number
                   && (pre_user.subscribe_space != buy_package.1) {
                    // Convert remaining block number by new storage package 
                    let pre_space: u128 = u128::try_from(pre_user.subscribe_space).unwrap();
                    let now_space: u128 = u128::try_from(buy_package.1).unwrap();
                    let left_number = (pre_user.expire_number.saturating_sub(cur_number)).saturated_into::<u128>();
                    let rest_number = u32::try_from(pre_space.saturating_mul(left_number) / now_space).unwrap(); 

                    new_user.expire_number = cur_number.saturating_add(rest_number.into()).saturating_add(buy_package.3);
                }
                
                // Save user storage information.
                <WalletAccountStorage<T>>::insert(&new_user.parent_account.clone(), new_user);
            } else {
                let new_user = UserStorage {
                    peers: BTreeSet::new(),
                    used_space: 0,
                    subscribe_space: buy_package.1,
                    subscribe_price: buy_package.2,
                    call_minus_number: buy_package.4,
                    nft_update_number: cur_number,
                    db_update_number: cur_number,
                    expire_number: cur_number.saturating_add(buy_package.3),
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
                // Save user storage information.
                <WalletAccountStorage<T>>::insert(&for_account, new_user);
            }
            
            // The package fee is slashed from the user's account 
            T::Currency::slash(&who, buy_package.2);
            // Add package fee to The total amount(app/storage) of storage purchased by the user
            let app_reward = Perbill::from_rational(Self::app_reward_percent(), 100)*buy_package.2;
            <AppRewardTotal<T>>::put(Self::app_reward_total().saturating_add(app_reward));
            <StorageRewardTotal<T>>::put(Self::storage_reward_total().saturating_add(buy_package.2).saturating_sub(app_reward));

            // Emit an event.
            Self::deposit_event(Event::PurchaseStorage(package_id, for_account));
            Ok(())
        }

        /// Add the peer id of the storage node that the wallet account can request to.
        #[pallet::call_index(32)]
        #[pallet::weight(T::WeightInfo::add_request_peer_id_to_user())]
        pub fn add_request_peer_id_to_user(
            origin: OriginFor<T>, 
            for_account: T::AccountId, 
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_with_account(&who, &for_account, block_height, false)?;

            let mut message = DcString::new();
            message.extend(String::from("add_request_peer_id_to_user").as_bytes().to_vec().iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &for_account)?;

            if !<WalletAccountStorage<T>>::contains_key(&for_account) {
                Err(Error::<T>::AccountNotExist)?
            }
            let mut user_storage = <WalletAccountStorage<T>>::get(&for_account).unwrap();
            if user_storage.request_peers.len() >= USER_REQUEST_NODE_MAX_NUM {
                let mut tmp_set: BTreeSet<PeerId> = BTreeSet::<PeerId>::new();
                let mut change_num = USER_REQUEST_NODE_MAX_NUM;
                let mut set_iter = user_storage.request_peers.iter();
                // Increase the number of online nodes
                while change_num > 0 {
                    let cur_peer = set_iter.next_back();
                    // Jump the first member
                    if change_num == USER_REQUEST_NODE_MAX_NUM {
                        continue;
                    }
                    if cur_peer.is_some() {
                        let cur_peer_id = cur_peer.unwrap();
                        tmp_set.insert(cur_peer_id.to_vec());
                        change_num -= 1;
                    } else {
                        break;
                    }
                }
                user_storage.request_peers = tmp_set;
            }
            Self::change_used_space_expire_number(&for_account, 0 as SpaceSize, true, true)?;
            // Add the peer id 
            user_storage.request_peers.insert(peer_id);
            <WalletAccountStorage<T>>::insert(&for_account, user_storage);

            Ok(Pays::No.into())
        }

        /// Update the thread db config infomation of wallet account.
        #[pallet::call_index(33)]
        #[pallet::weight(T::WeightInfo::update_db_config())]
        pub fn update_db_config(
            origin: OriginFor<T>, 
            for_account: T::AccountId, 
            db_config: DcString,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_with_account(&who, &for_account, block_height, false)?;

            let mut message = DcString::new();
            message.extend(db_config.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &for_account)?;

            if !<WalletAccountStorage<T>>::contains_key(&for_account) {
                Err(Error::<T>::AccountNotExist)?
            }
            Self::change_used_space_expire_number(&for_account, 0 as SpaceSize, true, true)?;
            let mut user_storage = <WalletAccountStorage<T>>::get(&for_account).unwrap();
            // Update the thread db config infomation of wallet account
            user_storage.db_config = db_config;
            user_storage.db_update_number = block_height.into();
            <WalletAccountStorage<T>>::insert(&for_account, user_storage);

            Ok(Pays::No.into())
        }

        /// Create sub account.
        #[pallet::call_index(34)]
        #[pallet::weight(T::WeightInfo::create_sub_account())]
        pub fn create_sub_account(
            origin: OriginFor<T>, 
            parent_account: T::AccountId, 
            sub_account: T::AccountId,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let mut message = DcString::new();
            message.extend(sub_account.as_ref().iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &parent_account)?;

            let account_data = T::AccountStore::get(&sub_account);
            // The account has beed exist, can not be the sub-account
            if account_data != AccountData::default() {
                Err(Error::<T>::AccountAlreadyExist)?
            }
            
            let parent_opt = Self::wallet_account_storage(&parent_account);
            if parent_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let parent_info = parent_opt.unwrap();
            // A sub account cannot be used as a parent account  
            if parent_info.parent_account != parent_account {
                Err(Error::<T>::IsSubAccount)?
            }

            Self::change_used_space_expire_number(&parent_account, 0 as SpaceSize, true, true)?;
            let min_balance = T::Currency::minimum_balance();
            // Create sub account and tranfer minimum balance to the sub account
            T::Currency::transfer(&parent_account, &sub_account, min_balance, ExistenceRequirement::KeepAlive)?;
            
            let new_user = UserStorage {
                peers: BTreeSet::new(),
                used_space: 0,
                subscribe_space: 0,
                subscribe_price: Zero::zero(),
                call_minus_number: Zero::zero(),
                nft_update_number: frame_system::Pallet::<T>::block_number(),
                db_update_number: frame_system::Pallet::<T>::block_number(),
                expire_number: parent_info.expire_number,
                db_config: DcString::new(),
                enc_nft_account: NftAccount::new(),
                parent_account: parent_account,
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
            // Save user storage information.
            <WalletAccountStorage<T>>::insert(&sub_account, new_user);

            Ok(Pays::No.into())
        }

        /// Unbind sub account.
        #[pallet::call_index(35)]
        #[pallet::weight(T::WeightInfo::unbind_sub_account())]
        pub fn unbind_sub_account(
            origin: OriginFor<T>, 
            parent_account: T::AccountId, 
            sub_account: T::AccountId,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let mut message = DcString::new();
            message.extend(sub_account.as_ref().iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &parent_account)?;

            let parent_opt = Self::wallet_account_storage(&parent_account);
            if parent_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let mut parent_info = parent_opt.unwrap();
            // A sub account cannot be used as a parent account  
            if parent_info.parent_account != parent_account {
                Err(Error::<T>::IsSubAccount)?
            }

            let sub_opt = Self::wallet_account_storage(&sub_account);
            if sub_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let mut sub_info = sub_opt.unwrap();
            // Do not sub account of the parent  
            if sub_info.parent_account != parent_account {
                Err(Error::<T>::ParamErr)?
            }

            Self::change_used_space_expire_number(&parent_account, 0 as SpaceSize, true, true)?;
            let cur_number = frame_system::Pallet::<T>::block_number();
            parent_info.used_space = parent_info.used_space.saturating_sub(sub_info.used_space);
            sub_info.expire_number = cur_number;
            sub_info.parent_account = sub_account.clone();
            // Save user storage information.
            <WalletAccountStorage<T>>::insert(&parent_account, parent_info);
            <WalletAccountStorage<T>>::insert(&sub_account, sub_info);

            Ok(Pays::No.into())
        }

        /// Add peer in user storage information.
        #[pallet::call_index(36)]
        #[pallet::weight(T::WeightInfo::add_user_peer())]
        pub fn add_user_peer(
            origin: OriginFor<T>, 
            for_account: T::AccountId, 
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            // Insert peer into account storage
            Self::insert_user_peer(&for_account, peer_id)?;
            Ok(Pays::No.into())
        }

        /// Remove peer in user storage information by self node.
        #[pallet::call_index(37)]
        #[pallet::weight(T::WeightInfo::remove_self_user_peer())]
        pub fn remove_self_user_peer(
            origin: OriginFor<T>, 
            for_account: T::AccountId, 
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            Self::remove_user_peer(&peer_id, &for_account)?;
            
            Ok(Pays::No.into())
        }

        /// Remove peer in user storage information by other node.
        #[pallet::call_index(38)]
        #[pallet::weight(T::WeightInfo::remove_other_user_peer())]
        pub fn remove_other_user_peer(
            origin: OriginFor<T>, 
            peer_id: PeerId,
            for_account: T::AccountId, 
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            Self::check_node_status(&peer_id)?;
            Self::remove_user_peer(&peer_id, &for_account)?;
            
            Ok(Pays::No.into())
        }

        /// Apply NFT account.
        #[pallet::call_index(39)]
        #[pallet::weight(T::WeightInfo::apply_nft_account())]
        pub fn apply_nft_account(
            origin: OriginFor<T>, 
            nft_account: NftAccount, 
            for_account: T::AccountId,
            enc_nft_account: DcString,
            private_key_enc_hash: DcString,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_with_account(&who, &for_account, block_height, true)?;

            let mut message = DcString::new();
            message.extend(nft_account.iter().copied());
            message.extend(enc_nft_account.iter().copied());
            message.extend(private_key_enc_hash.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &for_account)?;
            
            if <NftToWalletAccount<T>>::contains_key(&nft_account) {
                Err(Error::<T>::NftAccoutApplied)?
            } else {
                <NftToWalletAccount<T>>::insert(&nft_account, &for_account);
            }

            // Update the encrypted NFT account to for_account
            Self::update_account_nft(&for_account, enc_nft_account, block_height, false)?;

            Self::change_used_space_expire_number(&for_account, 0 as SpaceSize, true, true)?;

            // Insert peer into account storage
            Self::insert_user_peer(&for_account, peer_id)?;

            Ok(Pays::No.into())
        }

        /// Transfer NFT account.
        #[pallet::call_index(40)]
        #[pallet::weight(T::WeightInfo::transfer_nft_account())]
        pub fn transfer_nft_account(
            origin: OriginFor<T>, 
            nft_account: NftAccount, 
            from_account: T::AccountId, 
            to_account: T::AccountId,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_with_account(&who, &from_account, block_height, true)?;
            let mut message = DcString::new();
            message.extend(nft_account.iter().copied());
            message.extend(to_account.as_ref().iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &from_account)?;
            if !<NftToWalletAccount<T>>::contains_key(&nft_account) {
                Err(Error::<T>::NftAccoutApplied)?
            }
            if !<WalletAccountStorage<T>>::contains_key(&to_account) {
                Err(Error::<T>::AccountNotExist)?
            }
            
            Self::change_used_space_expire_number(&from_account, 0 as SpaceSize, true, true)?;
            // Update the encrypted NFT account to for_account
            Self::update_account_nft(&from_account, NftAccount::new(), block_height, true)?;
            <NftToWalletAccount<T>>::insert(&nft_account, to_account.clone());
            Ok(Pays::No.into())
        }

        /// Update NFT account.
        #[pallet::call_index(41)]
        #[pallet::weight(T::WeightInfo::update_nft_account())]
        pub fn update_nft_account(
            origin: OriginFor<T>, 
            nft_account: NftAccount, 
            for_account: T::AccountId,
            enc_nft_account: DcString,
            private_key_enc_hash: DcString,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_with_account(&who, &for_account, block_height, true)?;

            let mut message = DcString::new();
            message.extend(nft_account.iter().copied());
            message.extend(enc_nft_account.iter().copied());
            message.extend(private_key_enc_hash.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &for_account)?;

            if !<NftToWalletAccount<T>>::contains_key(&nft_account)
               || <NftToWalletAccount<T>>::get(&nft_account).unwrap() != for_account {
                Err(Error::<T>::NftAccoutApplied)?
            }

            Self::change_used_space_expire_number(&for_account, 0 as SpaceSize, true, true)?;
            // Update the encrypted NFT account to for_account
            Self::update_account_nft(&for_account, enc_nft_account, block_height, false)?;

            Ok(Pays::No.into())
        }

        /// Add information of file.
        #[pallet::call_index(42)]
        #[pallet::weight(T::WeightInfo::add_file_info())]
        pub fn add_file_info(
            origin: OriginFor<T>, 
            owner: T::AccountId, 
            file_id: FileID, 
            file_size: SpaceSize, 
            file_type: u32,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let mut message = DcString::new();
            message.extend(file_id.iter().copied());
            message.extend(Self::u64_to_u8(file_size).iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(Self::u32_to_u8(file_type as u32).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &owner)?;

            // Chance used space and expire number of user's storage infomation
            Self::change_used_space_expire_number(&owner, file_size, true, true)?;

            let is_exist = <Files<T>>::contains_key(&file_id);
            if is_exist {
                let mut pre_info = <Files<T>>::get(&file_id).unwrap();
                pre_info.peers.insert(peer_id.clone());
                pre_info.users.insert(owner.clone());
                // Update storage.
                <Files<T>>::insert(file_id, pre_info);
            } else {
                let mut peers = BTreeSet::new();
                peers.insert(peer_id.clone());
                let mut users = BTreeSet::new();
                users.insert(owner.clone());
                let new_info = FileInfo {
                    peers: peers,
                    users: users,
                    file_size: file_size,
                    file_type: file_type,
                    db_log: BTreeSet::new(),
                };
                // Update storage.
                <Files<T>>::insert(&file_id, new_info);
            }

            Ok(Pays::No.into())
        }

        /// Add peer of file.
        #[pallet::call_index(43)]
        #[pallet::weight(T::WeightInfo::add_file_peer())]
        pub fn add_file_peer(
            origin: OriginFor<T>, 
            file_id: FileID, 
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let is_exist = <Files<T>>::contains_key(&file_id);
            if !is_exist {
                Err(Error::<T>::FileNotExist)?
            }
            let mut pre_info = <Files<T>>::get(&file_id).unwrap();
            if pre_info.file_type == FILE_TYPE_THREAD_DB {
                Err(Error::<T>::FileTypeError)?
            }
            pre_info.peers.insert(peer_id);
            // Update storage.
            <Files<T>>::insert(&file_id, pre_info);
                      
            Ok(Pays::No.into())
        }

        /// Remove peer of a file by self node.
        #[pallet::call_index(44)]
        #[pallet::weight(T::WeightInfo::remove_self_file_peer())]
        pub fn remove_self_file_peer(
            origin: OriginFor<T>, 
            file_id: FileID, 
            _file_type: u32,
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;
            Self::remove_file_peer(&peer_id, &file_id)?;
            
            Ok(Pays::No.into())
        }

        /// Remove peer of a file by other node.
        #[pallet::call_index(45)]
        #[pallet::weight(T::WeightInfo::remove_self_file_peer())]
        pub fn remove_other_file_peer(
            origin: OriginFor<T>, 
            peer_id: PeerId,
            file_id: FileID, 
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;

            Self::check_node_status(&peer_id)?;
            Self::remove_file_peer(&peer_id, &file_id)?;
            
            Ok(Pays::No.into())
        }

        /// Delete file info.
        #[pallet::call_index(46)]
        #[pallet::weight(T::WeightInfo::delete_file_info())]
        pub fn delete_file_info(
            origin: OriginFor<T>, 
            owner: T::AccountId, 
            file_id: FileID,
            file_type: u32,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let mut message = DcString::new();
            message.extend(file_id.iter().copied());
            message.extend(Self::u32_to_u8(file_type).iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            Self::verify(&signature, &message, &owner)?;

            let is_exist = <Files<T>>::contains_key(&file_id);
            if is_exist {
                let mut pre_info = <Files<T>>::get(&file_id).unwrap();
                // Chance used space and expire number of user's storage infomation
                Self::change_used_space_expire_number(&owner, pre_info.file_size, false, true)?;
                pre_info.peers.remove(&peer_id);
                pre_info.users.remove(&owner);
                if pre_info.peers.len() > 0 {
                    // Update storage.
                    <Files<T>>::insert(file_id, pre_info.clone());
                } else {
                    <Files<T>>::remove(file_id);
                }
            } else {
                Err(Error::<T>::FileNotExist)?
            }
            
            Ok(Pays::No.into())
        }

        /// Add db log information of thread db file.
        #[pallet::call_index(47)]
        #[pallet::weight(T::WeightInfo::add_log_to_thread_db())]
        pub fn add_log_to_thread_db(
            origin: OriginFor<T>, 
            file_id: FileID, 
            log_id: DcString,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;

            let mut message = DcString::new();
            message.extend(file_id.iter().copied());
            message.extend(log_id.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peer_id.iter().copied());

            let is_exist = <Files<T>>::contains_key(&file_id);
            if is_exist {
                let mut pre_info = <Files<T>>::get(&file_id).unwrap();
                let users =  pre_info.users.clone();
                let mut users_iter = users.iter();
                let owner = users_iter.next().unwrap();
                Self::verify(&signature, &message, &owner)?;
                // Chance used space and expire number of user's storage infomation
                Self::change_used_space_expire_number(&owner, 0 as SpaceSize, true, true)?;

                pre_info.db_log.insert(log_id);
                // Update storage.
                <Files<T>>::insert(file_id, pre_info);
            } else {
                Err(Error::<T>::FileNotExist)?
            }

            Ok(Pays::No.into())
        }

        /// Update the thread db log infomation of user.
        #[pallet::call_index(48)]
        #[pallet::weight(T::WeightInfo::add_space_to_thread_db())]
        pub fn add_space_to_thread_db(
            origin: OriginFor<T>, 
            file_id: FileID,
            increase_size: u32,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;
            
            let mut message = DcString::new();
            message.extend(file_id.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(Self::u32_to_u8(increase_size).iter().copied());
            message.extend(peer_id.iter().copied());

            let is_exist = <Files<T>>::contains_key(&file_id);
            if is_exist {
                let mut pre_info = <Files<T>>::get(&file_id).unwrap();
                let users =  pre_info.users.clone();
                let mut users_iter = users.iter();
                let owner = users_iter.next().unwrap();
                Self::verify(&signature, &message, &owner)?;
                pre_info.file_size = pre_info.file_size.saturating_add(increase_size as SpaceSize);
                // Chance used space and expire number of user's storage infomation
                Self::change_used_space_expire_number(&owner, increase_size as SpaceSize, true, true)?;
                // Update storage.
                <Files<T>>::insert(&file_id, pre_info);
            } else {
                Err(Error::<T>::FileNotExist)?
            }

            Ok(Pays::No.into())
        }

        /// Report file missing.
        #[pallet::call_index(49)]
        #[pallet::weight(T::WeightInfo::report_file_miss())]
        pub fn report_file_miss(
            origin: OriginFor<T>,
            file_id: FileID,
            _file_type: u32,
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;
            Self::remove_file_peer(&peer_id, &file_id)?;
            
            Ok(Pays::No.into())
        }

        /// Report the loss of login information to the blockchain.
        #[pallet::call_index(50)]
        #[pallet::weight(T::WeightInfo::report_login_info_miss())]
        pub fn report_login_info_miss(
            origin: OriginFor<T>,
            for_account: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            let peer_id = Self::get_peer_id_with_req_acc_id(&who)?;
            Self::remove_account_peer(&peer_id, &for_account)?;
            Ok(Pays::No.into())
        }

        /// Report tee faking of storage node.
        #[pallet::call_index(51)]
        #[pallet::weight(T::WeightInfo::report_tee_faking())]
        pub fn report_tee_faking(
            origin: OriginFor<T>,
            peer_id: PeerId, 
            _ext_height: T::BlockNumber,
            _ext_num: u32,
            _ext_tee_report_hash: DcString,
            _ext_tee_report: DcString,
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            Self::check_peer_request_without_account(&who, block_height)?;

            Self::process_report(who.clone(), &peer_id, ReportType::ReportTeeFaking)?;
            Ok(Pays::No.into())
        }

        /// Verify tee faking of storage node.
        #[pallet::call_index(52)]
        #[pallet::weight(T::WeightInfo::verify_tee_faking())]
        pub fn verify_tee_faking(
            origin: OriginFor<T>,
            peer_id: PeerId, 
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            Self::check_peer_request_without_account(&who, block_height)?;
            Self::process_report(who.clone(), &peer_id, ReportType::VerifyTeeFaking)?;

            Ok(Pays::No.into())
        }

        /// Report a storage node offchain to the chain
        #[pallet::call_index(53)]
        #[pallet::weight(T::WeightInfo::report_peer_offchain())]
        pub fn report_peer_offchain(
            origin: OriginFor<T>,
            peer_id: PeerId, 
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            Self::check_peer_request_without_account(&who, block_height)?;

            Self::process_report(who.clone(), &peer_id, ReportType::ReportPeerOffchain)?;

            Ok(Pays::No.into())
        }

        /// Report a storage node no response to the chain
        #[pallet::call_index(54)]
        #[pallet::weight(T::WeightInfo::report_peer_no_response())]
        pub fn report_peer_no_response(
            origin: OriginFor<T>,
            peer_id: PeerId, 
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            Self::check_peer_request_without_account(&who, block_height)?;

            Self::process_report(who.clone(), &peer_id, ReportType::ReportPeerNoResponse)?;

            Ok(Pays::No.into())
        }

        /// Report a storage node error to the chain.
        #[pallet::call_index(55)]
        #[pallet::weight(T::DbWeight::get().reads_writes(3, 1))]
        pub fn report_peer_error(
            origin: OriginFor<T>,
            peer_id: PeerId, 
            block_height: u32,
            _tee_report: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            Self::check_peer_request_without_account(&who, block_height)?;

            let peer_ret = Self::peers(&peer_id);
            if peer_ret.is_none() {
                Err(Error::<T>::PeerIdNotExist)?
            }
            let cur_num = frame_system::Pallet::<T>::block_number();
            let mut peer_info = peer_ret.unwrap();
            // After the node is offchain for a period of time, set the node status to abnormal
            if peer_info.status == NODE_STATUS_OFFCHAIN
               && peer_info.report_number < cur_num.saturating_sub(Self::blocks_of_offchain_to_abnormal()) {
                T::StakingProvider::report_offence(&peer_info.stash, T::StakingProvider::get_staking_active(&peer_info.stash));
                peer_info.status = NODE_STATUS_ABNORMAL;
                <Peers<T>>::insert(peer_id.clone(), peer_info);
            } else {
                Err(Error::<T>::ErrorNodeReport)?
            }

            Ok(Pays::No.into())
        }

        /// Report spam of user.
        #[pallet::call_index(56)]
        #[pallet::weight(T::WeightInfo::report_spam())]
        pub fn report_spam(
            origin: OriginFor<T>,
            report_account: T::AccountId, 
            report_block_height: u32,
            report_signature: DcString,
            msg_id: DcString,
            sender_account: T::AccountId,
            app_id: DcString,
            msg_block_height: u32,
            msg_encrypt: DcString,
            msg_signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let m_block_height: T::BlockNumber = msg_block_height.into();
            let cur_num = frame_system::Pallet::<T>::block_number();
            let can_not_report = Self::interval_blocks_can_not_report();
            if cur_num > can_not_report && m_block_height < cur_num.saturating_sub(can_not_report) {
                return Ok(Pays::No.into());
            }
            // Check report signature
            let mut rep_message = DcString::new();
            rep_message.extend(msg_id.iter().copied());
            rep_message.extend(Self::u32_to_u8(report_block_height).iter().copied());

            Self::verify(&report_signature, &rep_message, &report_account)?;

            // Check message signature
            let mut sender_message = DcString::new();
            sender_message.extend(msg_id.iter().copied());
            sender_message.extend(app_id.iter().copied());
            sender_message.extend(Self::u32_to_u8(msg_block_height).iter().copied());
            sender_message.extend(msg_encrypt.iter().copied());

            Self::verify(&msg_signature, &sender_message, &sender_account)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, report_block_height)?;
            
            // Check if reporting user is stored in peer node
            let report_opt = Self::wallet_account_storage(&report_account);
            if report_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let report_info = report_opt.unwrap();
            if !report_info.peers.contains(&peer_id) {
                Err(Error::<T>::PeerIdNotExist)?
            }
            
            // Set frozen status and report number of the sender
            let sender_opt = <WalletAccountStorage<T>>::get(&sender_account);
            if sender_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let mut sender_info = sender_opt.unwrap();
            if sender_info.spam_report_number == 0u32.into() {
                sender_info.spam_report_number = frame_system::Pallet::<T>::block_number();
            }
            if sender_info.spam_frozen_status == 0 {
                sender_info.spam_report_amount += 1;
            }
            if sender_info.spam_report_amount >= Self::frozen_report_spam_amount() {
                sender_info.spam_frozen_status = 1;
            }
            <WalletAccountStorage<T>>::insert(&sender_account, sender_info);

            Ok(Pays::No.into())
        }

        /// set the account information associated with the app id.
        #[pallet::call_index(57)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn set_app_account(
            origin: OriginFor<T>,
            app_id: DcString,
            rewarded_account: T::AccountId, 
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            if app_id.len() > APPID_MAX_LENGTH.try_into().unwrap() {
                Err(Error::<T>::AppIdLengthErr)?
            }
            let is_exist = <AccountOfApp<T>>::contains_key(&app_id);
            if is_exist {
                let acc_info = Self::account_of_app(&app_id).unwrap();
                if acc_info.private_account == who {
                    <AccountOfApp<T>>::insert(app_id, AppAccountInfo{private_account: who, rewarded_stash: rewarded_account});
                } else {
                    Err(Error::<T>::NotController)?
                }
            } else {
                <AccountOfApp<T>>::insert(app_id, AppAccountInfo{private_account: who, rewarded_stash: rewarded_account});
            }
            Ok(Pays::No.into())
        }

        /// User login
        #[pallet::call_index(58)]
        #[pallet::weight(T::WeightInfo::user_login(app_ids.len().try_into().unwrap()))]
        pub fn user_login(
            origin: OriginFor<T>,
            login_account: T::AccountId, 
            app_ids: Vec<DcString>, 
            block_height: u32,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, block_height)?;
            
            // Check if the user is stored in peer node
            let user_opt = Self::wallet_account_storage(&login_account);
            if user_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let cur_number = frame_system::Pallet::<T>::block_number();
            let mut user_info = user_opt.unwrap();
            if !user_info.peers.contains(&peer_id) {
                Err(Error::<T>::PeerIdNotExist)?
            }
            if cur_number.saturating_sub(user_info.login_number) < Self::interval_blocks_login().saturating_sub(300u32.into()) {
                Err(Error::<T>::ExcessiveLogin)?
            }
            
            user_info.login_number = block_height.into();

            // Reduce the number of spam reports
            if user_info.spam_report_amount > 0 {
                let interval_number = cur_number.saturating_sub(user_info.spam_report_number);
                let left_number = interval_number % Self::interval_blocks_reduce_spam();
                let reduce_amount: u32 = (interval_number / Self::interval_blocks_reduce_spam()).saturated_into::<u32>();

                if user_info.spam_report_amount > reduce_amount {
                    user_info.spam_report_amount = user_info.spam_report_amount.saturating_sub(reduce_amount);
                } else {
                    user_info.spam_report_amount = 0;
                }
                user_info.spam_report_number = cur_number.saturating_sub(left_number);
            }

            // Reduce the number of comment reports
            if user_info.comment_report_amount > 0 {
                let interval_number = cur_number.saturating_sub(user_info.comment_report_number);
                let left_number = interval_number % Self::interval_blocks_reduce_comment();
                let reduce_amount: u32 = (interval_number / Self::interval_blocks_reduce_comment()).saturated_into::<u32>();

                if user_info.comment_report_amount > reduce_amount {
                    user_info.comment_report_amount = user_info.comment_report_amount.saturating_sub(reduce_amount);
                } else {
                    user_info.comment_report_amount = 0;
                }
                user_info.comment_report_number = cur_number.saturating_sub(left_number);
            }

            <WalletAccountStorage<T>>::insert(&login_account, user_info);

            let mut login_count: BTreeMap<AppID, AppLoginInfo<T::AccountId>> = match <AppsAccountLoginTimes::<T>>::get() {
                Some(p) => p,
                None => BTreeMap::<AppID, AppLoginInfo<T::AccountId>>::new(),
            };
            
            let temp_count = login_count.clone();
            // Set the times of logins for app
            for app_id in &app_ids {
                let is_exist = <AccountOfApp<T>>::contains_key(&app_id);
                if is_exist {
                    // Get the config infomation(stash account, etc) of app id.
                    let acc_info = Self::account_of_app(&app_id).unwrap();
                    if temp_count.contains_key(app_id) {
                        let login_info = temp_count.get(app_id).unwrap();
                        login_count.insert(app_id.to_vec(), AppLoginInfo{rewarded_stash: acc_info.rewarded_stash, login_times: login_info.login_times+1});
                    } else {
                        login_count.insert(app_id.to_vec(), AppLoginInfo{rewarded_stash: acc_info.rewarded_stash, login_times: 1});
                    }
                }
            }
            
            <AppsAccountLoginTimes::<T>>::put(login_count);

            Ok(Pays::No.into())
        }

        /// New a theme.
        #[pallet::call_index(59)]
        #[pallet::weight(T::WeightInfo::new_theme())]
        pub fn new_theme(
            origin: OriginFor<T>, 
            for_account: T::AccountId,
            theme_id: DcString, 
            app_id: DcString,
            comment_space: SpaceSize,
            open_flag: u32,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let _ = Self::check_peer_request_without_account(&who, block_height)?;
            
            let mut message = DcString::new();
            message.extend(theme_id.iter().copied());
            message.extend(app_id.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(Self::u64_to_u8(comment_space).iter().copied());
            message.extend(Self::u32_to_u8(open_flag).iter().copied());

            Self::verify(&signature, &message, &for_account)?;
            if !<WalletAccountStorage<T>>::contains_key(&for_account) {
                Err(Error::<T>::AccountNotExist)?
            }
            Self::change_used_space_expire_number(&for_account, comment_space as SpaceSize, true, false)?;
            let mut user_storage = <WalletAccountStorage<T>>::get(&for_account).unwrap();
            user_storage.comment_space = user_storage.comment_space.saturating_add(comment_space);
            <WalletAccountStorage<T>>::insert(&for_account, user_storage);

            Ok(Pays::No.into())
        }

        /// Add the comment space of a theme.
        #[pallet::call_index(60)]
        #[pallet::weight(T::WeightInfo::add_theme_comment_space())]
        pub fn add_theme_comment_space(
            origin: OriginFor<T>, 
            for_account: T::AccountId,
            theme_id: DcString, 
            app_id: DcString,
            add_space: SpaceSize,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let _ = Self::check_peer_request_without_account(&who, block_height)?;
            
            let mut message = DcString::new();
            message.extend(theme_id.iter().copied());
            message.extend(app_id.iter().copied());
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(Self::u64_to_u8(add_space).iter().copied());
            Self::verify(&signature, &message, &for_account)?;

            if !<WalletAccountStorage<T>>::contains_key(&for_account) {
                Err(Error::<T>::AccountNotExist)?
            }

            Self::change_used_space_expire_number(&for_account, add_space as SpaceSize, true, false)?;
            let mut user_storage = <WalletAccountStorage<T>>::get(&for_account).unwrap();
            user_storage.comment_space = user_storage.comment_space.saturating_add(add_space);
            <WalletAccountStorage<T>>::insert(&for_account, user_storage);

            Ok(Pays::No.into())
        }

        /// Add the comment space of user.
        #[pallet::call_index(61)]
        #[pallet::weight(T::WeightInfo::add_user_comment_space())]
        pub fn add_user_comment_space(
            origin: OriginFor<T>, 
            for_account: T::AccountId,
            block_height: u32,
            signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let peerid = Self::check_peer_request_without_account(&who, block_height)?;
            
            let mut message = DcString::new();
            message.extend(Self::u32_to_u8(block_height).iter().copied());
            message.extend(peerid.iter().copied());
            Self::verify(&signature, &message, &for_account)?;

            if !<WalletAccountStorage<T>>::contains_key(&for_account) {
                Err(Error::<T>::AccountNotExist)?
            }
            let reduce_space = Self::comment_reduce_space();
            Self::change_used_space_expire_number(&for_account, reduce_space as SpaceSize, true, false)?;
            let mut user_storage = <WalletAccountStorage<T>>::get(&for_account).unwrap();
            user_storage.comment_space = user_storage.comment_space.saturating_add(reduce_space);
            <WalletAccountStorage<T>>::insert(&for_account, user_storage);

            Ok(Pays::No.into())
        }

        /// Report malicious comments.
        #[pallet::call_index(62)]
        #[pallet::weight(T::WeightInfo::report_malicious_comment())]
        pub fn report_malicious_comment(
            origin: OriginFor<T>,
            report_account: T::AccountId, 
            report_block_height: u32,
            report_signature: DcString,
            theme_id: DcString,
            content_id: DcString,
            comment_account: T::AccountId,
            app_id: DcString,
            comment_block_height: u32,
            refer_comment_key: DcString,
            content_type: u32,
            comment_signature: DcString,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let c_block_height: T::BlockNumber = comment_block_height.into();
            let cur_num = frame_system::Pallet::<T>::block_number();
            let can_not_report = Self::interval_blocks_can_not_report();
            if cur_num > can_not_report && c_block_height < cur_num.saturating_sub(can_not_report) {
                return Ok(Pays::No.into());
            }
            // Check report signature
            let mut rep_message = DcString::new();
            rep_message.extend(theme_id.iter().copied());
            rep_message.extend(app_id.iter().copied());
            rep_message.extend(Self::u32_to_u8(report_block_height).iter().copied());
            rep_message.extend(content_id.iter().copied());

            Self::verify(&report_signature, &rep_message, &report_account)?;

            // Check comment signature
            let mut comment_message = DcString::new();
            comment_message.extend(theme_id.iter().copied());
            comment_message.extend(app_id.iter().copied());
            comment_message.extend(Self::u32_to_u8(comment_block_height).iter().copied());
            comment_message.extend(content_id.iter().copied());
            comment_message.extend(refer_comment_key.iter().copied());
            comment_message.extend(Self::u32_to_u8(content_type).iter().copied());

            Self::verify(&comment_signature, &comment_message, &comment_account)?;

            // Check params and get the peer Id
            let peer_id = Self::check_peer_request_without_account(&who, report_block_height)?;

            // Check if reporting user is stored in peer node
            let report_opt = Self::wallet_account_storage(&report_account);
            if report_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let report_info = report_opt.unwrap();
            if !report_info.peers.contains(&peer_id) {
                Err(Error::<T>::PeerIdNotExist)?
            }
            
            // Set frozen status and report number of the comment user
            let commenter_opt = <WalletAccountStorage<T>>::get(&comment_account);
            if commenter_opt.is_none() {
                Err(Error::<T>::AccountNotExist)?
            }

            let mut commenter_info = commenter_opt.unwrap();
            if commenter_info.comment_report_number == 0u32.into() {
                commenter_info.comment_report_number = frame_system::Pallet::<T>::block_number();
            }
            
            if commenter_info.comment_frozen_status == 0 {
                commenter_info.comment_report_amount += 1;
            }
            if commenter_info.comment_report_amount >= Self::frozen_report_comment_amount() {
                commenter_info.comment_frozen_status = 1;
            }
            <WalletAccountStorage<T>>::insert(&comment_account, commenter_info);

            Ok(Pays::No.into())
        }
    }
}

impl<T: Config> Pallet<T> 
    where 
        T::AccountId: AsRef<[u8]>,
{
    /// Get the storage package by id
    fn get_package(package_id: PackageId) -> Option<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)> {
        let packages: BTreeSet<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)> = match Self::storage_packages() {
            Some(p) => p,
            None => BTreeSet::<(PackageId, SpaceSize, BalanceOf<T>, T::BlockNumber, T::BlockNumber)>::new(),
        };
        if packages.is_empty() {
            return None;
        }
        for info in packages.iter() {
            if info.0 == package_id {
                return Some(*info);
            }
        }
        None
    }

    /// Chance used space and expire number of user's storage infomation
    fn change_used_space_expire_number(owner: &T::AccountId, file_size: SpaceSize, is_add: bool, is_reduce_expire: bool) -> DispatchResult {
        // consume user space 
        let owner_opt = <WalletAccountStorage<T>>::get(&owner);
        if owner_opt.is_none() {
            Err(Error::<T>::AccountNotExist)?
        }

        let mut owner_info = owner_opt.unwrap();
        if owner_info.expire_number < frame_system::Pallet::<T>::block_number()
           || owner_info.expire_number.saturating_sub(frame_system::Pallet::<T>::block_number()) < owner_info.call_minus_number {
            Err(Error::<T>::UserPackageExpired)?
        }
        if file_size > 0 as SpaceSize || (owner_info.parent_account == *owner && is_reduce_expire) {
            if is_add {
                owner_info.used_space = owner_info.used_space.saturating_add(file_size);
            } else {
                owner_info.used_space = owner_info.used_space.saturating_sub(file_size);
            }
            if owner_info.parent_account == *owner && is_reduce_expire {
                let minus_number: u128 = T::BlockMultiplier::get_next_fee_multiplier().saturating_mul_int(owner_info.call_minus_number.saturated_into::<u128>());
                owner_info.expire_number = owner_info.expire_number.saturating_sub(u32::try_from(minus_number).unwrap().into());
            }
            <WalletAccountStorage<T>>::insert(&owner, owner_info.clone());
        }
        
        // It is a sub account 
        if owner_info.parent_account != *owner {
            // consume parent user space
            let parent_opt = <WalletAccountStorage<T>>::get(&owner_info.parent_account);
            if parent_opt.is_some() {
                let mut parent_info = parent_opt.unwrap();
                if parent_info.expire_number < frame_system::Pallet::<T>::block_number()
                   || parent_info.expire_number.saturating_sub(frame_system::Pallet::<T>::block_number()) < parent_info.call_minus_number {
                    Err(Error::<T>::UserPackageExpired)?
                }
                if is_add {
                    parent_info.used_space = parent_info.used_space.saturating_add(file_size);
                } else {
                    parent_info.used_space = parent_info.used_space.saturating_sub(file_size);
                }
                if is_reduce_expire {
                    let minus_number: u128 = T::BlockMultiplier::get_next_fee_multiplier().saturating_mul_int(parent_info.call_minus_number.saturated_into::<u128>());
                    parent_info.expire_number = parent_info.expire_number.saturating_sub(u32::try_from(minus_number).unwrap().into());
                }
                <WalletAccountStorage<T>>::insert(&owner_info.parent_account, parent_info);
            }
        }
        Ok(())
    }

    /// Insert peer into account storage
    fn insert_user_peer(for_account: &T::AccountId, peer_id: PeerId) -> DispatchResult {
        let is_exist = <WalletAccountStorage<T>>::contains_key(for_account);
        if !is_exist {
            Err(Error::<T>::AccountNotExist)?
        }
        let mut pre_info = <WalletAccountStorage<T>>::get(for_account).unwrap();
        pre_info.peers.insert(peer_id);
        // Update storage.
        <WalletAccountStorage<T>>::insert(for_account, pre_info);
        Ok(())
    }

    /// Update the encrypted NFT account to for_account
    fn update_account_nft(
        for_account: &T::AccountId,
        enc_nft_account: DcString, 
        block_height: u32,
        is_clear_peer: bool,
    ) -> DispatchResult {
        let user_opt = <WalletAccountStorage<T>>::get(for_account);
        if user_opt.is_none() {
            Err(Error::<T>::AccountNotExist)?
        }
        let mut user_storage = user_opt.unwrap();
        // Update the encrypted NFT account to for_account
        user_storage.enc_nft_account = enc_nft_account;
        user_storage.nft_update_number = block_height.into();
        if is_clear_peer {
            user_storage.peers = BTreeSet::<PeerId>::new();
        }
        <WalletAccountStorage<T>>::insert(for_account, user_storage);
        Ok(())
    }

    /// u32 to [u8]
    fn u32_to_u8(v: u32) -> [u8; 4] {
        unsafe {
            let u32_ptr: *const u32 = &v as *const u32;
            let u8_ptr: *const u8 = u32_ptr as *const u8;
            return [*u8_ptr.offset(0), *u8_ptr.offset(1), *u8_ptr.offset(2), *u8_ptr.offset(3)];
        }
    }

    /// u64 to [u8]
    fn u64_to_u8(v: u64) -> [u8; 8] {
        unsafe {
            let u64_ptr: *const u64 = &v as *const u64;
            let u8_ptr: *const u8 = u64_ptr as *const u8;
            return [*u8_ptr.offset(0), *u8_ptr.offset(1), *u8_ptr.offset(2), *u8_ptr.offset(3),
                    *u8_ptr.offset(4), *u8_ptr.offset(5), *u8_ptr.offset(6), *u8_ptr.offset(7)];
        }
    }

    /// Get peer ID by request account ID
    fn get_peer_id_with_req_acc_id(req_account_id: &T::AccountId) -> Result<PeerId, Error<T>> {
        // Get the peer ID
        let peer_id_ret = Self::request_account_peer(req_account_id);
        if peer_id_ret.is_none() {
            Err(Error::<T>::AccountNotExist)?
        }
        let peer_id = peer_id_ret.unwrap();
        // Get the peer infomation
        let peer_ret = Self::peers(&peer_id);
        if peer_ret.is_none() {
            Err(Error::<T>::PeerIdNotExist)?
        }

        let peer_info = peer_ret.unwrap();
        if peer_info.status != NODE_STATUS_ONCHAIN {
            Err(Error::<T>::NodeStatusError)?
        }

        Ok(peer_id)
    }

    /// Check the block number of wallet account
    fn check_account_block_number(for_account: &T::AccountId, block_height: u32, is_nft: bool) -> Result<(), Error<T>> {
        let block_num :T::BlockNumber = block_height.into();
        // Get the storage information of for_account
        let is_exist = <WalletAccountStorage<T>>::contains_key(for_account);
        // The first purchase 
        if !is_exist {
            Err(Error::<T>::AccountNotExist)?
        }
        let for_strorage = Self::wallet_account_storage(for_account).unwrap();
        // Check the block number
        if (is_nft && block_num < for_strorage.nft_update_number)
           || (!is_nft && block_num < for_strorage.db_update_number) {
            Err(Error::<T>::BlockNumberInvalid)?
        }

        if frame_system::Pallet::<T>::block_number() > for_strorage.expire_number {
            Err(Error::<T>::UserPackageExpired)?
        }
        Ok(())
    }

    /// Check the block number of request
    fn check_request_block_number(block_height: u32) -> Result<(), Error<T>> {
        let block_num :T::BlockNumber = block_height.into();
        let cur_num = frame_system::Pallet::<T>::block_number();
        let valid_num = Self::valid_call_block_number();
        // Check the block number
        if cur_num > valid_num && (block_num < cur_num.saturating_sub(valid_num) || block_num > cur_num) {
            Err(Error::<T>::BlockNumberInvalid)?
        }
        Ok(())
    }

    /// Check params and get the peer Id
    fn check_peer_request_with_account(
        req_account_id: &T::AccountId, 
        for_account: &T::AccountId, 
        block_height: u32,
        is_nft: bool,
    ) -> Result<PeerId, Error<T>> {
        // Check the block number of request
        Self::check_request_block_number(block_height)?;
        // Get peer ID by request account ID
        let peer_id = Self::get_peer_id_with_req_acc_id(req_account_id)?;
        // Check the block number of wallet account
        Self::check_account_block_number(for_account, block_height, is_nft)?;
        Ok(peer_id)
    }

    /// Check params and get the peer Id
    fn check_peer_request_without_account(req_account_id: &T::AccountId, block_height: u32) -> Result<PeerId, Error<T>> {
        // Check the block number of request
        Self::check_request_block_number(block_height)?;
        // Get peer ID by request account ID
        let peer_id = Self::get_peer_id_with_req_acc_id(req_account_id)?;
        Ok(peer_id)
    }

    /// Data signature verify
    #[cfg(all(not(feature = "runtime-benchmarks"), not(test)))]
    fn verify(signature: &DcString, message: &DcString, account: &T::AccountId) -> DispatchResult {
        let sig_ret = ed25519::Signature::try_from(signature.as_ref());
        if sig_ret.is_err() {
            Err(Error::<T>::DataSignatureVerify)?
        }
        let pub_key = ed25519::Public::try_from(account.as_ref());
        if pub_key.is_err() {
            Err(Error::<T>::DataSignatureVerify)?
        }
        
        let is_sign_ok = ed25519_verify(&sig_ret.unwrap(), message, &pub_key.unwrap());
        if !is_sign_ok {
            Err(Error::<T>::DataSignatureVerify)?
        }
        Ok(())
    }
    
    #[cfg(any(feature = "runtime-benchmarks", test))]
    fn verify(_signature: &DcString, _message: &DcString, _account: &T::AccountId) -> DispatchResult {
        Ok(())
    }

    /// Parse file ids from string to array.
    // fn parse_file_ids(ids_str: &DcString) -> BTreeSet<DcString> {
    //     let mut ids_set = BTreeSet::<DcString>::new();
    //     if ids_str.len() == 0 {
    //         return ids_set;
    //     }
    //     // Create a str slice.
	// 	let ret = sp_std::str::from_utf8(ids_str);
    //     if ret.is_err() {
    //         return ids_set;
    //     }

    //     let val = lite_json::parse_json(ret.unwrap());
    //     if val.is_err() {
    //         return ids_set;
    //     }
	// 	match val.unwrap() {
	// 		JsonValue::Array(obj) => {
	// 			for file_id in obj.iter() {
    //                 match file_id {
    //                     JsonValue::String(id) => ids_set.insert(id.iter().map(|c| *c as u8).collect::<Vec<_>>()),
    //                     _ => true,
    //                 };
    //             }
    //             return ids_set;
	// 		},
	// 		_ => return ids_set,
	// 	}
    // }

    /// Check the status of storage node
    fn check_node_status(peer_id: &PeerId) -> DispatchResult {
        let is_exist = <Peers<T>>::contains_key(peer_id);
        if is_exist {
            let peer_info = <Peers<T>>::get(&peer_id).unwrap();
            if peer_info.status == NODE_STATUS_ABNORMAL
                || peer_info.status == NODE_STATUS_CLOSED
                || peer_info.status == NODE_STATUS_DISCARD {
                    Ok(())
            } else {
                Err(Error::<T>::NodeStatusError)?
            }
        } else {
            Err(Error::<T>::PeerIdNotExist)?
        }
    }

    /// Remove peer of a file.
    fn remove_file_peer(
        peer_id: &PeerId, 
        file_id: &FileID, 
    ) -> DispatchResult {
        let is_exist = <Files<T>>::contains_key(file_id);
        if !is_exist {
            Err(Error::<T>::FileNotExist)?
        }
        let mut pre_info = <Files<T>>::get(file_id).unwrap();
        if !pre_info.peers.contains(peer_id) {
            Err(Error::<T>::PeerIdNotExist)?
        }
        pre_info.peers.remove(peer_id);
        if pre_info.peers.len() > 0 {
            // Update storage.
            <Files<T>>::insert(file_id, pre_info);
        } else {
            <Files<T>>::remove(file_id);
            if pre_info.users.len() > 0 {
                let mut users_iter = pre_info.users.iter();
                // Chance used space and expire number of user's storage infomation
                Self::change_used_space_expire_number(&users_iter.next().unwrap(), pre_info.file_size, false, false)?;
            }
        }

        Ok(())
    }

    pub fn remove_account_peer(
        peer_id: &PeerId, 
        for_account: &T::AccountId,
    ) -> DispatchResult {
        let is_exist = <WalletAccountStorage<T>>::contains_key(for_account);
        if !is_exist {
            Err(Error::<T>::AccountNotExist)?
        }
        let mut pre_info = <WalletAccountStorage<T>>::get(&for_account).unwrap();
        if !pre_info.peers.contains(peer_id) {
            Err(Error::<T>::PeerIdNotExist)?
        }
        pre_info.peers.remove(peer_id);
        <WalletAccountStorage<T>>::insert(for_account, pre_info);
        Ok(())
    }

    /// Remove peer in user storage information.
    fn remove_user_peer(
        peer_id: &PeerId, 
        for_account: &T::AccountId,
    ) -> DispatchResult {
        let is_exist = <WalletAccountStorage<T>>::contains_key(for_account);
        if !is_exist {
            Err(Error::<T>::AccountNotExist)?
        }
        let mut pre_info = <WalletAccountStorage<T>>::get(for_account).unwrap();
        if !pre_info.peers.contains(peer_id) {
            Err(Error::<T>::PeerIdNotExist)?
        }
        pre_info.peers.remove(peer_id);
        // Update storage.
        <WalletAccountStorage<T>>::insert(for_account, pre_info);

        Ok(())
    }

    /// Process report data 
    fn process_report(report_account: T::AccountId, peer_id: &PeerId, report_type: ReportType) -> DispatchResult {
        let peer_ret = Self::peers(&peer_id);
        if peer_ret.is_none() {
            Err(Error::<T>::PeerIdNotExist)?
        }
        let mut peer_info = peer_ret.unwrap();
        if peer_info.status != NODE_STATUS_ONCHAIN {
            Err(Error::<T>::NodeStatusError)?
        }
        let era_index = T::StakingProvider::get_current_era_index();
        let report_info = ReportInfo {
            report_type: report_type,
            peer_id: peer_id.to_vec(),
        };
        let accounts_ret = <ReportsInEra<T>>::get(era_index, report_info.clone());
        if accounts_ret.is_some() {
            let mut accounts: BTreeSet::<T::AccountId> = accounts_ret.unwrap();
            // Repeat report
            if accounts.contains(&report_account) {
                Err(Error::<T>::ErrorNodeReport)?
            }
            accounts.insert(report_account);
            let accounts_count: u32 = u32::try_from(accounts.len()).unwrap();
            let punish_count;
            if report_type == ReportType::ReportTeeFaking {
                punish_count = Self::faking_report_number();
            } else {
                punish_count = Self::abnormal_report_number();
            }
            <ReportsInEra<T>>::insert(era_index, report_info, accounts);
            
            if accounts_count >= punish_count {
                if report_type == ReportType::ReportTeeFaking {
                    return Ok(());
                } else if report_type == ReportType::VerifyTeeFaking {
                    T::StakingProvider::report_offence(&peer_info.stash, T::StakingProvider::get_staking_active(&peer_info.stash));
                    if peer_info.status == NODE_STATUS_ONCHAIN {
                        <OnchainPeerNumber<T>>::mutate(|n| *n -= 1);
                    }
                    peer_info.status = NODE_STATUS_DISCARD;
                    <Peers<T>>::insert(peer_id, peer_info.clone());
                    // Set the status of the nodes based on the amount of stake
                    Self::update_peers_of_stash(&peer_info.stash, Zero::zero());
                } else {
                    let cur_num = frame_system::Pallet::<T>::block_number();
                    if peer_info.reward_number > cur_num {
                        peer_info.reward_number = peer_info.reward_number.saturating_add(Self::start_reward_block_number());
                    } else {
                        peer_info.reward_number = cur_num.saturating_add(Self::start_reward_block_number());
                    }
                    
                    <OnchainPeerNumber<T>>::mutate(|n| *n -= 1);
                    peer_info.status = NODE_STATUS_OFFCHAIN;
                    
                    <Peers<T>>::insert(peer_id, peer_info);
                }
            }
        } else {
            let mut accounts = BTreeSet::<T::AccountId>::new();
            accounts.insert(report_account);
            <ReportsInEra<T>>::insert(era_index, report_info, accounts);
        }
        Ok(())
    }
}

impl<T: Config> Pallet<T> 
{
    /// Set the status of the nodes based on the amount of stake
    pub fn update_peers_of_stash(stash: &T::AccountId, staking_active: BalanceOf<T>) {
        // Get peer ID of the stash
        Self::stash_peers(stash).map(|peer_id_set| {
            let min_amount = Self::min_staking_amount();
            let mut onchain_set: BTreeSet<PeerId> = BTreeSet::<PeerId>::new();
            let mut other_set: BTreeSet<PeerId> = BTreeSet::<PeerId>::new();
            // Group by status
            for peer_id in peer_id_set.iter() {
                Self::peers(&peer_id).map(|pre_info| {
                    if pre_info.status == NODE_STATUS_ONCHAIN {
                        onchain_set.insert(peer_id.to_vec());
                    } else if pre_info.status != NODE_STATUS_DISCARD && pre_info.status != NODE_STATUS_CLOSED {
                        other_set.insert(peer_id.to_vec());
                    }
                });
            }
            let should_onchain_num = (staking_active / min_amount).saturated_into::<usize>();
            if onchain_set.len() > should_onchain_num {
                let mut change_num = onchain_set.len().saturating_sub(should_onchain_num);
                let mut set_iter = onchain_set.iter();
                // Increase the number of online nodes
                while change_num > 0 {
                    let cur_peer = set_iter.next_back();
                    if cur_peer.is_some() {
                        let cur_peer_id = cur_peer.unwrap();
                        Self::peers(&cur_peer_id).map(|mut pre_info| {
                            <OnchainPeerNumber<T>>::mutate(|n| *n -= 1);
                            pre_info.status = NODE_STATUS_JOINING;
                            <Peers<T>>::insert(cur_peer_id, pre_info);
                            change_num -= 1;
                        });
                    } else {
                        break;
                    }
                }
            } else if onchain_set.len() < should_onchain_num && other_set.len() > 0 {
                let mut change_num = should_onchain_num.saturating_sub(onchain_set.len());
                if change_num > other_set.len() {
                    change_num = other_set.len();
                }
                let cur_block_num = frame_system::Pallet::<T>::block_number();
                let mut set_iter = other_set.iter();
                // Reduce the number of online nodes
                while change_num > 0 {
                    let cur_peer = set_iter.next();
                    if cur_peer.is_some() {
                        let cur_peer_id = cur_peer.unwrap();
                        Self::peers(&cur_peer_id).map(|mut pre_info| {
                            if pre_info.status == NODE_STATUS_JOINING {
                                pre_info.staked_number = cur_block_num;
                                pre_info.status = NODE_STATUS_STAKED;
                                change_num -= 1;

                                <Peers<T>>::insert(cur_peer_id, pre_info);
                            } else if (pre_info.status == NODE_STATUS_STAKED 
                                       && cur_block_num.saturating_sub(pre_info.staked_number) >= Self::tee_report_verify_number())
                                || pre_info.status != NODE_STATUS_STAKED {
                                <OnchainPeerNumber<T>>::mutate(|n| *n += 1);
                                pre_info.status = NODE_STATUS_ONCHAIN;
                                change_num -= 1;

                                <Peers<T>>::insert(cur_peer_id, pre_info);
                            }
                        });
                    } else {
                        break;
                    }
                }
            }
        });
    }
}

/// Interact with DC
pub trait DcProvider {
    /// The account identifier type.
    type AccountId;

    type Balance;

    /// Get the rewardable peers's space.
    fn rewardable_peers_space() -> Option<(SpaceSize, BTreeMap<Self::AccountId, SpaceSize>)>;
    /// Get the rewardable app's login times.
    fn rewardable_app_login_times() -> Option<(LoginTimes, BTreeMap<Self::AccountId, LoginTimes>)>;
    /// Is it possible to unbind the incoming amount.
    fn can_be_unbound_amount(stash: &Self::AccountId, value: Self::Balance) -> Self::Balance;
    /// Update the staked amount of peer.
    fn update_active(stash: &Self::AccountId, active: Self::Balance);
    /// Pay out for storage for the current era. 
    fn era_storage_payout(era_duration: u64) -> Self::Balance;
    /// Pay out for app for the current era. 
    fn era_app_payout(era_duration: u64) -> Self::Balance;

    /// The total cost of storage purchased by users. 
    fn purchase_storage_cost() -> Self::Balance;
}

impl<T: Config> DcProvider for Pallet<T> {
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    /// Get the rewardable peers's space.
    fn rewardable_peers_space() -> Option<(SpaceSize, BTreeMap<Self::AccountId, SpaceSize>)> {
        let mut total = 0;
        let mut accounts = BTreeMap::<Self::AccountId, SpaceSize>::new();
        let cur_block_num = frame_system::Pallet::<T>::block_number();
        // Get the space info
        <Peers<T>>::iter_values()
            .for_each(|storage_node| {
                if storage_node.status == NODE_STATUS_ONCHAIN
                   && storage_node.reward_number < frame_system::Pallet::<T>::block_number()
                   && cur_block_num.saturating_sub(storage_node.report_number) < Self::interval_blocks_work_report() {
                    let mut cur_space: SpaceSize = storage_node.total_space/ONE_G_BYTE;
                    // When the sgx version is 2, increase the income by 20%.
                    if storage_node.sgx_version_number ==  2 {
                        cur_space = cur_space * 12 / 10;
                    }
                    total = total.saturating_add(cur_space);
                    let tmp_stash = storage_node.stash.clone();
                    if accounts.contains_key(&tmp_stash) {
                        accounts.insert(storage_node.stash, cur_space.saturating_add(*(accounts.get(&tmp_stash).unwrap())));
                    } else {
                        accounts.insert(storage_node.stash, cur_space);
                    }
                }
            });
        Some((total, accounts))
    }

    /// Get the rewardable app's login times.
    fn rewardable_app_login_times() -> Option<(LoginTimes, BTreeMap<Self::AccountId, LoginTimes>)> {
        let mut total: LoginTimes = 0;
        let mut accounts = BTreeMap::<Self::AccountId, LoginTimes>::new();
        // Get the login info
        let login_count: BTreeMap<AppID, AppLoginInfo<T::AccountId>> = match <AppsAccountLoginTimes::<T>>::get() {
            Some(p) => p,
            None => BTreeMap::<AppID, AppLoginInfo<T::AccountId>>::new(),
        };

        for (_, login_info) in login_count.iter() {
            total = total.saturating_add(login_info.login_times);
            accounts.insert(login_info.rewarded_stash.clone(), login_info.login_times);
        }
        Some((total, accounts))
    }

    /// With reference to the input value, the number that can be unbound.
    fn can_be_unbound_amount(stash: &Self::AccountId, value: Self::Balance) -> Self::Balance {
        Self::stash_peers(stash).map_or(value, |peer_id_set| {
            let min_amount = Self::min_staking_amount();
            let staking_active = T::StakingProvider::get_staking_active(stash);
            let mut should_bond_amount: Self::Balance = Zero::zero();
            for peer_id in peer_id_set.iter() {
                Self::peers(&peer_id).map(|pre_info| {
                    if pre_info.status != NODE_STATUS_DISCARD {
                        should_bond_amount = should_bond_amount.saturating_add(min_amount);
                    }
                });
            }
            if staking_active > should_bond_amount {
                if value <= staking_active.saturating_sub(should_bond_amount) {
                    value
                } else {
                    staking_active.saturating_sub(should_bond_amount)
                }
            } else {
                Zero::zero()
            }
        })
    }

    /// Update the staked amount of peer
    fn update_active(stash: &Self::AccountId, staking_active: Self::Balance) {
        // Set the status of the nodes based on the amount of stake
        Self::update_peers_of_stash(stash, staking_active);
    }

    /// Pay out for storage for the current era. 
    fn era_storage_payout(era_duration: u64) -> Self::Balance {
        let total = Self::storage_reward_total();
        if total > Zero::zero() {
            // Milliseconds per year for the Julian year (365.25 days).
            const MILLISECONDS_PER_YEAR: u64 = 1000 * 3600 * 24 * 36525 / 100;

            let portion = Perbill::from_rational(era_duration as u64, MILLISECONDS_PER_YEAR);
            let pay_balance = portion * total;
            <StorageRewardTotal<T>>::put(total.saturating_sub(pay_balance));
            pay_balance
        } else {
            Zero::zero()
        }
    }

    /// Pay out for app for the current era. 
    fn era_app_payout(era_duration: u64) -> Self::Balance {
        let total = Self::app_reward_total();
        if total > Zero::zero() {
            // Milliseconds per year for the Julian year (365.25 days).
            const MILLISECONDS_PER_YEAR: u64 = 1000 * 3600 * 24 * 36525 / 100;

            let portion = Perbill::from_rational(era_duration as u64, MILLISECONDS_PER_YEAR);
            let pay_balance = portion * total;
            <AppRewardTotal<T>>::put(total.saturating_sub( pay_balance));
            pay_balance
        } else {
            Zero::zero()
        }
    }

    /// The total cost of storage purchased by users. 
    fn purchase_storage_cost() -> Self::Balance {
        Self::storage_reward_total().saturating_add(Self::app_reward_total())
    }
}

/// Get staking infomation
pub trait StakingProvider {
    /// The account identifier type.
    type AccountId;

    type Balance;

    /// Get staking active of stash.
    fn get_staking_active(stash: &Self::AccountId) -> Self::Balance;

    /// Get current erea index
    fn get_current_era_index() -> EraIndex;

    /// Check the controller account mapped by the "stash" account
    fn is_bonded_controller(stash: &Self::AccountId, controller: &Self::AccountId) -> bool;

    fn report_offence(stash: &Self::AccountId, slash: Self::Balance,);
}
