#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::traits::Zero;
use frame_support::sp_runtime::{SaturatedConversion, Saturating};
use frame_support::{dispatch::Vec, pallet_prelude::*, traits::fungible};
use frame_system::pallet_prelude::BlockNumberFor;

pub use pallet::*;
pub use types::{ProposalData, ProposalId, ProposalKind, VoteInfo, VoteRatio};

#[cfg(test)]
mod mock;
mod types;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::fungible::{InspectFreeze, MutateFreeze};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Balance;
	pub type FreezeIdOf<T> = <<T as Config>::NativeBalance as fungible::freeze::Inspect<
		<T as frame_system::Config>::AccountId,
	>>::Id;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// Freeze identifier used by the pallet
		#[pallet::constant]
		type FreezeIdForPallet: Get<FreezeIdOf<Self>>;

		/// Maximum offchain data length.
		#[pallet::constant]
		type ProposalOffchainDataLimit: Get<u32>;

		/// Maximum number of accounts that can be stored inside the account list.
		#[pallet::constant]
		type AccountSizeLimit: Get<u32>;

		/// Maximum duration for a proposal.
		#[pallet::constant]
		type ProposalMaximumDuration: Get<u32>;

		/// Minimum duration for a proposal.
		#[pallet::constant]
		type ProposalMinimumDuration: Get<u32>;

		/// Maximum delay for a proposal to start.
		#[pallet::constant]
		type ProposalDelayLimit: Get<u32>;
	}

	/// All well-known voters registered to participate in proposal voting
	#[pallet::storage]
	#[pallet::getter(fn registered_voters)]
	pub type RegisteredVoters<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	/// The ID that will be used by the next proposal created
	#[pallet::storage]
	#[pallet::getter(fn next_proposal_id)]
	pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

	/// All proposals staged or in progress
	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalId,
		ProposalData<T, T::AccountId, T::AccountSizeLimit, T::ProposalOffchainDataLimit>,
		OptionQuery,
	>;

	/// All votes for proposals in progress.
	/// The key is the proposal ID and the voter ID, to ensure it's unique.
	#[pallet::storage]
	#[pallet::getter(fn get_votes)]
	pub type Votes<T: Config> = StorageDoubleMap<
		_,
		Blake2_256,
		T::AccountId,
		Blake2_256,
		ProposalId,
		VoteInfo,
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewVoterRegistered {
			who: T::AccountId,
		},
		VoterUnregistered {
			who: T::AccountId,
		},
		NewProposal {
			proposal_id: ProposalId,
			offchain_data: BoundedVec<u8, T::ProposalOffchainDataLimit>,
			creator: T::AccountId,
			kind: ProposalKind,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			start_block: BlockNumberFor<T>,
			end_block: BlockNumberFor<T>,
		},
		ProposalCancelled {
			proposal_id: ProposalId,
		},
		ProposalClosed {
			proposal_id: ProposalId,
			ratio: (u128, u128),
		},
		AccountListSet {
			proposal_id: ProposalId,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
		},
		/// A new vote was added to an in progress proposal
		VoteAdded {
			proposal_id: ProposalId,
			voter: T::AccountId,
			aye: bool,
			power: u128,
		},
		/// A vote was removed from an in progress proposal
		VoteDropped {
			proposal_id: ProposalId,
			voter: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Origin has no permission to operate
		OriginNoPermission,
		/// A user is trying to vote, but is not registered in the `RegisteredVoters` storage.
		VoterNotRegistered,
		/// A proposal does not exist in storage
		ProposalDoesNotExist,
		/// A proposal has already started
		ProposalHasAlreadyStarted,
		/// A proposal has already ended
		ProposalHasAlreadyEnded,
		/// A proposal has not started yet
		ProposalHasNotStartedYet,
		/// A proposal has not ended yet
		ProposalHasNotEndedYet,
		/// A proposal cannot start in the past
		ProposalCannotStartInThePast,
		/// A proposal cannot end before starting
		ProposalCannotFinishBeforeStarting,
		/// The proposal duration is too long
		ProposalDurationIsTooLong,
		/// The proposal duration is too short
		ProposalDurationIsTooShort,
		/// The delay for a proposal to start is too far away
		ProposalStartIsTooFarAway,
		/// The voter has insufficient free funds to vote with power
		InsufficientBalance,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_voter(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			RegisteredVoters::<T>::insert(&who, ());
			Self::deposit_event(Event::<T>::NewVoterRegistered { who });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn unregister_voter(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			let caller =
				ensure_signed_or_root(origin).map_err(|_| Error::<T>::OriginNoPermission)?;
			ensure!((caller.is_none() || caller.unwrap() == who), Error::<T>::OriginNoPermission);
			RegisteredVoters::<T>::remove(&who);
			Self::deposit_event(Event::<T>::VoterUnregistered { who });

			// TODO: clean up votes & adjust ratio

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_proposal(
			origin: OriginFor<T>,
			offchain_data: BoundedVec<u8, T::ProposalOffchainDataLimit>,
			kind: ProposalKind,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
			start_block: BlockNumberFor<T>,
			end_block: BlockNumberFor<T>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(
				RegisteredVoters::<T>::get(caller.clone()).is_some(),
				Error::<T>::VoterNotRegistered
			);

			let current_block = Pallet::<T>::get_current_block_number();
			ensure!(current_block <= start_block, Error::<T>::ProposalCannotStartInThePast);
			ensure!(start_block < end_block, Error::<T>::ProposalCannotFinishBeforeStarting);

			let duration = end_block.saturating_sub(start_block);
			let buffer = start_block.saturating_sub(current_block);
			ensure!(
				buffer <= T::ProposalDelayLimit::get().into(),
				Error::<T>::ProposalStartIsTooFarAway
			);
			ensure!(
				duration >= T::ProposalMinimumDuration::get().into(),
				Error::<T>::ProposalDurationIsTooShort
			);
			ensure!(
				duration <= T::ProposalMaximumDuration::get().into(),
				Error::<T>::ProposalDurationIsTooLong
			);

			// TODO: ensure account_list not empty for private proposals?

			let proposal_id = Pallet::<T>::get_next_proposal_id();
			let proposal = ProposalData::new(
				offchain_data.clone(),
				kind.clone(),
				caller.clone(),
				account_list.clone(),
				start_block,
				end_block,
			);

			Proposals::<T>::insert(proposal_id, proposal);

			let event = Event::NewProposal {
				proposal_id,
				offchain_data,
				creator: caller,
				kind,
				account_list,
				start_block,
				end_block,
			};
			Self::deposit_event(event);

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn cancel_proposal(origin: OriginFor<T>, proposal_id: ProposalId) -> DispatchResult {
			let caller = ensure_signed_or_root(origin)?;

			let current_block = Pallet::<T>::get_current_block_number();
			let proposal =
				Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			ensure!(
				(caller.is_none() || proposal.is_creator(&caller.unwrap())),
				Error::<T>::OriginNoPermission
			);
			ensure!(!proposal.has_started(&current_block), Error::<T>::ProposalHasAlreadyStarted);

			Proposals::<T>::remove(proposal_id);
			Self::deposit_event(Event::<T>::ProposalCancelled { proposal_id });
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn close_proposal(
			origin: OriginFor<T>,
			proposal_id: ProposalId,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			let current_block = Pallet::<T>::get_current_block_number();
			let proposal =
				Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			ensure!(proposal.has_ended(&current_block), Error::<T>::ProposalHasNotEndedYet);

			Proposals::<T>::remove(proposal_id);
			Self::deposit_event(Event::<T>::ProposalClosed { proposal_id, ratio: proposal.ratio });
			Ok(Pays::No.into())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_account_list(
			origin: OriginFor<T>,
			proposal_id: ProposalId,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
		) -> DispatchResult {
			let caller = ensure_signed_or_root(origin)?;

			let current_block = Pallet::<T>::get_current_block_number();
			let proposal =
				Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			ensure!(
				(caller.is_none() || proposal.is_creator(&caller.unwrap())),
				Error::<T>::OriginNoPermission
			);
			ensure!(!proposal.has_started(&current_block), Error::<T>::ProposalHasAlreadyStarted);

			Proposals::<T>::insert(
				proposal_id,
				ProposalData { account_list: account_list.clone(), ..proposal },
			);
			Self::deposit_event(Event::<T>::AccountListSet { proposal_id, account_list });
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn vote(
			origin: OriginFor<T>,
			proposal_id: ProposalId,
			aye: bool,
			power: u128,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(
				RegisteredVoters::<T>::get(caller.clone()).is_some(),
				Error::<T>::VoterNotRegistered
			);

			let current_block = Pallet::<T>::get_current_block_number();

			Proposals::<T>::try_mutate(proposal_id, |maybe_proposal| -> DispatchResult {
				let proposal = maybe_proposal.as_mut().ok_or(Error::<T>::ProposalDoesNotExist)?;

				ensure!(proposal.has_started(&current_block), Error::<T>::ProposalHasNotStartedYet);
				ensure!(!proposal.has_ended(&current_block), Error::<T>::ProposalHasNotEndedYet);

				let maybe_account_list = proposal.clone().account_list;
				if let Some(account_list) = maybe_account_list {
					let allowed_voter = match proposal.kind {
						ProposalKind::Public => !account_list.contains(&caller),
						ProposalKind::Private => account_list.contains(&caller),
					};
					ensure!(allowed_voter, Error::<T>::OriginNoPermission)
				}

				if power.is_zero() {
					T::NativeBalance::thaw(&T::FreezeIdForPallet::get(), &caller)?;
				} else {
					let new_amount = Pallet::<T>::calculate_quadratic_amount(power);
					ensure!(
						T::NativeBalance::balance_freezable(&caller).ge(&new_amount),
						Error::<T>::InsufficientBalance
					);
					T::NativeBalance::set_freeze(
						&T::FreezeIdForPallet::get(),
						&caller,
						new_amount,
					)?;
				}

				let maybe_vote = Votes::<T>::get(caller.clone(), proposal_id);
				if let Some(vote) = maybe_vote {
					// TODO: ensure power not the same
					let prev_power = vote.power;
					if prev_power.lt(&power) {
						let power_diff = power.saturating_sub(prev_power);
						proposal.add_ratio(aye, power_diff);
					} else {
						let power_diff = prev_power.saturating_sub(power);
						proposal.remove_ratio(aye, power_diff);
					}
				} else {
					proposal.add_ratio(aye, power);
				}

				if power.is_zero() {
					Votes::<T>::remove(caller.clone(), proposal_id);
					Self::deposit_event(Event::VoteDropped { proposal_id, voter: caller });
				} else {
					Votes::<T>::insert(caller.clone(), proposal_id, VoteInfo { aye, power });
					Self::deposit_event(Event::VoteAdded {
						proposal_id,
						voter: caller,
						aye,
						power,
					});
				}
				Ok(().into())
			})?;

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn get_next_proposal_id() -> ProposalId {
		let proposal_id = NextProposalId::<T>::get();
		let next_id = proposal_id.checked_add(1).expect("Overflow u32 check; qed.");
		NextProposalId::<T>::put(next_id);
		proposal_id
	}

	fn get_current_block_number() -> BlockNumberFor<T> {
		frame_system::Pallet::<T>::block_number()
	}

	fn calculate_quadratic_amount(power: u128) -> BalanceOf<T> {
		power.checked_mul(power).unwrap_or(u128::MAX).saturated_into()
	}
}

// Look at `../interface/` to better understand this API.
impl<T: Config> pba_interface::VotingInterface for Pallet<T> {
	type AccountId = T::AccountId;
	type VotingBalance = <T::NativeBalance as fungible::Inspect<Self::AccountId>>::Balance;
	// You can change this if you need.
	type ProposalId = u32;

	fn add_voter(_who: Self::AccountId, _amount: Self::VotingBalance) -> DispatchResult {
		unimplemented!()
	}

	fn create_proposal(_metadata: Vec<u8>) -> Result<Self::ProposalId, DispatchError> {
		unimplemented!()
	}

	fn vote(
		_proposal: Self::ProposalId,
		_voter: Self::AccountId,
		_aye: bool,
		_vote_weight: Self::VotingBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn close_vote(_proposal: Self::ProposalId) -> Result<bool, DispatchError> {
		unimplemented!()
	}
}
