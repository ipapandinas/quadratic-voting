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
				Event::ProposalCreated {
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

			let next_proposal_id = Voting::get_next_proposal_id();
			assert_noop!(
				Voting::cancel_proposal(RuntimeOrigin::signed(ALICE), next_proposal_id),
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
				Event::VoteCompleted { proposal_id, ratio: VoteRatio::default() }.into(),
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

			let next_proposal_id = Voting::get_next_proposal_id();
			assert_noop!(
				Voting::close_proposal(RuntimeOrigin::signed(BOB), next_proposal_id),
				Error::<Test>::ProposalDoesNotExist
			);
		})
	}
}

mod set_account_list {
	use crate::ProposalData;

	use super::*;

	#[test]
	fn set_account_list() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 10;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());
			let proposal_id = Voting::get_next_proposal_id() - 1;
			let raw_proposal = Voting::proposals(proposal_id);
			assert!(raw_proposal.is_some());

			let new_account_list = BoundedVec::try_from(vec![BOB]).unwrap();
			assert_ok!(Voting::set_account_list(
				RuntimeOrigin::signed(ALICE),
				proposal_id,
				Some(new_account_list.clone())
			));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(
				proposal,
				Some(ProposalData {
					account_list: Some(new_account_list.clone()),
					..raw_proposal.unwrap()
				})
			);

			// Event
			System::assert_last_event(
				Event::AccountListSet { proposal_id, account_list: Some(new_account_list) }.into(),
			);
		})
	}

	#[test]
	fn cannot_set_account_list_after_start() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			let new_account_list = BoundedVec::try_from(vec![BOB]).unwrap();
			assert_noop!(
				Voting::set_account_list(
					RuntimeOrigin::signed(ALICE),
					proposal_id,
					Some(new_account_list)
				),
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

			let next_proposal_id = Voting::get_next_proposal_id();
			let new_account_list = BoundedVec::try_from(vec![BOB]).unwrap();
			assert_noop!(
				Voting::set_account_list(
					RuntimeOrigin::signed(ALICE),
					next_proposal_id,
					Some(new_account_list)
				),
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
			let new_account_list = BoundedVec::try_from(vec![BOB]).unwrap();
			assert_noop!(
				Voting::set_account_list(
					RuntimeOrigin::signed(BOB),
					proposal_id,
					Some(new_account_list)
				),
				Error::<Test>::OriginNoPermission
			);
		})
	}
}

mod vote {
	use frame_support::traits::fungible::freeze::Inspect;
	use sp_core::Get;

	use crate::VoteInfo;

	use super::*;

	fn vote_setup() {
		let start_block = 1;
		let end_block = 200;
		assert_ok!(Voting::register_voter(RuntimeOrigin::root(), ALICE));
		assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());
	}

	#[test]
	fn works_only_if_registered_voter() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(2), 0, true, 1),
				Error::<Test>::VoterNotRegistered
			);
		})
	}

	#[test]
	fn cannot_vote_proposal_not_existing() {
		new_test_ext().execute_with(|| {
			setup();
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(ALICE), 1, true, 1),
				Error::<Test>::ProposalDoesNotExist
			);
		})
	}

	#[test]
	fn cannot_vote_proposal_not_started() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 10;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(ALICE), proposal_id, true, 1),
				Error::<Test>::ProposalHasNotStartedYet
			);
		})
	}

	#[test]
	fn cannot_vote_proposal_finished() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 10;
			let end_block = 200;
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			System::set_block_number(201);

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(ALICE), proposal_id, true, 1),
				Error::<Test>::ProposalHasAlreadyEnded
			);
		})
	}

	#[test]
	fn banned_voter_cannot_vote() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			let account_list = BoundedVec::try_from(vec![BOB]).unwrap();
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), BOB));
			setup();

			assert_ok!(ProposalBuilder::new()
				.start(start_block)
				.end(end_block)
				.set_account_list(Some(account_list))
				.execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(BOB), proposal_id, true, 1),
				Error::<Test>::OriginNoPermission
			);
		})
	}

	#[test]
	fn not_allowed_voter_cannot_vote_private_proposal() {
		new_test_ext().execute_with(|| {
			System::set_block_number(1);
			let start_block = 1;
			let end_block = 200;
			let account_list = BoundedVec::try_from(vec![ALICE]).unwrap();
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), BOB));
			setup();

			assert_ok!(ProposalBuilder::new()
				.start(start_block)
				.end(end_block)
				.set_account_list(Some(account_list))
				.private()
				.execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(BOB), proposal_id, true, 1),
				Error::<Test>::OriginNoPermission
			);
		})
	}

	#[test]
	fn ext_builder_balance_setup_works() {
		ExtBuilder::new_build(vec![(ALICE, 10), (BOB, 100)]).execute_with(|| {
			let alice_balance = Balances::free_balance(ALICE);
			assert!(alice_balance == 10);

			let bob_balance = Balances::free_balance(BOB);
			assert!(bob_balance == 100);

			let freeze_id: () =
				<<Test as pallet_voting::Config>::FreezeIdForPallet as Get<_>>::get();
			let alice_balance_frozen_balance =
				<<Test as crate::Config>::NativeBalance as Inspect<
					<Test as frame_system::Config>::AccountId,
				>>::balance_frozen(&freeze_id, &ALICE);
			assert_eq!(alice_balance_frozen_balance, 0);

			let bob_balance_frozen_balance = <<Test as crate::Config>::NativeBalance as Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::balance_frozen(&freeze_id, &BOB);
			assert_eq!(bob_balance_frozen_balance, 0);
		})
	}

	#[test]
	fn cannot_vote_with_insufficient_funds() {
		ExtBuilder::new_build(vec![(ALICE, 10)]).execute_with(|| {
			let start_block = 1;
			let end_block = 200;
			let aye = true;
			let power = 4; // 16 tokens required
			setup();

			assert_ok!(ProposalBuilder::new().start(start_block).end(end_block).execute());

			let proposal_id = Voting::get_next_proposal_id() - 1;
			assert_noop!(
				Voting::vote(RuntimeOrigin::signed(ALICE), proposal_id, aye, power),
				Error::<Test>::InsufficientBalance
			);
		})
	}

	#[test]
	fn single_vote() {
		ExtBuilder::new_build(vec![(ALICE, 10)]).execute_with(|| {
			let freeze_id: () =
				<<Test as pallet_voting::Config>::FreezeIdForPallet as Get<_>>::get();

			let aye = true;
			let power = 3; // 9 tokens required
			let quadratic_amount = Voting::calculate_quadratic_amount(power);
			vote_setup();

			let proposal_id = Voting::next_proposal_id() - 1;

			// Execution
			assert_ok!(Voting::vote(RuntimeOrigin::signed(ALICE), proposal_id, aye, power));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(proposal.unwrap().ratio, (quadratic_amount, quadratic_amount));

			let alice_frozen_balance = <<Test as crate::Config>::NativeBalance as Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::balance_frozen(&freeze_id, &ALICE);
			assert_eq!(alice_frozen_balance, quadratic_amount);

			let vote = Voting::votes(ALICE, proposal_id);
			assert_eq!(vote, Some(VoteInfo { proposal_id, aye, power }));

			// Event
			System::assert_last_event(
				Event::VoteAdded { proposal_id, voter: ALICE, aye, power }.into(),
			);
		})
	}

	#[test]
	fn vote_adjustment() {
		ExtBuilder::new_build(vec![(ALICE, 100)]).execute_with(|| {
			let freeze_id: () =
				<<Test as pallet_voting::Config>::FreezeIdForPallet as Get<_>>::get();

			let init_aye = true;
			let init_power = 3; // 9 tokens required
			vote_setup();

			let proposal_id = Voting::next_proposal_id() - 1;

			// Initial vote execution
			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_id,
				init_aye,
				init_power
			));

			System::set_block_number(2);

			// Vote adjustment
			let second_aye = true;
			let second_power = 4; // 16 tokens required - diff = 7
			let second_quadratic_amount = Voting::calculate_quadratic_amount(second_power);

			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_id,
				second_aye,
				second_power
			));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(proposal.unwrap().ratio, (second_quadratic_amount, second_quadratic_amount));

			let alice_frozen_balance = <<Test as crate::Config>::NativeBalance as Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::balance_frozen(&freeze_id, &ALICE);
			assert_eq!(alice_frozen_balance, second_quadratic_amount);

			let vote = Voting::votes(ALICE, proposal_id);
			assert_eq!(vote, Some(VoteInfo { proposal_id, aye: second_aye, power: second_power }));

			// Event
			System::assert_last_event(
				Event::VoteAdded {
					proposal_id,
					voter: ALICE,
					aye: second_aye,
					power: second_power,
				}
				.into(),
			);
		})
	}

	#[test]
	fn retract_vote() {
		ExtBuilder::new_build(vec![(ALICE, 10)]).execute_with(|| {
			let freeze_id: () =
				<<Test as pallet_voting::Config>::FreezeIdForPallet as Get<_>>::get();

			let init_aye = true;
			let init_power = 3; // 9 tokens required
			vote_setup();

			let proposal_id = Voting::next_proposal_id() - 1;

			// Initial vote execution
			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_id,
				init_aye,
				init_power
			));

			System::set_block_number(2);

			// Vote adjustment
			let second_aye = true;
			let second_power = 0; // 16 tokens required - diff = 7
			let second_quadratic_amount = Voting::calculate_quadratic_amount(second_power);

			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_id,
				second_aye,
				second_power
			));

			// Storage
			let proposal = Voting::proposals(proposal_id);
			assert_eq!(proposal.unwrap().ratio, (second_quadratic_amount, second_quadratic_amount));

			let alice_frozen_balance = <<Test as crate::Config>::NativeBalance as Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::balance_frozen(&freeze_id, &ALICE);
			assert_eq!(alice_frozen_balance, second_quadratic_amount);

			let vote = Voting::votes(ALICE, proposal_id);
			assert_eq!(vote, None);

			// Event
			System::assert_last_event(Event::VoteDropped { proposal_id, voter: ALICE }.into());
		})
	}

	#[test]
	fn multiple_proposal_votes() {
		ExtBuilder::new_build(vec![(ALICE, 30)]).execute_with(|| {
			let freeze_id: () =
				<<Test as pallet_voting::Config>::FreezeIdForPallet as Get<_>>::get();

			let proposal_1_start_block = 1;
			let proposal_1_end_block = 200;
			let proposal_2_start_block = 10;
			let proposal_2_end_block = 210;
			assert_ok!(Voting::register_voter(RuntimeOrigin::root(), ALICE));
			assert_ok!(ProposalBuilder::new()
				.start(proposal_1_start_block)
				.end(proposal_1_end_block)
				.execute());
			let proposal_1_id = Voting::next_proposal_id() - 1;
			assert_ok!(ProposalBuilder::new()
				.start(proposal_2_start_block)
				.end(proposal_2_end_block)
				.execute());
			let proposal_2_id = Voting::next_proposal_id() - 1;

			let proposal_1_vote_aye = true;
			let proposal_1_vote_power = 3; // 9 tokens required
			let proposal_1_quadratic_amount =
				Voting::calculate_quadratic_amount(proposal_1_vote_power);
			let proposal_2_vote_aye = false;
			let proposal_2_vote_power = 4; // 16 tokens required
			let proposal_2_quadratic_amount =
				Voting::calculate_quadratic_amount(proposal_2_vote_power);

			// Vote proposal 1
			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_1_id,
				proposal_1_vote_aye,
				proposal_1_vote_power
			));

			System::set_block_number(20);

			// Vote proposal 2
			assert_ok!(Voting::vote(
				RuntimeOrigin::signed(ALICE),
				proposal_2_id,
				proposal_2_vote_aye,
				proposal_2_vote_power
			));

			// Storage
			let proposal_1 = Voting::proposals(proposal_1_id);
			assert_eq!(
				proposal_1.unwrap().ratio,
				(proposal_1_quadratic_amount, proposal_1_quadratic_amount)
			);

			let proposal_2 = Voting::proposals(proposal_2_id);
			assert_eq!(proposal_2.unwrap().ratio, (0, proposal_2_quadratic_amount));

			let alice_frozen_balance = <<Test as crate::Config>::NativeBalance as Inspect<
				<Test as frame_system::Config>::AccountId,
			>>::balance_frozen(&freeze_id, &ALICE);
			assert_eq!(
				alice_frozen_balance,
				proposal_1_quadratic_amount.saturating_add(proposal_2_quadratic_amount)
			);

			let vote_1 = Voting::votes(ALICE, proposal_1_id);
			assert_eq!(
				vote_1,
				Some(VoteInfo {
					proposal_id: proposal_1_id,
					aye: proposal_1_vote_aye,
					power: proposal_1_vote_power
				})
			);

			let vote_2 = Voting::votes(ALICE, proposal_2_id);
			assert_eq!(
				vote_2,
				Some(VoteInfo {
					proposal_id: proposal_2_id,
					aye: proposal_2_vote_aye,
					power: proposal_2_vote_power
				})
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

	pub fn private(mut self) -> Self {
		self.kind = ProposalKind::Private;
		self
	}

	pub fn set_account_list(
		mut self,
		account_list: Option<BoundedVec<u64, AccountSizeLimit>>,
	) -> Self {
		self.account_list = account_list;
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
