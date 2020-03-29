//  Copyright (c) 2019 Alain Brenzikofer
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.


//! Unit tests for the tokens module.

#![cfg(test)]

use super::*;
use mock::{ExtBuilder, System, TestEvent, EncointerBalances, ALICE, BOB};
use support::{assert_noop, assert_ok};

use encointer_currencies::CurrencyIdentifier;


#[test]
fn issue_should_work() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(EncointerBalances::issue(cid, &ALICE, BalanceType::from_num(50)));
			assert_eq!(EncointerBalances::balance(cid, &ALICE), BalanceType::from_num(50));
			assert_eq!(EncointerBalances::total_issuance(cid), BalanceType::from_num(50));
		});
}

#[test]
fn burn_should_work() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(EncointerBalances::issue(cid, &ALICE, BalanceType::from_num(50)));
			assert_ok!(EncointerBalances::burn(cid, &ALICE, BalanceType::from_num(20)));
			assert_eq!(EncointerBalances::balance(cid, &ALICE), BalanceType::from_num(30));
			assert_eq!(EncointerBalances::total_issuance(cid), BalanceType::from_num(30));
		});
}

#[test]
fn transfer_should_work() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
			let cid = CurrencyIdentifier::default();
			assert_ok!(EncointerBalances::issue(cid, &ALICE, BalanceType::from_num(50)));
			assert_ok!(EncointerBalances::transfer(Some(ALICE).into(), BOB, cid, BalanceType::from_num(10)));
			assert_eq!(EncointerBalances::balance(cid, &ALICE), BalanceType::from_num(40));
			assert_eq!(EncointerBalances::balance(cid, &BOB), BalanceType::from_num(10));
			assert_eq!(EncointerBalances::total_issuance(cid), BalanceType::from_num(50));

			let transferred_event = TestEvent::tokens(RawEvent::Transferred(cid, ALICE, BOB, BalanceType::from_num(10)));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_noop!(
				EncointerBalances::transfer(Some(ALICE).into(), BOB, cid, BalanceType::from_num(60)),
				Error::BalanceTooLow.into(),
			);
		});
}
