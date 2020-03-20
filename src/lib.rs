//  Copyright (c) 2019 Alain Brenzikofer and laminar.one
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
// FIXME: `pallet/palette-` prefix should be used for all pallet modules, but currently `system`
// would cause compiling error in `decl_module!` and `construct_runtime!`
// #3295 https://github.com/paritytech/substrate/issues/3295
use system::{self as system, ensure_signed};

use encointer_currencies::CurrencyIdentifier;

pub mod traits;
use traits::{
	arithmetic::{self, Signed},
	MultiCurrency, MultiCurrencyExtended,
};

mod mock;
mod tests;

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Balance: Parameter + Member + SimpleArithmetic + Default + Copy + MaybeSerializeDeserialize;
	type Amount: Signed
		+ TryInto<Self::Balance>
		+ TryFrom<Self::Balance>
		+ Parameter
		+ Member
		+ arithmetic::SimpleArithmetic
		+ Default
		+ Copy
		+ MaybeSerializeDeserialize;
}

decl_storage! {
	trait Store for Module<T: Trait> as EncointerBalances {
		/// The total issuance of a token type.
		pub TotalIssuance get(fn total_issuance) build(|config: &GenesisConfig<T>| {
			let issuance = config.initial_balance * (config.endowed_accounts.len() as u32).into();
			config.tokens.iter().map(|id| (id.clone(), issuance)).collect::<Vec<_>>()
		}): map CurrencyIdentifier => T::Balance;

		/// The balance of a token type under an account.
		pub Balance get(fn balance): double_map CurrencyIdentifier, blake2_256(T::AccountId) => T::Balance;
	}
	add_extra_genesis {
		config(tokens): Vec<CurrencyIdentifier>;
		config(initial_balance): T::Balance;
		config(endowed_accounts): Vec<T::AccountId>;

		build(|config: &GenesisConfig<T>| {
			config.tokens.iter().for_each(|currency_id| {
				config.endowed_accounts.iter().for_each(|account_id| {
					<Balance<T>>::insert(currency_id, account_id, &config.initial_balance);
				})
			})
		})
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
		<T as Trait>::Balance
	{
		/// Token transfer success (currency_id, from, to, amount)
		Transferred(CurrencyIdentifier, AccountId, AccountId, Balance),
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
			#[compact] amount: T::Balance,
		) {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as MultiCurrency<_>>::transfer(currency_id, &from, &to, amount)?;

			Self::deposit_event(RawEvent::Transferred(currency_id, from, to, amount));
		}
	}
}

decl_error! {
	/// Error for token module.
	pub enum Error {
		BalanceTooLow,
		TotalIssuanceOverflow,
		AmountIntoBalanceFailed,
	}
}

impl<T: Trait> Module<T> {}

impl<T: Trait> MultiCurrency<T::AccountId> for Module<T> {
	type Balance = T::Balance;
	type CurrencyId = CurrencyIdentifier;
	type Error = Error;

	fn total_issuance(currency_id: CurrencyIdentifier) -> Self::Balance {
		<TotalIssuance<T>>::get(currency_id)
	}

	fn balance(currency_id: CurrencyIdentifier, who: &T::AccountId) -> Self::Balance {
		// TODO: apply demurrage
		<Balance<T>>::get(currency_id, who)
	}

	fn transfer(
		currency_id: CurrencyIdentifier,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		// TODO: apply demurrage
		ensure!(Self::balance(currency_id, from) >= amount, Error::BalanceTooLow);

		if from != to {
			<Balance<T>>::mutate(currency_id, from, |balance| *balance -= amount);
			<Balance<T>>::mutate(currency_id, to, |balance| *balance += amount);
		}

		Ok(())
	}

	fn deposit(
		currency_id: CurrencyIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		ensure!(
			Self::total_issuance(currency_id).checked_add(&amount).is_some(),
			Error::TotalIssuanceOverflow,
		);

		<TotalIssuance<T>>::mutate(currency_id, |v| *v += amount);
		<Balance<T>>::mutate(currency_id, who, |v| *v += amount);

		Ok(())
	}

	fn withdraw(
		currency_id: CurrencyIdentifier,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> result::Result<(), Self::Error> {
		ensure!(
			Self::balance(currency_id, who).checked_sub(&amount).is_some(),
			Error::BalanceTooLow,
		);

		<TotalIssuance<T>>::mutate(currency_id, |v| *v -= amount);
		<Balance<T>>::mutate(currency_id, who, |v| *v -= amount);

		Ok(())
	}

	fn slash(currency_id: CurrencyIdentifier, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		let slashed_amount = Self::balance(currency_id, who).min(amount);
		<TotalIssuance<T>>::mutate(currency_id, |v| *v -= slashed_amount);
		<Balance<T>>::mutate(currency_id, who, |v| *v -= slashed_amount);
		amount - slashed_amount
	}
}

impl<T: Trait> MultiCurrencyExtended<T::AccountId> for Module<T> {
	type Amount = T::Amount;

	fn update_balance(
		currency_id: CurrencyIdentifier,
		who: &T::AccountId,
		by_amount: Self::Amount,
	) -> Result<(), Self::Error> {
		let by_balance =
			TryInto::<Self::Balance>::try_into(by_amount.abs()).map_err(|_| Error::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(currency_id, who, by_balance)
		} else {
			Self::withdraw(currency_id, who, by_balance)
		}
	}
}
