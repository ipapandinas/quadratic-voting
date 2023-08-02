#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{dispatch::Vec, pallet_prelude::*, traits::fungible};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::fungible};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	pub type ProposalId = u32;

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
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn registered_voters)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type RegisteredVoters<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, (), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewVoterRegistered { who: T::AccountId },
		VoterUnregistered { who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Origin has no permission to operate on the registered voter
		OriginNoPermission,
		/// A user is trying to vote, but is not registered in the `RegisteredVoters` storage.
		VoterNotRegistered,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
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

		// An example dispatchable that may throw a custom error.
		// #[pallet::call_index(1)]
		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;

		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
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
