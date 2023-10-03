#![cfg_attr(not(feature = "std"), no_std)]

// Note that these interfaces should not limit or heavily influence the design of your pallet.
//
// These interfaces do NOT make sense to expose as the extrinsics of your pallet.
// Instead, these will simply be used to execute unit tests to verify the basic logic of your
// pallet is working. You should design your own extrinsic functions which make sense for
// exposing to end users of your pallet.
//
// It should be totally possible to create more complex or unique pallets, while adhering to
// the interfaces below.
//
// If any of these interfaces are not compatible with your design or vision, talk to an
// instructor and we can figure out the best way forward.

use core::{cmp::Ord, fmt::Debug};
use frame_support::{
	dispatch::Vec,
	pallet_prelude::{
		DispatchError, DispatchResult, MaxEncodedLen, MaybeSerializeDeserialize, Member, Parameter,
	},
	traits::tokens::Balance as BalanceTrait,
};

/// A minimal interface to test the functionality of the Voting pallet.
pub trait VotingInterface {
	/// The type which can be used to identify accounts.
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ Ord
		+ MaxEncodedLen
		+ Clone;
	/// The type representing the balance users can vote with.
	type VotingBalance: BalanceTrait;
	/// The type representing a unique ID for a proposal.
	type ProposalId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;

	/// This function should register a user in the identity system, allowing that user to vote, and
	/// give that user some voting balance equal to `amount`.
	fn add_voter(who: Self::AccountId, amount: Self::VotingBalance) -> DispatchResult;

	/// Create a proposal with the following metadata.
	///
	/// If `Ok`, return the `ProposalId`.
	fn create_proposal(metadata: Vec<u8>) -> Result<Self::ProposalId, DispatchError>;

	/// Make a voter vote on a proposal with a given vote weight.
	///
	/// If the voter supports the proposal, they will vote `aye = true`, otherwise they should vote
	/// `aye = false`.
	///
	/// The `vote_weight` should represent the value after we take the sqrt of their voting balance,
	/// thus you can simply square the `amount` rather than taking the sqrt of some value.
	///
	/// For example: If a user votes with `vote_weight = 10`, then we should check they have at
	/// least `100` total voting balance.
	fn vote(
		proposal: Self::ProposalId,
		voter: Self::AccountId,
		aye: bool,
		vote_weight: Self::VotingBalance,
	) -> DispatchResult;

	/// Do whatever is needed to resolve the vote, and determine the outcome.
	///
	/// If `Ok`, return the result of the vote with a bool, `true` being the vote passed, and
	/// `false` being the vote failed.
	fn close_vote(proposal: Self::ProposalId) -> Result<bool, DispatchError>;
}
