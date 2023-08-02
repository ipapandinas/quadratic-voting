use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

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
