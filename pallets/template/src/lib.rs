#![cfg_attr(not(feature = "std"), no_std)]


// Documentation...


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
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use crate::Event::ProposedProposal;

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
	#[pallet::getter(fn memberrequestedfordao)]
	pub type MemberRequestedForDao<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn daousers)]
	pub type DaoUsers<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposal)]
	pub type Proposal<T: Config> =
		StorageMap<_, Identity, T::Hash, Vote<T::AccountId>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MemberRequestedToJoin { who: T::AccountId },
		MemberAddedToDao { who: T::AccountId },
		ProposedProposal {proposal_id: T::Hash},
		ProposalVoted{
			who: T::AccountId,
			proposal_id: T::Hash,
			recent_vote: Votes,
			total_yes: Vec<T::AccountId>,
			total_no : Vec<T::AccountId>,
		}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		MemberAlreadyRequested,
		MemberAlreadyPresentInDao,
		MemberNotPresentInDao,
		InvalidProposal,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn request_to_join(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let mut all_members = MemberRequestedForDao::<T>::get();
			let index = all_members
				.binary_search(&who)
				.err()
				.ok_or(Error::<T>::MemberAlreadyRequested)?;

			all_members.insert(index, who.clone());
			MemberRequestedForDao::<T>::put(all_members);

			Self::deposit_event(Event::<T>::MemberRequestedToJoin { who });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn add_requested_user_into_dao_member(origin: OriginFor<T>, who: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			let mut all_dao_member = DaoUsers::<T>::get();
			let index = all_dao_member.binary_search(&who).err().ok_or(Error::<T>::MemberAlreadyPresentInDao)?;

			// Need to check if member is not present in member request storage.

			all_dao_member.insert(index, who.clone());

			DaoUsers::<T>::put(all_dao_member);

			// need to remove this member from requestmembertodao storage.

			Self::deposit_event(Event::<T>::MemberAddedToDao {who});

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn propose_proposal(origin: OriginFor<T>, proposal_id: T::Hash) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			let votes = Vote {
				total_yes : Vec::new(),
				total_no : Vec::new(),
			};

			Proposal::<T>::insert(proposal_id, votes);
			// need to add who in event.
			Self::deposit_event(Event::ProposedProposal{
				proposal_id
			});
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn approve_proposal(
			origin: OriginFor<T>,
			proposal_id: T::Hash,
			approve: Votes,
		) -> DispatchResult {

			let who = ensure_signed(origin)?;

			let all_dao_users = DaoUsers::<T>::get();

			ensure!(all_dao_users.contains(&who), Error::<T>::MemberNotPresentInDao);



			match approve {
				Votes::Yes => {
					Proposal::<T>::mutate(&proposal_id, |mut info| {

						let total_votes = info.as_mut().unwrap();
						let mut total_yes_votes = &mut total_votes.total_yes;
						let mut total_no_votes = &mut total_votes.total_no;

						&total_yes_votes.push(who.clone());

						Self::deposit_event(Event::<T>::ProposalVoted{
							who,
							proposal_id,
							recent_vote: approve,
							total_yes: total_yes_votes.clone(),
							total_no : total_no_votes.clone(),
						});
					});


				}
				Votes::No => {
					Proposal::<T>::mutate(&proposal_id, |mut info| {

						let total_votes = info.as_mut().unwrap();
						let mut total_yes_votes = &mut total_votes.total_yes;
						let mut total_no_votes = &mut total_votes.total_no;

						&total_no_votes.push(who.clone());

						Self::deposit_event(Event::<T>::ProposalVoted{
							who,
							proposal_id,
							recent_vote: approve,
							total_yes: total_yes_votes.clone(),
							total_no : total_no_votes.clone(),
						});
					});
				}
			}


			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn check_status_of_proposal(origin: OriginFor<T>, proposal: T::Hash) -> DispatchResult {
			Ok(())
		}

	}
}
