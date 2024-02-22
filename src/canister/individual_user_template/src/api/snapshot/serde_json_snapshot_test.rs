#[cfg(test)]
mod test {

    use std::{
        collections::{BTreeMap, BTreeSet, HashMap, HashSet},
        time::SystemTime,
    };

    use candid::Principal;
    use ic_cdk::api::management_canister::main::CanisterId;
    use shared_utils::{
        canister_specific::individual_user_template::types::{
            configuration::IndividualUserConfiguration,
            follow::{FollowData, FollowEntryDetail, FollowList},
            hot_or_not::{
                AggregateStats, BetDetails, BetDirection, BetOutcomeForBetMaker, BetPayout,
                GlobalBetId, GlobalRoomId, PlacedBetDetail, RoomBetPossibleOutcomes, RoomDetailsV1,
                SlotDetailsV1, SlotId, StablePrincipal,
            },
            post::{FeedScore, PostViewStatistics},
            profile::{UserProfile, UserProfileGlobalStats},
            token::TokenBalance,
        },
        common::types::{
            app_primitive_type::PostId,
            known_principal::KnownPrincipalType,
            top_posts::{
                post_score_index::PostScoreIndex,
                post_score_index_item::{PostScoreIndexItem, PostStatus},
            },
            utility_token::token_event::{MintEvent, TokenEvent},
            version_details::VersionDetails,
        },
    };
    use test_utils::setup::test_constants::get_mock_user_alice_canister_id;

    use crate::{
        api::{
            snapshot::{
                CanisterDataForSnapshot, FollowDataForSnapshot, FollowListForSnapshot,
                HotOrNotDetailsForSnapshot, PostForSnapshot, PostScoreIndexForSnapshot,
                TokenBalanceForSnapshot,
            },
            well_known_principal::get_well_known_principal_value,
        },
        data_model::CanisterData,
    };

    #[test]
    fn test_serde_json_snapshot() {
        let mut created_posts = BTreeMap::<u64, PostForSnapshot>::new();

        let temp_principal = get_mock_user_alice_canister_id();

        let post1 = PostForSnapshot {
            id: 1,
            description: "todo".to_string(),
            hashtags: vec!["dfasf".to_string()],
            video_uid: "21".to_string(),
            status: PostStatus::ReadyToView,
            created_at: SystemTime::now(),
            likes: HashSet::from([temp_principal]),
            share_count: 12,
            view_stats: PostViewStatistics {
                total_view_count: 12,
                threshold_view_count: 12,
                average_watch_percentage: 21,
            },
            home_feed_score: FeedScore {
                current_score: 1200,
                last_synchronized_score: 1200,
                last_synchronized_at: SystemTime::now(),
            },
            creator_consent_for_inclusion_in_hot_or_not: true,
            hot_or_not_details: Some(HotOrNotDetailsForSnapshot {
                hot_or_not_feed_score: FeedScore {
                    current_score: 1200,
                    last_synchronized_score: 1200,
                    last_synchronized_at: SystemTime::now(),
                },
                aggregate_stats: AggregateStats {
                    total_number_of_hot_bets: 31,
                    total_number_of_not_bets: 30,
                    total_amount_bet: 12000,
                },
            }),
            is_nsfw: false,
        };
        created_posts.insert(1, post1);

        let mut room_details_map: BTreeMap<GlobalRoomId, RoomDetailsV1> = BTreeMap::new();
        let global_room_id = GlobalRoomId(1, 1, 1);
        room_details_map.insert(
            global_room_id,
            RoomDetailsV1 {
                bet_outcome: RoomBetPossibleOutcomes::HotWon,
                room_bets_total_pot: 12000,
                total_hot_bets: 31,
                total_not_bets: 30,
            },
        );

        let mut bet_details_map: BTreeMap<GlobalBetId, BetDetails> = BTreeMap::new();
        let global_bet_id = GlobalBetId(global_room_id, StablePrincipal(temp_principal));
        bet_details_map.insert(
            global_bet_id,
            BetDetails {
                amount: 100,
                bet_direction: BetDirection::Hot,
                payout: BetPayout::Calculated(1000),
                bet_maker_canister_id: temp_principal,
            },
        );

        let mut post_principal_map: BTreeMap<(PostId, StablePrincipal), ()> = BTreeMap::new();
        post_principal_map.insert((1, StablePrincipal(temp_principal)), ());

        let mut slot_details_map: BTreeMap<(PostId, SlotId), SlotDetailsV1> = BTreeMap::new();
        slot_details_map.insert((1, 1), SlotDetailsV1 { active_room_id: 2 });

        let mut all_hot_or_not_bets_placed: BTreeMap<(CanisterId, PostId), PlacedBetDetail> =
            BTreeMap::new();
        all_hot_or_not_bets_placed.insert(
            (temp_principal, 1),
            PlacedBetDetail {
                canister_id: temp_principal,
                post_id: 1,
                slot_id: 1,
                room_id: 1,
                amount_bet: 100,
                bet_direction: BetDirection::Hot,
                bet_placed_at: SystemTime::now(),
                outcome_received: BetOutcomeForBetMaker::Won(10),
            },
        );

        let mut follow_sorted_index = BTreeMap::<u64, FollowEntryDetail>::new();
        follow_sorted_index.insert(
            1,
            FollowEntryDetail {
                principal_id: temp_principal,
                canister_id: temp_principal,
            },
        );
        let mut follow_members = HashMap::<FollowEntryDetail, u64>::new();
        follow_members.insert(
            FollowEntryDetail {
                principal_id: temp_principal,
                canister_id: temp_principal,
            },
            1,
        );

        let mut known_principal_ids = HashMap::<KnownPrincipalType, Principal>::new();
        known_principal_ids.insert(KnownPrincipalType::CanisterIdPostCache, temp_principal);

        let mut utility_history = BTreeMap::<u64, TokenEvent>::new();
        utility_history.insert(
            1,
            TokenEvent::Mint {
                amount: 10,
                details: MintEvent::NewUserSignup {
                    new_user_principal_id: temp_principal,
                },
                timestamp: SystemTime::now(),
            },
        );

        let mut items_sorted_by_score = BTreeMap::<u64, Vec<PostScoreIndexItem>>::new();
        items_sorted_by_score.insert(
            1,
            vec![PostScoreIndexItem {
                score: 100,
                post_id: 1,
                publisher_canister_id: temp_principal,
            }],
        );

        let mut item_prescence_index = HashMap::<(Principal, u64), u64>::new();
        item_prescence_index.insert((temp_principal, 1), 1);

        let mut principal_list = BTreeSet::<Principal>::new();
        principal_list.insert(temp_principal);

        let canister_data_snapshot = CanisterDataForSnapshot {
            all_created_posts: created_posts,
            room_details_map: room_details_map,
            bet_details_map: bet_details_map,
            post_principal_map: post_principal_map,
            slot_details_map: slot_details_map,
            all_hot_or_not_bets_placed: all_hot_or_not_bets_placed,
            configuration: IndividualUserConfiguration {
                url_to_send_canister_metrics_to: Some("dsfsd".to_string()),
            },
            follow_data: FollowDataForSnapshot {
                follower: FollowListForSnapshot {
                    sorted_index: follow_sorted_index.clone(),
                    members: follow_members.clone(),
                },
                following: FollowListForSnapshot {
                    sorted_index: follow_sorted_index,
                    members: follow_members,
                },
            },
            known_principal_ids: known_principal_ids,
            my_token_balance: TokenBalanceForSnapshot {
                utility_token_balance: 100,
                utility_token_transaction_history: utility_history,
                lifetime_earnings: 1200,
            },
            posts_index_sorted_by_home_feed_score: PostScoreIndexForSnapshot {
                items_sorted_by_score: items_sorted_by_score.clone(),
                item_presence_index: item_prescence_index.clone(),
            },
            posts_index_sorted_by_hot_or_not_feed_score: PostScoreIndexForSnapshot {
                items_sorted_by_score: items_sorted_by_score.clone(),
                item_presence_index: item_prescence_index.clone(),
            },
            principals_i_follow: principal_list.clone(),
            principals_that_follow_me: principal_list,
            profile: UserProfile {
                display_name: Some("dadfk".to_string()),
                unique_user_name: Some("dadfk".to_string()),
                principal_id: Some(temp_principal),
                profile_picture_url: Some("dadfk".to_string()),
                profile_stats: UserProfileGlobalStats {
                    hot_bets_received: 100,
                    not_bets_received: 100,
                },
            },
            version_details: VersionDetails {
                version_number: 1,
                version: "1.0.0".to_string(),
            },
        };

        let serde_str = serde_json::to_string(&canister_data_snapshot);
        assert_eq!(serde_str.is_ok(), true);

        let canister_data_snapshot: CanisterDataForSnapshot =
            serde_json::from_str(serde_str.unwrap().as_str()).unwrap();

        let canister_data = CanisterData::from(canister_data_snapshot);

        // println!("canister_data: {:?}", canister_data.all_created_posts);
    }
}
