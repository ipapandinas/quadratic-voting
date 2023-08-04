use crate::{
	mock::{self, *},
	pallet::{self as pallet_voting},
	Error, Event, ProposalKind,
};
use frame_support::{assert_noop, assert_ok, BoundedVec};
use frame_system::RawOrigin;
use sp_runtime::DispatchResult;

const ALICE: u64 = 0;
const BOB: u64 = 1;

mod register_voter {
	use super::*;
	use sp_runtime::DispatchError;

	#[test]
	fn works_only_if_root() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			assert_noop!(
				Voting::register_voter(RuntimeOrigin::signed(0), 0),
				DispatchError::BadOrigin
			);
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 1));
			assert_eq!(Voting::registered_voters(1), Some(()));
			System::assert_last_event(Event::NewVoterRegistered { who: 1 }.into());
		});
	}
}

mod unregister_voter {
	use super::*;

	#[test]
	fn works_with_root() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 0));
			assert_eq!(Voting::registered_voters(0), Some(()));
			assert_ok!(Voting::unregister_voter(RuntimeOrigin::root(), 0));
			assert_eq!(Voting::registered_voters(0), None);
			System::assert_last_event(Event::VoterUnregistered { who: 0 }.into());
		})
	}

	#[test]
	fn works_with_correct_signed_origin() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), 0));
			assert_eq!(Voting::registered_voters(0), Some(()));
			assert_noop!(
				Voting::unregister_voter(RuntimeOrigin::signed(1), 0),
				Error::<Test>::OriginNoPermission
			);
			assert_ok!(Voting::unregister_voter(RuntimeOrigin::signed(0), 0));
			assert_eq!(Voting::registered_voters(0), None);
			System::assert_last_event(Event::VoterUnregistered { who: 0 }.into());
		})
	}
}

fn setup() {
	assert_ok!(Voting::register_voter(RuntimeOrigin::root(), ALICE));
}

mod create_proposal {
	use super::*;
	use crate::ProposalData;

	#[test]
	fn new_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 20;
			let end_block = 120;
			setup();

			let proposal_data: ProposalData<
				Test,
				u64,
				AccountSizeLimit,
				ProposalOffchainDataLimit,
			> = ProposalData::new(
				BoundedVec::default(),
				ProposalKind::default(),
				ALICE,
				Some(BoundedVec::default()),
				start_block,
				end_block,
			);

			// Execution
			assert_ok!(Voting::create_proposal(
				RuntimeOrigin::signed(proposal_data.creator),
				proposal_data.clone().offchain_data,
				proposal_data.kind,
				proposal_data.clone().account_list,
				proposal_data.start_block,
				proposal_data.end_block
			));

			// Storage
			let proposal_id = Voting::get_next_proposal_id() - 1;
			let proposal = Voting::proposals(proposal_id);

			assert_eq!(proposal, Some(proposal_data.clone()));

			// Event
			System::assert_last_event(
				Event::NewProposal {
					proposal_id,
					offchain_data: proposal_data.offchain_data,
					creator: ALICE,
					kind: proposal_data.kind,
					account_list: proposal_data.account_list,
					start_block,
					end_block,
				}
				.into(),
			);
		})
	}

	#[test]
	fn cannot_start_proposal_in_the_past() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			setup();

			// Execution
			assert_noop!(
				ProposalBuilder::new().start(0).execute(),
				Error::<Test>::ProposalCannotStartInThePast
			);
		})
	}

	#[test]
	fn cannot_finish_proposal_before_starting() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			setup();

			// Execution
			assert_noop!(
				ProposalBuilder::new().end(0).execute(),
				Error::<Test>::ProposalCannotFinishBeforeStarting
			);
		})
	}

	#[test]
	fn too_much_delay_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			setup();

			let max_delay = <Test as pallet_voting::Config>::ProposalDelayLimit::get();
			// Execution
			assert_noop!(
				ProposalBuilder::new()
					.start(u32::try_from(System::block_number()).unwrap_or(0) + max_delay + 1)
					.execute(),
				Error::<Test>::ProposalStartIsTooFarAway
			);
		})
	}

	#[test]
	fn too_long_duration_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			setup();

			let max_duration = <Test as pallet_voting::Config>::ProposalMaximumDuration::get();
			// Execution
			assert_noop!(
				ProposalBuilder::new()
					.end(u32::try_from(System::block_number()).unwrap_or(0) + max_duration + 1)
					.execute(),
				Error::<Test>::ProposalDurationIsTooLong
			);
		})
	}

	#[test]
	fn too_short_duration_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			setup();

			let min_duration = <Test as pallet_voting::Config>::ProposalMinimumDuration::get();
			// Execution
			assert_noop!(
				ProposalBuilder::new()
					.end(u32::try_from(System::block_number()).unwrap_or(0) + min_duration - 1)
					.execute(),
				Error::<Test>::ProposalDurationIsTooShort
			);
		})
	}
}

mod cancel_proposal {
	use super::*;

	#[test]
	fn cancel_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 10;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_ok!(Voting::cancel_proposal(RuntimeOrigin::signed(ALICE), proposal_id));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(proposal, None);

			// Event
			System::assert_last_event(Event::ProposalCancelled { proposal_id }.into());
		})
	}

	#[test]
	fn cannot_cancel_proposal_after_start() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			System::set_block_number(100);

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::cancel_proposal(RuntimeOrigin::signed(ALICE), proposal_id),
				Error::<Test>::ProposalHasAlreadyStarted
			);
		})
	}

	#[test]
	fn cannot_cancel_proposal_not_existing() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::cancel_proposal(RuntimeOrigin::signed(ALICE), proposal_id + 1),
				Error::<Test>::ProposalDoesNotExist
			);
		})
	}

	#[test]
	fn works_only_if_root_or_creator() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 10;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::cancel_proposal(RuntimeOrigin::signed(BOB), proposal_id),
				Error::<Test>::OriginNoPermission
			);
		})
	}
}

mod close_proposal {
	use super::*;
	use crate::VoteRatio;

	#[test]
	fn close_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			System::set_block_number(200);

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_ok!(Voting::close_proposal(RuntimeOrigin::signed(BOB), proposal_id));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(proposal, None);

			// Event
			System::assert_last_event(
				Event::ProposalClosed { proposal_id, ratio: VoteRatio::default() }.into(),
			);
		})
	}

	#[test]
	fn cannot_close_proposal_before_end() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			System::set_block_number(199);

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::close_proposal(RuntimeOrigin::signed(BOB), proposal_id),
				Error::<Test>::ProposalHasNotEndedYet
			);
		})
	}

	#[test]
	fn cannot_close_proposal_not_existing() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			System::set_block_number(200);

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::close_proposal(RuntimeOrigin::signed(BOB), proposal_id + 1),
				Error::<Test>::ProposalDoesNotExist
			);
		})
	}
}

pub struct ProposalBuilder {
	pub origin: mock::RuntimeOrigin,
	pub offchain_data: BoundedVec<u8, ProposalOffchainDataLimit>,
	pub kind: ProposalKind,
	pub account_list: Option<BoundedVec<u64, AccountSizeLimit>>,
	pub start_block: BlockNumber,
	pub end_block: BlockNumber,
}

impl ProposalBuilder {
	pub fn new() -> ProposalBuilder {
		let max_duration = <Test as pallet_voting::Config>::ProposalMaximumDuration::get();
		Self {
			origin: RawOrigin::Signed(ALICE).into(),
			offchain_data: BoundedVec::default(),
			kind: ProposalKind::default(),
			account_list: Some(BoundedVec::default()),
			start_block: u32::try_from(System::block_number()).unwrap_or(0),
			end_block: u32::try_from(System::block_number()).unwrap_or(0) + max_duration - 1,
		}
	}

	pub fn start(mut self, start_block: BlockNumber) -> Self {
		self.start_block = start_block;
		self
	}

	pub fn end(mut self, end_block: BlockNumber) -> Self {
		self.end_block = end_block;
		self
	}

	pub fn execute(self) -> DispatchResult {
		Voting::create_proposal(
			self.origin,
			self.offchain_data,
			self.kind,
			self.account_list,
			self.start_block as u64,
			self.end_block as u64,
		)
	}
}
