#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::permissions::*;
use crate::types::*;
use crate::{
    nft::{NftOwner, TransactionalStatus},
    Call, ChannelById, Config, ContentActor, Event, Module as Pallet,
};
use balances::Pallet as Balances;
use common::{build_merkle_path_helper, generate_merkle_root_helper, BudgetManager};
use frame_benchmarking::{benchmarks, Zero};
use frame_support::{
    storage::StorageMap,
    traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_arithmetic::traits::One;
use sp_runtime::traits::Hash;
use sp_runtime::SaturatedConversion;
use sp_std::{
    cmp::min,
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    convert::TryInto,
    vec,
};
use storage::Module as Storage;

benchmarks! {
    where_clause {
    where
    T: RuntimeConfig,
    T::AccountId: CreateAccountId
}

    /*
    ============================================================================
    ======================================== CHANNEL CUD GROUP =================
    ============================================================================
    */

    create_channel {

        let a in 1 .. T::MaxNumberOfCollaboratorsPerChannel::get(); //max colaborators

        let b in (T::StorageBucketsPerBagValueConstraint::get().min as u32) ..
            (T::StorageBucketsPerBagValueConstraint::get().max() as u32);

        let c in
        (T::DistributionBucketsPerBagValueConstraint::get().min as u32) ..
        (T::DistributionBucketsPerBagValueConstraint::get().max() as u32);

        let d in 1 .. T::MaxNumberOfAssetsPerChannel::get(); //max objs number

        let e in 1 .. MAX_BYTES_METADATA; //max bytes for metadata

        let max_obj_size: u64 = T::MaxDataObjectSize::get();

        let (_, storage_wg_lead_account_id) = insert_storage_leader::<T>();

        let (_, distribution_wg_lead_account_id) =
            insert_distribution_leader::<T>();

        let (_, lead_account_id) = insert_content_leader::<T>();

        let sender = RawOrigin::Signed(lead_account_id);

        let group_id = setup_worst_case_curator_group_with_curators::<T>(
            max_curators_per_group::<T>()
        )?;

        let channel_owner = ChannelOwner::CuratorGroup(group_id);

        let params = generate_channel_creation_params::<T>(
            storage_wg_lead_account_id,
            distribution_wg_lead_account_id,
            a, b, c, d, e,
            max_obj_size,
        );

    }: _ (sender, channel_owner, params.clone())
    verify {

        let channel_id: T::ChannelId = One::one();
        assert!(ChannelById::<T>::contains_key(&channel_id));

        let channel = ChannelById::<T>::get(channel_id);
        let channel_acc = ContentTreasury::<T>::account_for_channel(channel_id);

        assert_last_event::<T>(
            Event::<T>::ChannelCreated(
                channel_id,
                channel,
                params,
                channel_acc
            ).into()
        );
    }

    channel_update_with_assets {

        let a in 1 .. T::MaxNumberOfCollaboratorsPerChannel::get(); //max colaborators

        let b in 1 .. T::MaxNumberOfAssetsPerChannel::get(); //max objs number to upload

        let c in 1 .. T::MaxNumberOfAssetsPerChannel::get(); //max objs number to remove

        let d in 1 .. MAX_BYTES_METADATA; //max bytes for new metadata

        let e in (T::StorageBucketsPerBagValueConstraint::get().min as u32) ..
         (T::StorageBucketsPerBagValueConstraint::get().max() as u32);

        let max_obj_size: u64 = T::MaxDataObjectSize::get();

        let assets_to_remove: BTreeSet<T::DataObjectId> = (0..c).map(|i| i.saturated_into()).collect();

        let (channel_id,
            group_id,
            lead_account_id,
            curator_id,
            curator_account_id) =
        setup_worst_case_scenario_curator_channel::<T>(
            c,
            e,
            T::DistributionBucketsPerBagValueConstraint::get().max() as u32,
            false
        ).unwrap();

        let channel = ChannelById::<T>::get(channel_id);

        let permissions: ChannelAgentPermissions =
            worst_case_channel_agent_permissions()
            .into_iter()
            .skip(1)
            .collect();

        let collaborators = Some(channel.collaborators
            .into_iter()
            .take(a as usize)
            .map(|(member_id, _)|{
                (member_id, permissions.clone())
            })
            .collect::<BTreeMap<_, _>>());

        let assets_to_upload = StorageAssets::<T> {
                expected_data_size_fee:
                    Storage::<T>::data_object_per_mega_byte_fee(),
                object_creation_list: create_data_object_candidates_helper(
                    b,
                    max_obj_size
                ),
        };

        let new_data_object_ids: BTreeSet<T::DataObjectId> = (c..c+b).map(|i| i.saturated_into()).collect();

        let expected_data_object_state_bloat_bond =
            Storage::<T>::data_object_state_bloat_bond_value();

        let new_meta = Some(vec![0xff].repeat(d as usize));

        let update_params = ChannelUpdateParameters::<T> {
            assets_to_upload: Some(assets_to_upload),
            new_meta,
            assets_to_remove,
            collaborators,
            expected_data_object_state_bloat_bond,
            storage_buckets_num_witness: Some(storage_buckets_num_witness::<T>(channel_id)?),
        };

        let origin = RawOrigin::Signed(curator_account_id);
        let actor = ContentActor::Curator(group_id, curator_id);

    }: update_channel(
        origin, actor, channel_id, update_params.clone())
    verify {

        assert!(ChannelById::<T>::contains_key(&channel_id));

        assert_last_event::<T>(
            Event::<T>::ChannelUpdated(actor,
                channel_id,
                update_params,
                new_data_object_ids).into()
        );
    }

    channel_update_without_assets {

        let a in 1 .. T::MaxNumberOfCollaboratorsPerChannel::get(); //max colaborators

        let b in 1 .. MAX_BYTES_METADATA; //max bytes for new metadata

        let (channel_id,
            group_id,
            lead_account_id,
            curator_id,
            curator_account_id) =
        setup_worst_case_scenario_curator_channel::<T>(
            T::MaxNumberOfAssetsPerChannel::get(),
            T::StorageBucketsPerBagValueConstraint::get().max() as u32,
            T::DistributionBucketsPerBagValueConstraint::get().max() as u32,
            false
        ).unwrap();

        let channel = ChannelById::<T>::get(channel_id);

        let permissions: ChannelAgentPermissions =
            worst_case_channel_agent_permissions()
            .into_iter()
            .skip(1)
            .collect();

        let collaborators = Some(channel.collaborators
            .into_iter()
            .take(a as usize)
            .map(|(member_id, _)|{
                (member_id, permissions.clone())
            })
            .collect::<BTreeMap<_, _>>());

        let expected_data_object_state_bloat_bond =
            Storage::<T>::data_object_state_bloat_bond_value();

        let new_meta = Some(vec![0xff].repeat(b as usize));

        let update_params = ChannelUpdateParameters::<T> {
            assets_to_upload: None,
            new_meta,
            assets_to_remove: BTreeSet::new(),
            collaborators,
            expected_data_object_state_bloat_bond,
            storage_buckets_num_witness: None
        };

        let origin = RawOrigin::Signed(curator_account_id);
        let actor = ContentActor::Curator(group_id, curator_id);

    }: update_channel(
        origin, actor, channel_id, update_params.clone())
    verify {

        assert!(ChannelById::<T>::contains_key(&channel_id));

        assert_last_event::<T>(
            Event::<T>::ChannelUpdated(actor,
                channel_id,
                update_params,
                BTreeSet::new()).into()
        );
    }

    delete_channel {

        let a in 1 .. T::MaxNumberOfAssetsPerChannel::get(); //max objs number

        let b in (T::StorageBucketsPerBagValueConstraint::get().min as u32) ..
         (T::StorageBucketsPerBagValueConstraint::get().max() as u32);

        let c in
            (T::DistributionBucketsPerBagValueConstraint::get().min as u32) ..
            (T::DistributionBucketsPerBagValueConstraint::get().max() as u32);

        let max_obj_size: u64 = T::MaxDataObjectSize::get();

        let (
            channel_id,
            group_id,
            lead_account_id,
            curator_id,
            curator_account_id
        ) =
        setup_worst_case_scenario_curator_channel::<T>(a, b, c, true).unwrap();

        let origin = RawOrigin::Signed(curator_account_id);
        let actor = ContentActor::Curator(group_id, curator_id);
        let channel_bag_witness = channel_bag_witness::<T>(channel_id)?;
    }: _ (origin, actor, channel_id, channel_bag_witness, a.into())
    verify {

        assert_last_event::<T>(
            Event::<T>::ChannelDeleted(
                actor,
                channel_id
            ).into()
        );
    }

    /*
    ===============================================================================================
    ======================================== CURATOR GROUPS =======================================
    ===============================================================================================
     */

    create_curator_group {
        let a in 0 .. (T::MaxKeysPerCuratorGroupPermissionsByLevelMap::get() as u32);

        let (_, lead_account) = insert_content_leader::<T>();
        let group_id = Pallet::<T>::next_curator_group_id();
        let permissions_by_level: ModerationPermissionsByLevel::<T> = (0..a).map(
            |i| (i.saturated_into(), worst_case_content_moderation_actions_set())
        ).collect();
    }: _ (
        RawOrigin::Signed(lead_account),
        true,
        permissions_by_level.clone()
    )
        verify {
            let group = Pallet::<T>::curator_group_by_id(group_id);
            assert!(group == CuratorGroup::create(true, &permissions_by_level));
            assert_last_event::<T>(Event::<T>::CuratorGroupCreated(group_id).into());
        }

    update_curator_group_permissions {
        let a in 0 .. (T::MaxKeysPerCuratorGroupPermissionsByLevelMap::get() as u32);

        let (_, lead_account) = insert_content_leader::<T>();
        let group_id = setup_worst_case_curator_group_with_curators::<T>(
            max_curators_per_group::<T>()
        )?;
        let permissions_by_level: ModerationPermissionsByLevel::<T> = (0..a).map(
            |i| (i.saturated_into(), worst_case_content_moderation_actions_set())
        ).collect();
    }: _ (
        RawOrigin::Signed(lead_account),
        group_id,
        permissions_by_level.clone()
    )
        verify {
            let group = Pallet::<T>::curator_group_by_id(group_id);
            assert_eq!(group.get_permissions_by_level(), &permissions_by_level);
            assert_last_event::<T>(Event::<T>::CuratorGroupPermissionsUpdated(
                group_id,
                permissions_by_level
            ).into());
        }

    set_curator_group_status {
        let (_, lead_account) = insert_content_leader::<T>();
        let group_id = setup_worst_case_curator_group_with_curators::<T>(
            max_curators_per_group::<T>()
        )?;
        let group = Pallet::<T>::curator_group_by_id(group_id);
        assert!(group.is_active());
    }: _ (
        RawOrigin::Signed(lead_account),
        group_id,
        false
    )
        verify {
            let group = Pallet::<T>::curator_group_by_id(group_id);
            assert!(!group.is_active());
            assert_last_event::<T>(Event::<T>::CuratorGroupStatusSet(group_id, false).into());
        }

    add_curator_to_group {
        let (_, lead_account) = insert_content_leader::<T>();
        let permissions = worst_case_channel_agent_permissions();
        let group_id = setup_worst_case_curator_group_with_curators::<T>(
            max_curators_per_group::<T>() - 1
        )?;
        let (curator_id, _) = insert_curator::<T>(
            CURATOR_IDS[max_curators_per_group::<T>() as usize - 1]
        );
        let group = Pallet::<T>::curator_group_by_id(group_id);
        assert_eq!(group.get_curators().get(&curator_id), None);
    }: _ (
        RawOrigin::Signed(lead_account),
        group_id,
        curator_id,
        permissions.clone()
    )
        verify {
            let group = Pallet::<T>::curator_group_by_id(group_id);
            assert_eq!(group.get_curators().get(&curator_id), Some(&permissions));
            assert_last_event::<T>(Event::<T>::CuratorAdded(group_id, curator_id, permissions).into());
        }

    remove_curator_from_group {
        let (_, lead_account) = insert_content_leader::<T>();
        let group_id = setup_worst_case_curator_group_with_curators::<T>(
            max_curators_per_group::<T>()
        )?;
        let group = Pallet::<T>::curator_group_by_id(group_id);
        let curator_id = *group.get_curators().keys().next().unwrap();
    }: _ (
        RawOrigin::Signed(lead_account),
        group_id,
        curator_id
    )
        verify {
            let group = Pallet::<T>::curator_group_by_id(group_id);
            assert!(group.get_curators().get(&curator_id).is_none());
            assert_last_event::<T>(Event::<T>::CuratorRemoved(group_id, curator_id).into());
        }

    /*
    ===============================================================================================
    ============================================ VIDEOS ===========================================
    ===============================================================================================
    */
    create_video_without_nft {
        let a in 1..T::MaxNumberOfAssetsPerVideo::get();
        let b in
            (T::StorageBucketsPerBagValueConstraint::get().min as u32)
            ..(T::StorageBucketsPerBagValueConstraint::get().max() as u32);
        let c in 1..MAX_BYTES_METADATA;

        let (curator_account_id, actor, channel_id, params) = prepare_worst_case_scenario_video_creation_parameters::<T>(
            Some(a),
            b,
            None,
            c
        )?;
        let expected_video_id = Pallet::<T>::next_video_id();
        let expected_asset_ids: BTreeSet<T::DataObjectId> = (
            T::MaxNumberOfAssetsPerChannel::get().saturated_into()
            ..(T::MaxNumberOfAssetsPerChannel::get()+a).saturated_into()
        ).collect();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoCreation]
        );
    }: create_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        channel_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(expected_video_id);
        assert_eq!(video.in_channel, channel_id);
        assert_eq!(video.nft_status, None);
        assert_eq!(BTreeSet::from(video.data_objects), expected_asset_ids);
        assert_eq!(video.video_state_bloat_bond, Pallet::<T>::video_state_bloat_bond_value());
        assert_last_event::<T>(Event::<T>::VideoCreated(
            actor,
            channel_id,
            expected_video_id,
            params,
            expected_asset_ids
        ).into());
    }

    // Worst case scenario: initial state - EnglishAuction
    create_video_with_nft {
        let a in 1..T::MaxNumberOfAssetsPerVideo::get();
        let b in
            (T::StorageBucketsPerBagValueConstraint::get().min as u32)
            ..(T::StorageBucketsPerBagValueConstraint::get().max() as u32);
        let c in 2..MAX_AUCTION_WHITELIST_LENGTH;
        let d in 1..MAX_BYTES_METADATA;

        let (curator_account_id, actor, channel_id, params) = prepare_worst_case_scenario_video_creation_parameters::<T>(
            Some(a),
            b,
            Some(c),
            d
        )?;
        let expected_video_id = Pallet::<T>::next_video_id();
        let expected_asset_ids: BTreeSet<T::DataObjectId> = (
            T::MaxNumberOfAssetsPerChannel::get().saturated_into()
            ..(T::MaxNumberOfAssetsPerChannel::get()+a).saturated_into()
        ).collect();
        let expected_auction_start_block = frame_system::Pallet::<T>::block_number() + T::BlockNumber::one();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoCreation, PausableChannelFeature::VideoNftIssuance]
        );
    }: create_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        channel_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(expected_video_id);
        let nft_params = params.auto_issue_nft.as_ref().unwrap();
        assert_eq!(video.in_channel, channel_id);
        assert_eq!(video.nft_status.as_ref().unwrap().owner, NftOwner::<T::MemberId>::Member(T::MemberId::zero()));
        assert_eq!(video.nft_status.as_ref().unwrap().creator_royalty, nft_params.royalty);
        match &video.nft_status.as_ref().unwrap().transactional_status {
            TransactionalStatus::<T>::EnglishAuction(params) => {
                assert_eq!(params.whitelist.len(), c as usize);
                assert!(params.buy_now_price.is_some());
                assert_eq!(params.start, expected_auction_start_block)
            },
            _ => panic!("Unexpected video nft transactional status")
        }

        assert_eq!(BTreeSet::from(video.data_objects), expected_asset_ids);
        assert_eq!(video.video_state_bloat_bond, Pallet::<T>::video_state_bloat_bond_value());
        assert_last_event::<T>(Event::<T>::VideoCreated(
            actor,
            channel_id,
            expected_video_id,
            params,
            expected_asset_ids
        ).into());
    }

    update_video_without_assets_without_nft {
        let a in 1..MAX_BYTES_METADATA;
        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(
            Some(T::MaxNumberOfAssetsPerVideo::get()),
            T::StorageBucketsPerBagValueConstraint::get().max() as u32,
        )?;
        let params = VideoUpdateParameters::<T> {
            assets_to_upload: None,
            assets_to_remove: BTreeSet::new(),
            auto_issue_nft: None,
            expected_data_object_state_bloat_bond:
                storage::Pallet::<T>::data_object_state_bloat_bond_value(),
            new_meta: Some(vec![0xff].repeat(a as usize)),
            storage_buckets_num_witness: None
        };
        let existing_asset_ids: BTreeSet<T::DataObjectId> = (
            T::MaxNumberOfAssetsPerChannel::get().saturated_into()..
            (T::MaxNumberOfAssetsPerChannel::get() + T::MaxNumberOfAssetsPerVideo::get())
                .saturated_into()
        ).collect();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoUpdate]
        );
    }: update_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(video_id);
        assert_eq!(BTreeSet::from(video.data_objects), existing_asset_ids);
        assert!(video.nft_status.is_none());
        assert_last_event::<T>(Event::<T>::VideoUpdated(
            actor,
            video_id,
            params,
            BTreeSet::new()
        ).into());
    }

    update_video_with_assets_without_nft {
        let a in 1..T::MaxNumberOfAssetsPerVideo::get();
        let b in 1..T::MaxNumberOfAssetsPerVideo::get();
        let c in
            (T::StorageBucketsPerBagValueConstraint::get().min as u32)
            ..(T::StorageBucketsPerBagValueConstraint::get().max() as u32);
        let d in 1..MAX_BYTES_METADATA;

        // As many assets as possible, but leaving room for "a" additional assets,
        // provided that "b" assets will be removed
        let num_preexisting_assets = min(
            T::MaxNumberOfAssetsPerVideo::get() - a + b,
            T::MaxNumberOfAssetsPerVideo::get()
        );
        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(Some(num_preexisting_assets), c)?;

        let max_channel_assets: T::DataObjectId =
            T::MaxNumberOfAssetsPerChannel::get().saturated_into();
        let max_video_assets: T::DataObjectId =
            T::MaxNumberOfAssetsPerVideo::get().saturated_into();
        let assets_to_upload = worst_case_scenario_assets::<T>(a);
        let assets_to_remove: BTreeSet<T::DataObjectId> = (
            max_channel_assets
            ..max_channel_assets + b.saturated_into()
        ).collect();
        let params = VideoUpdateParameters::<T> {
            assets_to_upload: Some(assets_to_upload),
            assets_to_remove,
            auto_issue_nft: None,
            expected_data_object_state_bloat_bond:
                storage::Pallet::<T>::data_object_state_bloat_bond_value(),
            new_meta: Some(vec![0xff].repeat(d as usize)),
            storage_buckets_num_witness:
                Some(storage_buckets_num_witness::<T>(channel_id)).transpose()?
        };
        let expected_asset_ids: BTreeSet<T::DataObjectId> = (
            max_channel_assets + num_preexisting_assets.saturated_into()..
            max_channel_assets + num_preexisting_assets.saturated_into() + a.saturated_into()
        ).collect();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoUpdate]
        );
    }: update_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(video_id);
        assert!(video.nft_status.is_none());
        assert_eq!(BTreeSet::from(video.data_objects), expected_asset_ids);
        assert_last_event::<T>(Event::<T>::VideoUpdated(
            actor,
            video_id,
            params,
            expected_asset_ids
        ).into());
    }

    update_video_without_assets_with_nft {
        let a in 2..MAX_AUCTION_WHITELIST_LENGTH;
        let b in 1..MAX_BYTES_METADATA;

        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(
            Some(T::MaxNumberOfAssetsPerVideo::get()),
            T::StorageBucketsPerBagValueConstraint::get().max() as u32,
        )?;
        let params = VideoUpdateParameters::<T> {
            assets_to_upload: None,
            assets_to_remove: BTreeSet::new(),
            auto_issue_nft: Some(worst_case_scenario_video_nft_issuance_params::<T>(a)),
            expected_data_object_state_bloat_bond:
                storage::Pallet::<T>::data_object_state_bloat_bond_value(),
            new_meta: Some(vec![0xff].repeat(b as usize)),
            storage_buckets_num_witness: None
        };
        let existing_asset_ids: BTreeSet<T::DataObjectId> = (
            T::MaxNumberOfAssetsPerChannel::get().saturated_into()..
            (T::MaxNumberOfAssetsPerChannel::get() + T::MaxNumberOfAssetsPerVideo::get())
                .saturated_into()
        ).collect();
        let expected_auction_start_block = frame_system::Pallet::<T>::block_number() + T::BlockNumber::one();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoUpdate, PausableChannelFeature::VideoNftIssuance]
        );
    }: update_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(video_id);
        assert_eq!(BTreeSet::from(video.data_objects), existing_asset_ids);
        let nft_params = params.auto_issue_nft.as_ref().unwrap();
        assert_eq!(video.nft_status.as_ref().unwrap().owner, NftOwner::<T::MemberId>::Member(T::MemberId::zero()));
        assert_eq!(video.nft_status.as_ref().unwrap().creator_royalty, nft_params.royalty);
        match &video.nft_status.as_ref().unwrap().transactional_status {
            TransactionalStatus::<T>::EnglishAuction(params) => {
                assert_eq!(params.whitelist.len(), a as usize);
                assert!(params.buy_now_price.is_some());
                assert_eq!(params.start, expected_auction_start_block)
            },
            _ => panic!("Unexpected video nft transactional status")
        }
        assert_last_event::<T>(Event::<T>::VideoUpdated(
            actor,
            video_id,
            params,
            BTreeSet::new()
        ).into());
    }

    update_video_with_assets_with_nft {
        let a in 1..T::MaxNumberOfAssetsPerVideo::get();
        let b in 1..T::MaxNumberOfAssetsPerVideo::get();
        let c in
            (T::StorageBucketsPerBagValueConstraint::get().min as u32)
            ..(T::StorageBucketsPerBagValueConstraint::get().max() as u32);
        let d in 2..MAX_AUCTION_WHITELIST_LENGTH;
        let e in 1..MAX_BYTES_METADATA;

        // As many assets as possible, but leaving room for "a" additional assets,
        // provided that "b" assets will be removed
        let num_preexisting_assets = min(
            T::MaxNumberOfAssetsPerVideo::get() - a + b,
            T::MaxNumberOfAssetsPerVideo::get()
        );
        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(Some(num_preexisting_assets), c)?;

        let max_channel_assets: T::DataObjectId =
            T::MaxNumberOfAssetsPerChannel::get().saturated_into();
        let max_video_assets: T::DataObjectId =
            T::MaxNumberOfAssetsPerVideo::get().saturated_into();
        let assets_to_upload = worst_case_scenario_assets::<T>(a);
        let assets_to_remove: BTreeSet<T::DataObjectId> = (
            max_channel_assets
            ..max_channel_assets + b.saturated_into()
        ).collect();
        let params = VideoUpdateParameters::<T> {
            assets_to_upload: Some(assets_to_upload),
            assets_to_remove,
            auto_issue_nft: Some(worst_case_scenario_video_nft_issuance_params::<T>(d)),
            expected_data_object_state_bloat_bond:
                storage::Pallet::<T>::data_object_state_bloat_bond_value(),
            new_meta: Some(vec![0xff].repeat(e as usize)),
            storage_buckets_num_witness:
                Some(storage_buckets_num_witness::<T>(channel_id)).transpose()?
        };
        let expected_asset_ids: BTreeSet<T::DataObjectId> = (
            max_channel_assets + num_preexisting_assets.saturated_into()..
            max_channel_assets + num_preexisting_assets.saturated_into() + a.saturated_into()
        ).collect();
        let expected_auction_start_block = frame_system::Pallet::<T>::block_number() + T::BlockNumber::one();

        set_all_channel_paused_features_except::<T>(
            channel_id,
            vec![PausableChannelFeature::VideoUpdate, PausableChannelFeature::VideoNftIssuance]
        );
    }: update_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        params.clone()
    )
    verify {
        let video = Pallet::<T>::video_by_id(video_id);
        assert_eq!(BTreeSet::from(video.data_objects), expected_asset_ids);
        let nft_params = params.auto_issue_nft.as_ref().unwrap();
        assert_eq!(video.nft_status.as_ref().unwrap().owner, NftOwner::<T::MemberId>::Member(T::MemberId::zero()));
        assert_eq!(video.nft_status.as_ref().unwrap().creator_royalty, nft_params.royalty);
        match &video.nft_status.as_ref().unwrap().transactional_status {
            TransactionalStatus::<T>::EnglishAuction(params) => {
                assert_eq!(params.whitelist.len(), d as usize);
                assert!(params.buy_now_price.is_some());
                assert_eq!(params.start, expected_auction_start_block)
            },
            _ => panic!("Unexpected video nft transactional status")
        }
        assert_last_event::<T>(Event::<T>::VideoUpdated(
            actor,
            video_id,
            params,
            expected_asset_ids
        ).into());
    }

    delete_video_without_assets {
        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(
            None,
            T::StorageBucketsPerBagValueConstraint::get().max() as u32,
        )?;
    }: delete_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        0,
        None
    ) verify {
        assert!(Pallet::<T>::ensure_video_exists(&video_id).is_err());
        assert_last_event::<T>(Event::<T>::VideoDeleted(
            actor,
            video_id
        ).into());
    }

    delete_video_with_assets {
        let a in 1..T::MaxNumberOfAssetsPerVideo::get();
        let b in
            (T::StorageBucketsPerBagValueConstraint::get().min as u32)
            ..(T::StorageBucketsPerBagValueConstraint::get().max() as u32);
        let (
            video_id,
            (curator_account_id, actor, channel_id, _)
        ) = setup_worst_case_scenario_mutable_video::<T>(Some(a), b)?;
        let witness = storage_buckets_num_witness::<T>(channel_id)?;
    }: delete_video (
        RawOrigin::Signed(curator_account_id.clone()),
        actor,
        video_id,
        a as u64,
        Some(witness)
    ) verify {
        assert!(Pallet::<T>::ensure_video_exists(&video_id).is_err());
        assert_last_event::<T>(Event::<T>::VideoDeleted(
            actor,
            video_id
        ).into());
    }

    /*
    ===============================================================================================
    ====================================== CHANNEL TRANSFERS ======================================
    ===============================================================================================
     */

    initialize_channel_transfer {
        let a in 0 .. (T::MaxNumberOfCollaboratorsPerChannel::get() as u32);
        let (_, new_owner_id) = member_funded_account::<T>(0);
        let new_owner = ChannelOwner::Member(new_owner_id);
        let new_collaborators = worst_case_scenario_collaborators::<T>(
            T::MaxNumberOfCollaboratorsPerChannel::get(), // start id
            a // number of collaborators
        );
        let price = <T as balances::Config>::Balance::one();
        let (channel_id, group_id, _, curator_id, curator_account_id) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(false)?;
        let transfer_params = InitTransferParametersOf::<T> {
            new_owner: new_owner.clone(),
            new_collaborators: new_collaborators.clone(),
            price
        };
        let transfer_id = Pallet::<T>::next_transfer_id();
        let actor = ContentActor::Curator(group_id, curator_id);
    }: initialize_channel_transfer (
        RawOrigin::Signed(curator_account_id),
        channel_id,
        actor,
        transfer_params
    ) verify {
        let channel = Pallet::<T>::channel_by_id(channel_id);
        let pending_transfer = PendingTransfer {
            new_owner,
            transfer_params: TransferCommitmentParameters {
                price,
                new_collaborators: new_collaborators.try_into().unwrap(),
                transfer_id
            }
        };
        assert!(
            channel.transfer_status ==
                ChannelTransferStatus::PendingTransfer(pending_transfer.clone())
        );
        assert_last_event::<T>(
            Event::<T>::InitializedChannelTransfer(
                channel_id,
                actor,
                pending_transfer
            ).into()
        );
    }

    cancel_channel_transfer {
        let (channel_id, group_id, lead_account_id, curator_id, curator_account_id) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(true)?;
        let actor = ContentActor::Curator(group_id, curator_id);
    }: _ (
        RawOrigin::Signed(curator_account_id),
        channel_id,
        actor
    ) verify {
        let channel = Pallet::<T>::channel_by_id(channel_id);
        assert!(channel.transfer_status == ChannelTransferStatus::NoActiveTransfer);
        assert_last_event::<T>(
            Event::<T>::CancelChannelTransfer(
                channel_id,
                actor,
            ).into()
        );
    }

    accept_channel_transfer {
        let a in 0 .. (T::MaxNumberOfCollaboratorsPerChannel::get() as u32);

        let (channel_id, group_id, lead_account_id, _, _) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(false)?;
        let new_owner_group_id = clone_curator_group::<T>(group_id)?;
        let new_owner = ChannelOwner::CuratorGroup(new_owner_group_id);
        let new_collaborators = worst_case_scenario_collaborators::<T>(
            T::MaxNumberOfCollaboratorsPerChannel::get(), // start id
            a // number of collaborators
        );
        let price = <T as balances::Config>::Balance::one();
        working_group::Pallet::<T, ContentWorkingGroupInstance>::set_budget(
            RawOrigin::Root.into(),
            price
        )?;
        let transfer_params = InitTransferParametersOf::<T> {
            new_owner: new_owner.clone(),
            new_collaborators: new_collaborators.clone(),
            price
        };
        let transfer_id = Pallet::<T>::next_transfer_id();
        Pallet::<T>::initialize_channel_transfer(
            RawOrigin::Signed(lead_account_id.clone()).into(),
            channel_id,
            ContentActor::Lead,
            transfer_params
        )?;
        let witness = TransferCommitmentWitnessOf::<T> {
            transfer_id,
            price,
            new_collaborators
        };
    }: _ (
        RawOrigin::Signed(lead_account_id),
        channel_id,
        witness.clone()
    ) verify {
        let channel = Pallet::<T>::channel_by_id(channel_id);
        assert!(channel.transfer_status == ChannelTransferStatus::NoActiveTransfer);
        assert_eq!(channel.owner, new_owner);
        assert_last_event::<T>(
            Event::<T>::ChannelTransferAccepted(
                channel_id,
                witness
            ).into()
        );
    }

    // ================================================================================
    // ======================== CHANNEL PAYOUTS & WITHDRAWALS =========================
    // ================================================================================

    // WORST CASE SCENARIO:
    // - DB read cost already maxed out due to `payload` being a struct of `Option`s
    // - `payload` fields `Some(..)` in order to maximize the number of storage mutation performed
    update_channel_payouts {
        let origin = RawOrigin::Root;
        let (account_id, _) = member_funded_account::<T>(1);
        let hash = <<T as frame_system::Config>::Hashing as Hash>::hash(&"test".encode());
        let params = UpdateChannelPayoutsParameters::<T> {
            commitment: Some(hash.clone()),
                        payload: Some(ChannelPayoutsPayloadParameters::<T>{
                uploader_account: account_id,
                object_creation_params: storage::DataObjectCreationParameters {
                    size: 1u64,
                    ipfs_content_id: 1u32.to_be_bytes().as_slice().to_vec(),
                },
                expected_data_object_state_bloat_bond: Storage::<T>::data_object_state_bloat_bond_value(),
                expected_data_size_fee: Storage::<T>::data_object_per_mega_byte_fee(),
            }),
            min_cashout_allowed: Some(<T as balances::Config>::Balance::one()),
            max_cashout_allowed: Some(<T as balances::Config>::Balance::from(1_000_000u32)),
            channel_cashouts_enabled: Some(true),
        };
    }: _ (origin, params)
        verify {
            assert_eq!(
                Pallet::<T>::commitment(),
                hash,
            );
        }

    // WORST CASE SCENARIO:
    // - curator channel belonging to a group with max number curator and max curator permissions
    // - channel has all feature paused except the necessary for the extr. to succeed to maximize permission validation complexity
    withdraw_from_curator_channel_balance {
        let (channel_id, group_id, lead_account_id, _, _) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(false)?;
        let origin= RawOrigin::Signed(lead_account_id.clone());

        set_all_channel_paused_features_except::<T>(channel_id, vec![PausableChannelFeature::ChannelFundsTransfer]);

        let amount = <T as balances::Config>::Balance::from(100u32);
        let _ = Balances::<T>::deposit_creating(
            &ContentTreasury::<T>::account_for_channel(channel_id),
            amount + T::ExistentialDeposit::get(),
        );

        let actor = ContentActor::Lead;
        let origin = RawOrigin::Signed(lead_account_id.clone());
    }: withdraw_from_channel_balance(origin, actor, channel_id, amount)
        verify {
            assert_eq!(
                T::CouncilBudgetManager::get_budget(),
                amount,
            );

            assert_eq!(
                Balances::<T>::usable_balance(ContentTreasury::<T>::account_for_channel(channel_id)),
                T::ExistentialDeposit::get(),
            );
        }

    withdraw_from_member_channel_balance {
        let (channel_id, member_id, member_account_id, lead_account_id) =
            setup_worst_case_scenario_member_channel::<T>(
                T::MaxNumberOfAssetsPerChannel::get(),
                T::StorageBucketsPerBagValueConstraint::get().min as u32,
                T::DistributionBucketsPerBagValueConstraint::get().min as u32,
                false,
            ).unwrap();

        let lead_origin = RawOrigin::Signed(lead_account_id);

        set_all_channel_paused_features_except::<T>(channel_id, vec![PausableChannelFeature::ChannelFundsTransfer]);

        let amount = <T as balances::Config>::Balance::from(100u32);
        let _ = Balances::<T>::deposit_creating(
            &ContentTreasury::<T>::account_for_channel(channel_id),
            amount + T::ExistentialDeposit::get(),
        );

        let origin = RawOrigin::Signed(member_account_id.clone());
        let actor = ContentActor::Member(member_id);
        let owner_balance_pre = Balances::<T>::usable_balance(member_account_id.clone());
    }: withdraw_from_channel_balance(origin, actor, channel_id, amount)
        verify {
            assert_eq!(
                Balances::<T>::usable_balance(member_account_id),
                owner_balance_pre + amount,
            );

            assert_eq!(
                Balances::<T>::usable_balance(ContentTreasury::<T>::account_for_channel(channel_id)),
                T::ExistentialDeposit::get(),
            );
        }

    // Worst case scenario:
    // - curator channel belonging to a group with max number curator and max curator permissions
    // - channel has all feature paused except the necessary for the extr. to succeed to maximize permission validation complexity
    claim_channel_reward {
        let h in 1 .. MAX_MERKLE_PROOF_HASHES;

        let cumulative_reward_claimed: BalanceOf<T> = 100u32.into();
        let payments = create_pull_payments_with_reward::<T>(2u32.pow(h), cumulative_reward_claimed);
        let commitment = generate_merkle_root_helper::<T, _>(&payments).pop().unwrap();
        let proof = build_merkle_path_helper::<T, _>(&payments, 0);
        let (channel_id, group_id, lead_account_id, _, _) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(false)?;
        let origin = RawOrigin::Signed(lead_account_id.clone());

        set_all_channel_paused_features_except::<T>(channel_id, vec![PausableChannelFeature::CreatorCashout]);

        Pallet::<T>::update_channel_payouts(RawOrigin::Root.into(), UpdateChannelPayoutsParameters::<T> {
           commitment: Some(commitment),
            ..Default::default()
        })?;

        let actor = ContentActor::Lead;
        let item = payments[0].clone();
        T::CouncilBudgetManager::set_budget(cumulative_reward_claimed + T::ExistentialDeposit::get());
    }: _ (origin, actor, proof, item)
        verify {
            assert_eq!(
                Pallet::<T>::channel_by_id(channel_id).cumulative_reward_claimed,
                item.cumulative_reward_earned
            );
            assert_eq!(
                T::CouncilBudgetManager::get_budget(),
                T::ExistentialDeposit::get(),
            );
        }

    // Worst case scenario:
    // - curator channel belonging to a group with max number curator and max curator permissions
    // - channel has all feature paused except the necessary for the extr. to succeed to maximize permission validation complexity
    claim_and_withdraw_curator_channel_reward {
        let h in 1 .. MAX_MERKLE_PROOF_HASHES;

        let cumulative_reward_claimed: BalanceOf<T> = 100u32.into();
        let payments = create_pull_payments_with_reward::<T>(2u32.pow(h), cumulative_reward_claimed);
        let commitment = generate_merkle_root_helper::<T, _>(&payments).pop().unwrap();
        let proof = build_merkle_path_helper::<T, _>(&payments, 0);
        let (channel_id, group_id, lead_account_id, _, _) =
            setup_worst_case_scenario_curator_channel_all_max::<T>(false)?;
        let origin = RawOrigin::Signed(lead_account_id.clone());

        set_all_channel_paused_features_except::<T>(channel_id, vec![
                PausableChannelFeature::CreatorCashout,
                PausableChannelFeature::ChannelFundsTransfer,
            ]);

        Pallet::<T>::update_channel_payouts(RawOrigin::Root.into(), UpdateChannelPayoutsParameters::<T> {
           commitment: Some(commitment),
            ..Default::default()
        })?;

        let actor = ContentActor::Lead;
        let item = payments[0].clone();
        T::CouncilBudgetManager::set_budget(cumulative_reward_claimed + T::ExistentialDeposit::get());
    }: claim_and_withdraw_channel_reward(origin, actor, proof, item)
        verify {
            assert_eq!(
                Pallet::<T>::channel_by_id(channel_id).cumulative_reward_claimed,
                item.cumulative_reward_earned
            );
            assert_eq!(
                T::CouncilBudgetManager::get_budget(),
                cumulative_reward_claimed + T::ExistentialDeposit::get(),
            );
        }

    // Worst case scenario:
    // - channel belonging to a member with max number of collaborators and max agent permissions
    // - channel has all feature paused except the necessary for the extr. to succeed to maximize permission validation complexity
    claim_and_withdraw_member_channel_reward {
        let h in 1 .. MAX_MERKLE_PROOF_HASHES;

        let cumulative_reward_claimed: BalanceOf<T> = 100u32.into();
        let payments = create_pull_payments_with_reward::<T>(2u32.pow(h), cumulative_reward_claimed);
        let commitment = generate_merkle_root_helper::<T, _>(&payments).pop().unwrap();
        let proof = build_merkle_path_helper::<T, _>(&payments, 0);
        let (channel_id, member_id, member_account_id, lead_account_id) =
            setup_worst_case_scenario_member_channel_all_max::<T>(false)?;
        let lead_origin = RawOrigin::Signed(lead_account_id.clone());
        let origin = RawOrigin::Signed(member_account_id.clone());

        set_all_channel_paused_features_except::<T>(channel_id, vec![
                PausableChannelFeature::CreatorCashout,
                PausableChannelFeature::ChannelFundsTransfer,
            ]);

        Pallet::<T>::update_channel_payouts(RawOrigin::Root.into(), UpdateChannelPayoutsParameters::<T> {
           commitment: Some(commitment),
            ..Default::default()
        })?;

        let actor = ContentActor::Member(member_id);
        let balances_pre = Balances::<T>::usable_balance(member_account_id.clone());
        let item = payments[0].clone();
        T::CouncilBudgetManager::set_budget(cumulative_reward_claimed + T::ExistentialDeposit::get());
    }: claim_and_withdraw_channel_reward(origin, actor, proof, item)
        verify {
            assert_eq!(
                Pallet::<T>::channel_by_id(channel_id).cumulative_reward_claimed,
                item.cumulative_reward_earned
            );
            assert_eq!(
                Balances::<T>::usable_balance(member_account_id),
                cumulative_reward_claimed + balances_pre,
            );
        }
}

#[cfg(test)]
pub mod tests {
    use crate::tests::mock::{with_default_mock_builder, Content};
    use frame_support::assert_ok;

    #[test]
    fn create_channel() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_create_channel());
        });
    }

    #[test]
    fn channel_update_with_assets() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_channel_update_with_assets());
        });
    }

    #[test]
    fn channel_update_without_assets() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_channel_update_without_assets());
        });
    }

    #[test]
    fn delete_channel() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_delete_channel());
        });
    }

    #[test]
    fn create_curator_group() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_create_curator_group());
        });
    }

    #[test]
    fn update_curator_group_permissions() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_curator_group_permissions());
        });
    }

    #[test]
    fn set_curator_group_status() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_set_curator_group_status());
        })
    }

    #[test]
    fn add_curator_to_group() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_add_curator_to_group());
        })
    }

    #[test]
    fn remove_curator_from_group() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_remove_curator_from_group());
        })
    }

    #[test]
    fn initialize_channel_transfer() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_initialize_channel_transfer());
        });
    }

    #[test]
    fn create_video_without_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_create_video_without_nft());
        });
    }

    #[test]
    fn create_video_with_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_create_video_with_nft());
        });
    }

    #[test]
    fn update_video_without_assets_without_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_video_without_assets_without_nft());
        });
    }

    #[test]
    fn update_video_with_assets_without_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_video_with_assets_without_nft());
        });
    }

    #[test]
    fn update_video_without_assets_with_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_video_without_assets_with_nft());
        });
    }

    #[test]
    fn update_video_with_assets_with_nft() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_video_with_assets_with_nft());
        });
    }

    #[test]
    fn delete_video_without_assets() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_delete_video_without_assets());
        });
    }

    #[test]
    fn delete_video_with_assets() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_delete_video_with_assets());
        });
    }

    #[test]
    fn cancel_channel_transfer() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_cancel_channel_transfer());
        })
    }

    #[test]
    fn accept_channel_transfer() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_accept_channel_transfer());
        })
    }

    #[test]
    fn update_channel_payouts() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_update_channel_payouts());
        })
    }

    #[test]
    fn withdraw_from_member_channel_balance() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_withdraw_from_member_channel_balance());
        })
    }

    #[test]
    fn withdraw_from_curator_channel_balance() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_withdraw_from_curator_channel_balance());
        })
    }

    #[test]
    fn claim_channel_reward() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_claim_channel_reward());
        })
    }

    #[test]
    fn claim_channel_and_withdraw_member_channel_reward() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_claim_and_withdraw_member_channel_reward());
        })
    }

    #[test]
    fn claim_channel_and_withdraw_curator_channel_reward() {
        with_default_mock_builder(|| {
            assert_ok!(Content::test_benchmark_claim_and_withdraw_curator_channel_reward());
        })
    }
}
