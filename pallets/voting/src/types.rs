#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::prelude::fmt::Debug;
use scale_info::TypeInfo;
// use sp_std::fmt::Debug;

// pub type BlockNumber = u32;
pub type ProposalId = u32;
pub type VoteRatio = (u32, u32);

#[derive(
	PartialEq, Eq, Copy, Clone, RuntimeDebug, Encode, Decode, Default, TypeInfo, MaxEncodedLen,
)]
pub enum ProposalKind {
	#[default]
	Public,
	Private = 1,
}

#[derive(
	Encode, Decode, Eq, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
#[scale_info(skip_type_params(T, AccountSizeLimit, ProposalOffchainDataLimit))]
pub struct ProposalData<T, AccountId, AccountSizeLimit, ProposalOffchainDataLimit>
where
	T: frame_system::Config,
	AccountId: Clone + PartialEq + Debug,
	AccountSizeLimit: Get<u32>,
	ProposalOffchainDataLimit: Get<u32>,
{
	/// The data related to this proposal (e.g an CID Hash pointing to a Json file; a static or dynamic link; plain text)
	pub offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>,
	/// The vote ratio for this proposal.
	/// The first item represents the number of 'aye' votes.
	/// The second item represents the total number of votes.
	/// A poposal gets majority when the are more 'ayes' votes that the half number of total votes when closing the proposal.  
	pub ratio: VoteRatio,
	/// The proposal kind: 'Public' or 'Private'.
	/// A public proposal is open for all registered voters to vote. The proposal can be closed by the creator once the end_block is reached.
	/// A private proposal is similar to a quorum vote. The proposal is close as soon as majority is get among the number of registered voters allowed to vote in the account list. The proposal is closed and reject if majority is not reached when passing the ending block.
	pub kind: ProposalKind,
	/// The proposal creator.
	pub creator: AccountId,
	/// The accounts interacting with this list.
	/// For a 'public' proposal: banned accounts to vote.
	/// For a 'private' proposal: allowed accounts to vote.
	pub account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
	/// `BlockNumber` at which the proposal will accept votes.
	pub start_block: BlockNumberFor<T>,
	/// `BlockNumber` at which the proposal will no longer accept votes.
	pub end_block: BlockNumberFor<T>,
}

impl<T, AccountId, AccountSizeLimit, ProposalOffchainDataLimit>
	ProposalData<T, AccountId, AccountSizeLimit, ProposalOffchainDataLimit>
where
	T: frame_system::Config,
	AccountId: Clone + PartialEq + Debug,
	AccountSizeLimit: Get<u32>,
	ProposalOffchainDataLimit: Get<u32>,
{
	pub fn new(
		offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>,
		kind: ProposalKind,
		creator: AccountId,
		account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
		start_block: BlockNumberFor<T>,
		end_block: BlockNumberFor<T>,
	) -> ProposalData<T, AccountId, AccountSizeLimit, ProposalOffchainDataLimit> {
		Self {
			offchain_data,
			ratio: VoteRatio::default(),
			kind,
			creator,
			account_list,
			start_block,
			end_block,
		}
	}

	pub fn is_creator(&self, who: &AccountId) -> bool {
		self.creator == *who
	}

	pub fn has_started(&self, block: &BlockNumberFor<T>) -> bool {
		self.start_block.le(block)
	}

	pub fn has_ended(&self, block: &BlockNumberFor<T>) -> bool {
		self.end_block.le(block)
	}
}
