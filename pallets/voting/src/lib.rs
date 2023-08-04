#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::sp_runtime::Saturating;
use frame_support::{dispatch::Vec, pallet_prelude::*, traits::fungible};
use frame_system::pallet_prelude::BlockNumberFor;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
pub use types::{ProposalData, ProposalId, ProposalKind, VoteRatio};

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
	use frame_support::traits::fungible;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

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

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn registered_voters)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type RegisteredVoters<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn next_proposal_id)]
	pub type NextProposalId<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Proposals<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		ProposalId,
		ProposalData<T, T::AccountId, T::AccountSizeLimit, T::ProposalOffchainDataLimit>,
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
			ratio: (u32, u32),
		},
		AccountListSet {
			proposal_id: ProposalId,
			account_list: Option<BoundedVec<T::AccountId, T::AccountSizeLimit>>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Origin has no permission to operate on the registered voter
		OriginNoPermission,
		/// A user is trying to vote, but is not registered in the `RegisteredVoters` storage.
		VoterNotRegistered,
		/// A proposal does not exist in storage
		ProposalDoesNotExist,
		/// A proposal has already started
		ProposalHasAlreadyStarted,
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
		pub fn close_proposal(origin: OriginFor<T>, proposal_id: ProposalId) -> DispatchResult {
			ensure_signed(origin)?;

			let current_block = Pallet::<T>::get_current_block_number();
			let proposal =
				Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalDoesNotExist)?;

			ensure!(proposal.has_ended(&current_block), Error::<T>::ProposalHasNotEndedYet);

			Proposals::<T>::remove(proposal_id);
			Self::deposit_event(Event::<T>::ProposalClosed { proposal_id, ratio: proposal.ratio });
			Ok(())
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
