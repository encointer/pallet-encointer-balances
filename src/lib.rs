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

#![cfg_attr(not(feature = "std"), no_std)]

use support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use rstd::{
	convert::{TryFrom, TryInto},
	result,
};
use sr_primitives::traits::{
	CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, SimpleArithmetic, StaticLookup,
};
use system::{self as system, ensure_signed};
use fixed::{types::I64F64, traits::{FixedSigned}};
use encointer_currencies::CurrencyIdentifier;

mod mock;
mod tests;

// We're working with fixpoint here.
type BalanceType = I64F64;
pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as EncointerBalances {
		pub TotalIssuance: map CurrencyIdentifier => BalanceType;
		pub Balance: double_map CurrencyIdentifier, blake2_256(T::AccountId) => BalanceType;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
	{
		/// Token transfer success (currency_id, from, to, amount)
		Transferred(CurrencyIdentifier, AccountId, AccountId, BalanceType),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Transfer some balance to another account.
		pub fn transfer(
			origin,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdentifier,
			amount: BalanceType,
		) {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			Self::transfer_(currency_id, &from, &to, amount)?;

			Self::deposit_event(RawEvent::Transferred(currency_id, from, to, amount));
		}
	}
}

decl_error! {
	/// Error for token module.
	pub enum Error {
		BalanceTooLow,
		TotalIssuanceOverflow,
	}
}

impl<T: Trait> Module<T> {

	fn balance(currency_id: CurrencyIdentifier, who: &T::AccountId) -> BalanceType {
		// TODO: apply demurrage
		<Balance<T>>::get(currency_id, who)
	}

	fn total_issuance(currency_id: CurrencyIdentifier) -> BalanceType {
		// TODO: apply demurrage
		<TotalIssuance>::get(currency_id)
	}

	fn transfer_(
		currency_id: CurrencyIdentifier,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: BalanceType,
	) -> result::Result<(), Error> {
		// TODO: apply demurrage
		ensure!(Self::balance(currency_id, from) >= amount, Error::BalanceTooLow);

		if from != to {
			<Balance<T>>::mutate(currency_id, from, |balance| *balance -= amount);
			<Balance<T>>::mutate(currency_id, to, |balance| *balance += amount);
		}

		Ok(())
	}

	fn issue(
		currency_id: CurrencyIdentifier,
		who: &T::AccountId,
		amount: BalanceType,
	) -> result::Result<(), Error> {
		ensure!(
			Self::total_issuance(currency_id).checked_add(amount).is_some(),
			Error::TotalIssuanceOverflow,
		);
		// TODO: apply demurrage first
		<TotalIssuance>::mutate(currency_id, |v| *v += amount);
		<Balance<T>>::mutate(currency_id, who, |v| *v += amount);

		Ok(())
	}

	fn burn(
		currency_id: CurrencyIdentifier,
		who: &T::AccountId,
		amount: BalanceType,
	) -> result::Result<(), Error> {
		ensure!(
			Self::balance(currency_id, who).checked_sub(amount).is_some(),
			Error::BalanceTooLow,
		);
		// TODO: apply demurrage first
		<TotalIssuance>::mutate(currency_id, |v| *v -= amount);
		<Balance<T>>::mutate(currency_id, who, |v| *v -= amount);
		Ok(())
	}
}

