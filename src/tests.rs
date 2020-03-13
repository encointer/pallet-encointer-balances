//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use mock::{Balance, ExtBuilder, System, TestEvent, Tokens, ALICE, BOB};
use support::{assert_noop, assert_ok};

use encointer_currencies::CurrencyIdentifier;

#[test]
fn genesis_issuance_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_eq!(Tokens::balance(cid, &ALICE), 100);
			assert_eq!(Tokens::balance(cid, &BOB), 100);
			assert_eq!(Tokens::total_issuance(cid), 200);
		});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(Tokens::transfer(Some(ALICE).into(), BOB, cid, 50));
			assert_eq!(Tokens::balance(cid, &ALICE), 50);
			assert_eq!(Tokens::balance(cid, &BOB), 150);
			assert_eq!(Tokens::total_issuance(cid), 200);

			let transferred_event = TestEvent::tokens(RawEvent::Transferred(cid, ALICE, BOB, 50));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_noop!(
				Tokens::transfer(Some(ALICE).into(), BOB, cid, 60),
				Error::BalanceTooLow.into(),
			);
		});
}

#[test]
fn deposit_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(Tokens::deposit(cid, &ALICE, 100));
			assert_eq!(Tokens::balance(cid, &ALICE), 200);
			assert_eq!(Tokens::total_issuance(cid), 300);

			assert_noop!(
				Tokens::deposit(cid, &ALICE, Balance::max_value()),
				Error::TotalIssuanceOverflow,
			);
		});
}

#[test]
fn withdraw_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(Tokens::withdraw(cid, &ALICE, 50));
			assert_eq!(Tokens::balance(cid, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(cid), 150);

			assert_noop!(Tokens::withdraw(cid, &ALICE, 60), Error::BalanceTooLow);
		});
}

#[test]
fn slash_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			// slashed_amount < amount
			assert_eq!(Tokens::slash(cid, &ALICE, 50), 0);
			assert_eq!(Tokens::balance(cid, &ALICE), 50);
			assert_eq!(Tokens::total_issuance(cid), 150);

			// slashed_amount == amount
			assert_eq!(Tokens::slash(cid, &ALICE, 51), 1);
			assert_eq!(Tokens::balance(cid, &ALICE), 0);
			assert_eq!(Tokens::total_issuance(cid), 100);
		});
}

#[test]
fn update_balance_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(Tokens::update_balance(cid, &ALICE, 50));
			assert_eq!(Tokens::balance(cid, &ALICE), 150);
			assert_eq!(Tokens::total_issuance(cid), 250);

			assert_ok!(Tokens::update_balance(cid, &BOB, -50));
			assert_eq!(Tokens::balance(cid, &BOB), 50);
			assert_eq!(Tokens::total_issuance(cid), 200);

			assert_noop!(Tokens::update_balance(cid, &BOB, -60), Error::BalanceTooLow);
		});
}

#[test]
fn issue_encointer_balance_should_work() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(Tokens::deposit(cid, &ALICE, 50));
			assert_eq!(Tokens::total_issuance(cid), 50);
		});
}