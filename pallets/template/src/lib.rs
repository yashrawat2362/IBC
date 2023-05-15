#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::{
	codec::{Decode, Encode},
	inherent::Vec,
	sp_runtime::RuntimeDebug,
};
use scale_info::TypeInfo;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Votes {
	Yes,
	No,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Vote<AccountId> {
	total_yes: Vec<AccountId>,
	total_no: Vec<AccountId>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::Error::MemberAlreadyRequested;
	use frame_support::pallet_prelude::*;
	use frame_support::traits::OnTimestampSet;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	#[pallet::getter(fn memberrequested)]
	pub type MemberRequested<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn daousers)]
	pub type DaoUsers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal)]
	pub type Proposal<T: Config> =
		StorageMap<_, Identity, T::Hash, Vote<T::AccountId>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposaltime)]
	pub type ProposalTime<T: Config> =
	StorageMap<_, Identity, T::BlockNumber, T::Hash, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MemberRequestedToJoin { who: T::AccountId },
		MemberAddedToDao { who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MemberAlreadyRequested,
		MemberAlreadyPresentInDao,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			let is_proposal_expire = ProposalTime::<T>::contains_key(n);

			if is_proposal_expire {
				let Propsal_id = ProposalTime::<T>::get(n).unwrap();
				ProposalTime::<T>::remove(n);
				Self::deposit_event(Event::ProposalIdRemoved{
					id: Propsal_id,
				})
			}

			Weight::zero()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn request_to_join(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let mut all_members = MemberRequested::<T>::get();
			let index = all_members
				.binary_search(&who)
				.err()
				.ok_or(Error::<T>::MemberAlreadyRequested)?;

			all_members.insert(index, who.clone());
			MemberRequested::<T>::put(all_members);

			Self::deposit_event(Event::<T>::MemberRequestedToJoin { who });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn add_requested_user_into_dao_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			let mut all_dao_member = DaoUsers::<T>::get();
			let index = all_dao_member.binary_search(&who).err().ok_or(Error::<T>::MemberAlreadyPresentInDao)?;

			all_dao_member.insert(index, who.clone());

			DaoUsers::<T>::put(all_dao_member);

			Self::deposit_event(Event::<T>::MemberAddedToDao {who});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn propose_proposal(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			proposal: T::Hash,
			approve: Votes,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn check_status_of_proposal(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(10_000)]
		pub fn set_proposal_time(origin: OriginFor<T>, proposal: T::Hash, time_duration_in_days: u32) -> DispatchResult {

			ensure_signed(origin.clone())?;

			let prod_block_per_sec = 6;
			let block_per_day = 14_400;
			let total_block = block_per_day * time_duration_in_days;

			let mut expire_block =  frame_system::Pallet::<T>::block_number() + total_block.into();

			ProposalTime::<T>::insert(expire_block, proposal);
			Ok(())
		}

	}
}
