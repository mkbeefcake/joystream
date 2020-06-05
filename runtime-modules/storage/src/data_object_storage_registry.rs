// Clippy linter requirement
// disable it because of the substrate lib design
// example:  pub NextRelationshipId get(next_relationship_id) build(|config: &GenesisConfig<T>|
#![allow(clippy::redundant_closure_call)]

// Do not delete! Cannot be uncommented by default, because of Parity decl_module! issue.
//#![warn(missing_docs)]

use crate::data_directory::{self, ContentIdExists};
use codec::{Codec, Decode, Encode};
use roles::actors;
use roles::traits::Roles;
use rstd::prelude::*;
use sr_primitives::traits::{MaybeSerialize, Member, SimpleArithmetic};
use srml_support::{decl_event, decl_module, decl_storage, ensure, Parameter};
use system::{self, ensure_signed};

/// The _Data object storage registry_ main _Trait_
pub trait Trait:
    timestamp::Trait
    + system::Trait
    + data_directory::Trait
    + bureaucracy::Trait<bureaucracy::Instance2>
{
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    type DataObjectStorageRelationshipId: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + MaybeSerialize
        + PartialEq;

    type Roles: Roles<Self>;
    type ContentIdExists: data_directory::ContentIdExists<Self>;
}

// TODO: migrate to the Substrate error style
static MSG_CID_NOT_FOUND: &str = "Content with this ID not found.";
static MSG_DOSR_NOT_FOUND: &str = "No data object storage relationship found for this ID.";
static MSG_ONLY_STORAGE_PROVIDER_MAY_CREATE_DOSR: &str =
    "Only storage providers can create data object storage relationships.";
static MSG_ONLY_STORAGE_PROVIDER_MAY_CLAIM_READY: &str =
    "Only the storage provider in a DOSR can decide whether they're ready.";

const DEFAULT_FIRST_RELATIONSHIP_ID: u32 = 1;

#[derive(Clone, Encode, Decode, PartialEq, Debug)]
pub struct DataObjectStorageRelationship<T: Trait> {
    pub content_id: <T as data_directory::Trait>::ContentId,
    pub storage_provider: T::AccountId,
    pub ready: bool,
}

decl_storage! {
    trait Store for Module<T: Trait> as DataObjectStorageRegistry {

        // Start at this value
        pub FirstRelationshipId get(first_relationship_id) config(first_relationship_id): T::DataObjectStorageRelationshipId = T::DataObjectStorageRelationshipId::from(DEFAULT_FIRST_RELATIONSHIP_ID);

        // Increment
        pub NextRelationshipId get(next_relationship_id) build(|config: &GenesisConfig<T>| config.first_relationship_id): T::DataObjectStorageRelationshipId = T::DataObjectStorageRelationshipId::from(DEFAULT_FIRST_RELATIONSHIP_ID);

        // Mapping of Data object types
        pub Relationships get(relationships): map T::DataObjectStorageRelationshipId => Option<DataObjectStorageRelationship<T>>;

        // Keep a list of storage relationships per CID
        pub RelationshipsByContentId get(relationships_by_content_id): map T::ContentId => Vec<T::DataObjectStorageRelationshipId>;

        // TODO save only if metadata exists and there is at least one relation w/ ready == true.
        ReadyContentIds get(ready_content_ids): Vec<T::ContentId> = vec![];

        // TODO need? it can be expressed via StorageProvidersByContentId
        pub StorageProviderServesContent get(storage_provider_serves_content):
            map (T::AccountId, T::ContentId) => bool;

        pub StorageProvidersByContentId get(storage_providers_by_content_id):
            map T::ContentId => Vec<T::AccountId>;
    }
}

decl_event! {
    pub enum Event<T> where
        <T as data_directory::Trait>::ContentId,
        <T as Trait>::DataObjectStorageRelationshipId,
        <T as system::Trait>::AccountId
    {
        DataObjectStorageRelationshipAdded(DataObjectStorageRelationshipId, ContentId, AccountId),
        DataObjectStorageRelationshipReadyUpdated(DataObjectStorageRelationshipId, bool),

        StorageProviderAddedContent(AccountId, ContentId),
        StorageProviderRemovedContent(AccountId, ContentId),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn add_relationship(origin, cid: T::ContentId) {
            // Origin has to be a storage provider
            let who = ensure_signed(origin)?;

            // Check that the origin is a storage provider
            ensure!(<T as Trait>::Roles::account_has_role(&who, actors::Role::StorageProvider), MSG_ONLY_STORAGE_PROVIDER_MAY_CREATE_DOSR);

            // Content ID must exist
            ensure!(T::ContentIdExists::has_content(&cid), MSG_CID_NOT_FOUND);

            // Create new ID, data.
            let new_id = Self::next_relationship_id();
            let dosr: DataObjectStorageRelationship<T> = DataObjectStorageRelationship {
                content_id: cid,
                storage_provider: who.clone(),
                ready: false,
            };

            <Relationships<T>>::insert(new_id, dosr);
            <NextRelationshipId<T>>::mutate(|n| { *n += T::DataObjectStorageRelationshipId::from(1); });

            // Also add the DOSR to the list of DOSRs for the CID. Uniqueness is guaranteed
            // by the map, so we can just append the new_id to the list.
            let mut dosr_list = Self::relationships_by_content_id(cid);
            dosr_list.push(new_id);
            <RelationshipsByContentId<T>>::insert(cid, dosr_list);

            // Emit event
            Self::deposit_event(RawEvent::DataObjectStorageRelationshipAdded(new_id, cid, who));
        }

        // A storage provider may flip their own ready state, but nobody else.
        pub fn set_relationship_ready(origin, id: T::DataObjectStorageRelationshipId) {
            Self::toggle_dosr_ready(origin, id, true)?;
        }

        pub fn unset_relationship_ready(origin, id: T::DataObjectStorageRelationshipId) {
            Self::toggle_dosr_ready(origin, id, false)?;
        }
    }
}

impl<T: Trait> Module<T> {
    fn toggle_dosr_ready(
        origin: T::Origin,
        id: T::DataObjectStorageRelationshipId,
        ready: bool,
    ) -> Result<(), &'static str> {
        // Origin has to be the storage provider mentioned in the DOSR
        let who = ensure_signed(origin)?;

        // For that, we need to fetch the identified DOSR
        let mut dosr = Self::relationships(id).ok_or(MSG_DOSR_NOT_FOUND)?;
        ensure!(
            dosr.storage_provider == who,
            MSG_ONLY_STORAGE_PROVIDER_MAY_CLAIM_READY
        );

        // Flip to ready
        dosr.ready = ready;

        // Update DOSR and fire event.
        <Relationships<T>>::insert(id, dosr);
        Self::deposit_event(RawEvent::DataObjectStorageRelationshipReadyUpdated(
            id, true,
        ));

        Ok(())
    }
}
