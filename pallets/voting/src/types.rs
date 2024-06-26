#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::*, BoundedVec};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::{prelude::fmt::Debug, TypeInfo};

pub type ProposalId = u32;
/// The current vote ratio for a open proposal.
/// The first element represent 'aye' votes and the second the total number of votes.
pub type VoteRatio = (u128, u128);

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
	/// The data related to this proposal (e.g an CID Hash pointing to a Json file; a static or
	/// dynamic link; plain text)
	pub offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>,
	/// The vote ratio for this proposal.
	/// The first item represents the number of 'aye' votes.
	/// The second item represents the total number of votes.
	/// A poposal gets majority when the are more 'ayes' votes that the half number of total votes
	/// when closing the proposal.
	pub ratio: VoteRatio,
	/// The proposal kind: 'Public' or 'Private'.
	/// A public proposal is open for all registered voters to vote. The proposal can be closed by
	/// the creator once the end_block is reached. A private proposal is similar to a quorum vote.
	/// The proposal is close as soon as majority is get among the number of registered voters
	/// allowed to vote in the account list. The proposal is closed and reject if majority is not
	/// reached when passing the ending block.
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
	// TODO: document all helpers
	pub fn new(
		offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>,
		kind: ProposalKind,
		creator: AccountId,
		account_list: Option<BoundedVec<AccountId, AccountSizeLimit>>,
		start_block: BlockNumberFor<T>,
		end_block: BlockNumberFor<T>,
	) -> Self {
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

	// pub fn has_majority(&self) -> bool {
	// 	if self.kind == ProposalKind::Private {
	// 		let maybe_account_list = &self.account_list;
	// 		if let Some(account_list) = maybe_account_list {
	// 			let account_list_len = account_list.len();
	// 			let (aye, total) = self.ratio;
	// 			return account_list_len > 1 && aye > total / 2;
	// 		}
	// 	}

	// 	false
	// }

	pub fn add_ratio(&mut self, aye: bool, prev_power: u128, new_power: u128) {
		let prev_quadratic_amount = prev_power.checked_mul(prev_power).unwrap_or(u128::MAX);
		let new_quadratic_amount = new_power.checked_mul(new_power).unwrap_or(u128::MAX);
		let amount_diff = new_quadratic_amount.saturating_sub(prev_quadratic_amount);

		let new_ratio = if aye {
			(self.ratio.0.saturating_add(amount_diff), self.ratio.1.saturating_add(amount_diff))
		} else {
			(self.ratio.0, self.ratio.1.saturating_add(amount_diff))
		};
		*self = Self { ratio: new_ratio, ..self.clone() }
	}

	pub fn remove_ratio(&mut self, aye: bool, prev_power: u128, new_power: u128) {
		let prev_quadratic_amount = prev_power.checked_mul(prev_power).unwrap_or(u128::MAX);
		let new_quadratic_amount = new_power.checked_mul(new_power).unwrap_or(u128::MAX);
		let amount_diff = prev_quadratic_amount.saturating_sub(new_quadratic_amount);

		let new_ratio = if aye {
			(self.ratio.0.saturating_sub(amount_diff), self.ratio.1.saturating_sub(amount_diff))
		} else {
			(self.ratio.0, self.ratio.1.saturating_sub(amount_diff))
		};
		*self = Self { ratio: new_ratio, ..self.clone() }
	}
}

#[derive(
	Encode, Decode, Eq, CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo, MaxEncodedLen,
)]
pub struct VoteInfo {
	/// The proposal ID
	pub proposal_id: ProposalId,
	/// The vote:
	/// - 'aye' -> true,
	/// - 'nay' -> false,
	pub aye: bool,
	/// The power for this vote
	pub power: u128,
}
