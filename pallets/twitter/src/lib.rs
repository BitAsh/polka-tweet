//! A decentralized twitter based on Substrate

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};

use codec::{Encode, Decode};
use sp_std::prelude::*;
use sp_runtime::{RuntimeDebug, DispatchResult};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use frame_system::ensure_signed;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type TweetId = u128;

/// Tweet
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub struct Tweet<AccountId, BlockNumber> {
	/// Identifier of the retweet.
	id: TweetId,
	/// Created at, by block number.
	create_at: BlockNumber,
	/// Identifier of the original tweet.
	quote_tweet_id: Option<TweetId>,
	/// Text of the retweet.
	text: Vec<u8>,
	/// The comments of the retweet.
	comments: Vec<TweetId>,
	/// Author of the retweet.
	author: AccountId,
}

pub type TweetOf<T> = Tweet<<T as frame_system::Trait>::AccountId, <T as frame_system::Trait>::BlockNumber>;

pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}


decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Accounts get(fn accounts): map hasher(blake2_128_concat) T::AccountId => Vec<TweetId>;
		Tweets get(fn tweets): map hasher(blake2_128_concat) TweetId => Option<TweetOf<T>>;
		NextTweetId get(fn next_tweet_id): TweetId;
	}
}

decl_event!(
	pub enum Event<T> where Tweet = TweetOf<T> {
		Tweeted(Tweet),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Tweet not found.
		TweetNotFound,
		/// Text too long.
		TweetTooLong,
		/// Run out of tweet id.
		NoAvailableTweetId,
	}
}

pub const MAX_TEXT_LEN: u64 = 140;

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn new_tweet(origin, text: Vec<u8>) {
			let author = ensure_signed(origin)?;

			ensure!(text.len() <= 140, Error::<T>::TweetTooLong);

			let new_id = Self::alloc_id().ok_or(Error::<T>::NoAvailableTweetId)?;
			let tweet = Tweet {
				id: new_id,
				create_at: <frame_system::Module<T>>::block_number(),
				quote_tweet_id: None,
				text,
				comments: vec![],
				author: author.clone(),
			};

			<Accounts<T>>::mutate(&author, |tweets| {
				tweets.push(new_id);
			});
			<Tweets<T>>::insert(new_id, tweet.clone());

			Self::deposit_event(RawEvent::Tweeted(tweet));
		}

		#[weight = 10_000]
		pub fn retweet(origin, tweet_id: TweetId, text: Vec<u8>) {
			let author = ensure_signed(origin)?;

			ensure!(text.len() <= 140, Error::<T>::TweetTooLong);
			ensure!(Self::tweets(tweet_id).is_none(), Error::<T>::TweetNotFound);

			let new_id = Self::alloc_id().ok_or(Error::<T>::NoAvailableTweetId)?;
			let tweet = Tweet {
				id: new_id,
				create_at: <frame_system::Module<T>>::block_number(),
				quote_tweet_id: Some(tweet_id),
				text,
				comments: vec![],
				author: author.clone(),
			};

			<Accounts<T>>::mutate(&author, |tweets| {
				tweets.push(new_id);
			});
			<Tweets<T>>::insert(new_id, tweet.clone());

			Self::deposit_event(RawEvent::Tweeted(tweet));
		}

		#[weight = 10_000]
		pub fn comment(origin, text: Vec<u8>, tweet_id: TweetId) {
			let author = ensure_signed(origin)?;

			ensure!(text.len() <= 140, Error::<T>::TweetTooLong);
			ensure!(Self::tweets(tweet_id).is_none(), Error::<T>::TweetNotFound);

			let new_id = Self::alloc_id().ok_or(Error::<T>::NoAvailableTweetId)?;
			let comment = Tweet {
				id: new_id,
				create_at: <frame_system::Module<T>>::block_number(),
				quote_tweet_id: None,
				text,
				comments: vec![],
				author: author.clone(),
			};

			<Tweets<T>>::try_mutate_exists(&tweet_id, |maybe_tweet| -> DispatchResult {
				let tweet = maybe_tweet.as_mut().ok_or(Error::<T>::TweetNotFound)?;
				tweet.comments.push(new_id);
				Ok(())
			})?;
			<Accounts<T>>::mutate(&author, |tweets| {
				tweets.push(new_id);
			});
			<Tweets<T>>::insert(new_id, comment.clone());

			Self::deposit_event(RawEvent::Tweeted(comment));
		}
	}
}

impl<T: Trait> Module<T> {
	fn alloc_id() -> Option<TweetId> {
		let next = Self::next_tweet_id();

		let new_next = next.checked_add(1)?;
		NextTweetId::put(new_next);

		return Some(next);
	}
}
