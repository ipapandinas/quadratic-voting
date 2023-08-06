use crate as pallet_voting;
use frame_support::{
	parameter_types,
	traits::{ConstU128, ConstU16, ConstU32, ConstU64},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

type Block = frame_system::mocking::MockBlock<Test>;
type Balance = u128;
pub type BlockNumber = u32;

pub const PROPOSAL_ACCOUNT_SIZE_LIMIT: u32 = 1000;
pub const PROPOSAL_OFFCHAIN_DATA_LIMIT: u32 = 150;
pub const PROPOSAL_MAXIMUM_DURATION: BlockNumber = 1000;
pub const PROPOSAL_MINIMUM_DURATION: BlockNumber = 100;
pub const PROPOSAL_DELAY_LIMIT: BlockNumber = 100;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		Voting: pallet_voting,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type RuntimeHoldReason = ();
	type FreezeIdentifier = ();
	type MaxHolds = ConstU32<10>;
	type MaxFreezes = ConstU32<10>;
}

parameter_types! {
	pub const AccountSizeLimit: u32 = PROPOSAL_ACCOUNT_SIZE_LIMIT;
	pub const ProposalOffchainDataLimit: u32 = PROPOSAL_OFFCHAIN_DATA_LIMIT;
	pub const ProposalMaximumDuration: u32 = PROPOSAL_MAXIMUM_DURATION;
	pub const ProposalMinimumDuration: u32 = PROPOSAL_MINIMUM_DURATION;
	pub const ProposalDelayLimit: u32 = PROPOSAL_DELAY_LIMIT;
}

impl pallet_voting::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type NativeBalance = Balances;
	type AccountSizeLimit = AccountSizeLimit;
	type ProposalOffchainDataLimit = ProposalOffchainDataLimit;
	type ProposalMaximumDuration = ProposalMaximumDuration;
	type ProposalMinimumDuration = ProposalMinimumDuration;
	type ProposalDelayLimit = ProposalDelayLimit;
	type FreezeIdForPallet = ();
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub struct ExtBuilder {
	balances: Vec<(u64, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { balances: Vec::new() }
	}
}

impl ExtBuilder {
	pub fn new(balances: Vec<(u64, Balance)>) -> Self {
		Self { balances }
	}

	pub fn new_build(balances: Vec<(u64, Balance)>) -> sp_io::TestExternalities {
		Self::new(balances).build()
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = <frame_system::GenesisConfig<Test> as BuildStorage>::build_storage(
			&frame_system::GenesisConfig::default(),
		)
		.unwrap();

		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
