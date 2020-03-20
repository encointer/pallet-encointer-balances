//  Copyright (c) 2019 laminar.one
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
use system;
use primitives::H256;
use sr_primitives::{testing::Header, traits::IdentityLookup, Perbill};
use encointer_currencies::CurrencyIdentifier;

use super::*;

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

mod tokens {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		tokens<T>,
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
	type Hashing = ::sr_primitives::traits::BlakeTwo256;
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

pub type Balance = u64;
impl Trait for Runtime {
	type Event = TestEvent;
	type Balance = Balance;
	type Amount = i64;
}

pub type Tokens = Module<Runtime>;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;

pub struct ExtBuilder {
	endowed_accounts: Vec<AccountId>,
	initial_balance: Balance,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![0],
			initial_balance: 0,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, account_ids: Vec<AccountId>, initial_balance: Balance) -> Self {
		self.endowed_accounts = account_ids;
		self.initial_balance = initial_balance;
		self
	}

	pub fn one_hundred_for_alice_n_bob(self) -> Self {
		self.balances(vec![ALICE, BOB], 100)
	}

	pub fn build(self) -> runtime_io::TestExternalities {
		let mut t = system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		GenesisConfig::<Runtime> {
			tokens: vec![CurrencyIdentifier::default()],
			initial_balance: self.initial_balance,
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
