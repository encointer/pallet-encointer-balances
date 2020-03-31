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

//! Mocks for the tokens module.

#![cfg(test)]

use support::{impl_outer_event, impl_outer_origin, parameter_types};
use support::{assert_noop, assert_ok};
use system;
use sp_runtime::{testing::Header, traits::IdentityLookup, Perbill};
use primitives::{hashing::blake2_256, sr25519, Blake2Hasher, Pair, Public, H256};
use encointer_currencies::{CurrencyIdentifier, Location, Degree};
//use test_client::AccountKeyring;
use super::*;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

mod tokens {
	pub use crate::Event;
}
mod currencies {
	pub use encointer_currencies::Event;
}
impl_outer_event! {
	pub enum TestEvent for Runtime {
		tokens<T>,
		currencies<T>,
	}
}

// Workaround for https://github.com/rust-lang/rust/issues/26925 . Remove when sorted.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runtime;
parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

type AccountId = u64;
impl system::Trait for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
}
pub type System = system::Module<Runtime>;

impl encointer_currencies::Trait for Runtime {
    type Event = TestEvent;
}

pub type EncointerCurrencies = encointer_currencies::Module<Runtime>;

impl Trait for Runtime {
	type Event = TestEvent;
}

pub type EncointerBalances = Module<Runtime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub struct ExtBuilder {
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {}
	}
}

impl ExtBuilder {

	pub fn build(self) -> runtime_io::TestExternalities {
		let mut t = system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		t.into()
	}
}

/// register a simple test currency with 3 meetup locations and well known bootstrappers
pub fn register_test_currency() -> CurrencyIdentifier {
    // all well-known keys are boottrappers for easy testen afterwards
    let alice = 1;
    let bob = 2;
    let charlie = 3;
    let dave = 4;
    let eve = 5;
    let ferdie = 6;
    
    let a = Location::default(); // 0, 0
    
    let b = Location {
        lat: Degree::from_num(1),
        lon: Degree::from_num(1),
    };
    let c = Location {
        lat: Degree::from_num(2),
        lon: Degree::from_num(2),
    };
    let loc = vec![a, b, c];
    let bs = vec![
        alice.clone(),
        bob.clone(),
        charlie.clone(),
        dave.clone(),
        eve.clone(),
        ferdie.clone(),
    ];
    assert_ok!(EncointerCurrencies::new_currency(
        Origin::signed(alice.clone()),
        loc.clone(),
        bs.clone()
    ));
    CurrencyIdentifier::from(blake2_256(&(loc, bs).encode()))
}
