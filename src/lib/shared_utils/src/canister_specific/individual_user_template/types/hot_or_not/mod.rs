use std::{collections::BTreeMap, time::SystemTime};

use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;

use super::{
    error::BetOnCurrentlyViewingPostError,
    post::{FeedScore, Post},
};

#[derive(CandidType, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum BettingStatus {
    BettingOpen {
        started_at: SystemTime,
        number_of_participants: u8,
        ongoing_slot: u8,
        ongoing_room: u64,
        has_this_user_participated_in_this_post: Option<bool>,
    },
    BettingClosed,
}

pub const MAXIMUM_NUMBER_OF_SLOTS: u8 = 48;
pub const DURATION_OF_EACH_SLOT_IN_SECONDS: u64 = 60 * 60;
pub const TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS: u64 =
    MAXIMUM_NUMBER_OF_SLOTS as u64 * DURATION_OF_EACH_SLOT_IN_SECONDS;

#[derive(CandidType)]
pub enum UserStatusForSpecificHotOrNotPost {
    NotParticipatedYet,
    AwaitingResult(BetDetail),
    ResultAnnounced(BetResult),
}

#[derive(CandidType)]
pub enum BetResult {
    Won(u64),
    Lost,
    Draw,
}

#[derive(CandidType)]
pub struct BetDetail {
    amount: u64,
    bet_direction: BetDirection,
    bet_made_at: SystemTime,
}

#[derive(CandidType, Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
pub enum BetDirection {
    Hot,
    Not,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct HotOrNotBetId {
    pub canister_id: Principal,
    pub post_id: u64,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct HotOrNotDetails {
    pub hot_or_not_feed_score: FeedScore,
    pub aggregate_stats: AggregateStats,
    pub slot_history: BTreeMap<SlotId, SlotDetails>,
}

#[derive(CandidType, Clone, Deserialize, Debug, Serialize, Default)]
pub struct AggregateStats {
    pub total_number_of_hot_bets: u64,
    pub total_number_of_not_bets: u64,
    pub total_amount_bet: u64,
}

pub type SlotId = u8;

#[derive(CandidType, Clone, Deserialize, Default, Debug, Serialize)]
pub struct SlotDetails {
    pub room_details: BTreeMap<RoomId, RoomDetails>,
}

pub type RoomId = u64;

#[derive(CandidType, Clone, Deserialize, Default, Debug, Serialize)]
pub struct RoomDetails {
    pub bets_made: BTreeMap<BetMaker, BetDetails>,
    pub bet_outcome: RoomBetPossibleOutcomes,
    pub room_bets_total_pot: u64,
}

pub type BetMaker = Principal;

#[derive(CandidType, Clone, Deserialize, Debug, Serialize)]
pub struct BetDetails {
    pub amount: u64,
    pub bet_direction: BetDirection,
}

#[derive(CandidType, Clone, Default, Debug, Deserialize, Serialize)]
pub enum RoomBetPossibleOutcomes {
    #[default]
    BetOngoing,
    HotWon,
    NotWon,
    Draw,
}

impl Post {
    pub fn get_hot_or_not_betting_status_for_this_post(
        &self,
        current_time_when_request_being_made: &SystemTime,
        bet_maker_principal_id: &Principal,
    ) -> BettingStatus {
        let betting_status =
            match current_time_when_request_being_made
                .duration_since(self.created_at)
                .unwrap()
                .as_secs()
            {
                // * contest is still ongoing
                0..=TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS => {
                    let started_at = self.created_at;
                    let numerator = current_time_when_request_being_made
                        .duration_since(started_at)
                        .unwrap()
                        .as_secs();

                    let denominator = DURATION_OF_EACH_SLOT_IN_SECONDS;
                    let currently_ongoing_slot = ((numerator / denominator) + 1) as u8;

                    let temp_hot_or_not_default = &HotOrNotDetails::default();
                    let temp_slot_details_default = &SlotDetails::default();
                    let room_details = &self
                        .hot_or_not_details
                        .as_ref()
                        .unwrap_or(temp_hot_or_not_default)
                        .slot_history
                        .get(&currently_ongoing_slot)
                        .unwrap_or(temp_slot_details_default)
                        .room_details;

                    let temp_room_details_default = &RoomDetails::default();
                    let currently_active_room = room_details
                        .last_key_value()
                        .unwrap_or((&1, temp_room_details_default));
                    let number_of_participants = currently_active_room.1.bets_made.len() as u8;
                    BettingStatus::BettingOpen {
                        started_at,
                        number_of_participants,
                        ongoing_slot: currently_ongoing_slot,
                        ongoing_room: *currently_active_room.0 as u64,
                        has_this_user_participated_in_this_post: if *bet_maker_principal_id
                            == Principal::anonymous()
                        {
                            None
                        } else {
                            Some(self.has_this_principal_already_bet_on_this_post(
                                bet_maker_principal_id,
                            ))
                        },
                    }
                }
                // * contest is over
                _ => BettingStatus::BettingClosed,
            };

        betting_status
    }

    pub fn has_this_principal_already_bet_on_this_post(
        &self,
        principal_making_bet: &Principal,
    ) -> bool {
        self.hot_or_not_details
            .as_ref()
            .unwrap()
            .slot_history
            .iter()
            .map(|(_, slot_details)| slot_details.room_details.iter())
            .flatten()
            .map(|(_, room_details)| room_details.bets_made.iter())
            .flatten()
            .any(|(principal, _)| principal == principal_making_bet)
    }

    pub fn place_hot_or_not_bet(
        &mut self,
        bet_maker_principal_id: &Principal,
        bet_amount: u64,
        bet_direction: &BetDirection,
        current_time_when_request_being_made: &SystemTime,
    ) -> Result<BettingStatus, BetOnCurrentlyViewingPostError> {
        if *bet_maker_principal_id == Principal::anonymous() {
            return Err(BetOnCurrentlyViewingPostError::UserNotLoggedIn);
        }

        let betting_status = self.get_hot_or_not_betting_status_for_this_post(
            current_time_when_request_being_made,
            bet_maker_principal_id,
        );

        match betting_status {
            BettingStatus::BettingClosed => Err(BetOnCurrentlyViewingPostError::BettingClosed),
            BettingStatus::BettingOpen {
                ongoing_slot,
                ongoing_room,
                has_this_user_participated_in_this_post,
                ..
            } => {
                if has_this_user_participated_in_this_post.unwrap() {
                    return Err(BetOnCurrentlyViewingPostError::UserAlreadyParticipatedInThisPost);
                }

                let mut hot_or_not_details = self
                    .hot_or_not_details
                    .take()
                    .unwrap_or(HotOrNotDetails::default());
                let slot_history = hot_or_not_details
                    .slot_history
                    .entry(ongoing_slot)
                    .or_default();
                let room_details = slot_history.room_details.entry(ongoing_room).or_default();
                let bets_made_currently = &mut room_details.bets_made;

                // * Update bets_made currently
                if bets_made_currently.len() < 100 {
                    bets_made_currently.insert(
                        bet_maker_principal_id.clone(),
                        BetDetails {
                            amount: bet_amount,
                            bet_direction: bet_direction.clone(),
                        },
                    );
                    room_details.room_bets_total_pot += bet_amount;
                } else {
                    let new_room_number = ongoing_room + 1;
                    let mut bets_made = BTreeMap::default();
                    bets_made.insert(
                        bet_maker_principal_id.clone(),
                        BetDetails {
                            amount: bet_amount,
                            bet_direction: bet_direction.clone(),
                        },
                    );
                    slot_history.room_details.insert(
                        new_room_number,
                        RoomDetails {
                            bets_made,
                            room_bets_total_pot: bet_amount,
                            ..Default::default()
                        },
                    );
                }

                // * Update aggregate stats
                hot_or_not_details.aggregate_stats.total_amount_bet += bet_amount;
                match bet_direction {
                    BetDirection::Hot => {
                        hot_or_not_details.aggregate_stats.total_number_of_hot_bets += 1;
                    }
                    BetDirection::Not => {
                        hot_or_not_details.aggregate_stats.total_number_of_not_bets += 1;
                    }
                }

                self.hot_or_not_details = Some(hot_or_not_details);

                let slot_history = &self.hot_or_not_details.as_ref().unwrap().slot_history;
                let started_at = self.created_at;
                let number_of_participants = slot_history
                    .last_key_value()
                    .unwrap()
                    .1
                    .room_details
                    .last_key_value()
                    .unwrap()
                    .1
                    .bets_made
                    .len() as u8;
                let ongoing_slot = *slot_history.last_key_value().unwrap().0;
                let ongoing_room = *slot_history
                    .last_key_value()
                    .unwrap()
                    .1
                    .room_details
                    .last_key_value()
                    .unwrap()
                    .0;
                Ok(BettingStatus::BettingOpen {
                    started_at,
                    number_of_participants,
                    ongoing_slot,
                    ongoing_room,
                    has_this_user_participated_in_this_post: Some(true),
                })
            }
        }
    }

    // TODO: enable
    // pub fn tabulate_hot_or_not_outcome_for_slot(&mut self, slot_id: &u8) {
    //     let slot_to_tabulate = self
    //         .hot_or_not_details
    //         .as_mut()
    //         .unwrap()
    //         .slot_history
    //         .get_mut(slot_id)
    //         .unwrap();

    //     slot_to_tabulate
    //         .room_details
    //         .iter()
    //         .for_each(|(room_id, room_detail)| {
    //             let hot_vote_count_in_room =
    //                 room_detail
    //                     .bets_made
    //                     .iter()
    //                     .fold(0, |acc, (_, bet_details)| match bet_details.bet_direction {
    //                         BetDirection::Hot => acc + 1,
    //                         BetDirection::Not => acc,
    //                     });
    //         })
    // }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use test_utils::setup::test_constants::get_mock_user_alice_principal_id;

    use crate::canister_specific::individual_user_template::types::post::PostDetailsFromFrontend;

    use super::*;

    #[test]
    fn test_get_hot_or_not_betting_status_for_this_post() {
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
        );

        assert_eq!(result, BettingStatus::BettingClosed);

        let current_time = SystemTime::now();

        let result = post
            .get_hot_or_not_betting_status_for_this_post(&current_time, &Principal::anonymous());

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 1,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: None,
            }
        );

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::anonymous(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 3,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: None,
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: 3,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        (100..200).for_each(|num| {
            let result = post.place_hot_or_not_bet(
                &Principal::from_slice(&[num]),
                100,
                &BetDirection::Hot,
                &current_time
                    .checked_add(Duration::from_secs(
                        DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                    ))
                    .unwrap(),
            );

            assert!(result.is_ok());
        });

        let result = post.place_hot_or_not_bet(
            &Principal::from_slice(&[200]),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_ok());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &Principal::from_slice(&[100]),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 2,
                ongoing_slot: 3,
                ongoing_room: 2,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 2 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 2,
                ongoing_slot: 3,
                ongoing_room: 2,
                has_this_user_participated_in_this_post: Some(true),
            }
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
        );

        assert!(result.is_err());

        let result = post.get_hot_or_not_betting_status_for_this_post(
            &current_time
                .checked_add(Duration::from_secs(
                    DURATION_OF_EACH_SLOT_IN_SECONDS * 4 + 1,
                ))
                .unwrap(),
            &get_mock_user_alice_principal_id(),
        );

        assert_eq!(
            result,
            BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 0,
                ongoing_slot: 5,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true),
            }
        );
    }

    #[test]
    fn test_has_this_principal_already_bet_on_this_post() {
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        let result =
            post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

        assert_eq!(result, false);

        post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        )
        .ok();

        let result =
            post.has_this_principal_already_bet_on_this_post(&get_mock_user_alice_principal_id());

        assert_eq!(result, true);
    }

    #[test]
    fn test_place_hot_or_not_bet() {
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &SystemTime::now(),
        );

        assert!(post.hot_or_not_details.is_some());

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now()
                .checked_add(Duration::from_secs(
                    TOTAL_DURATION_OF_ALL_SLOTS_IN_SECONDS + 1,
                ))
                .unwrap(),
        );

        assert_eq!(result, Err(BetOnCurrentlyViewingPostError::BettingClosed));

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        );

        assert_eq!(
            result,
            Ok(BettingStatus::BettingOpen {
                started_at: post.created_at,
                number_of_participants: 1,
                ongoing_slot: 1,
                ongoing_room: 1,
                has_this_user_participated_in_this_post: Some(true)
            })
        );
        let hot_or_not_details = post.hot_or_not_details.clone().unwrap();
        assert_eq!(hot_or_not_details.slot_history.len(), 1);
        let room_details = &hot_or_not_details
            .slot_history
            .get(&1)
            .unwrap()
            .room_details;
        assert_eq!(room_details.len(), 1);
        let room = room_details.get(&1).unwrap();
        let bets_made = &room.bets_made;
        assert_eq!(bets_made.len(), 1);
        assert_eq!(
            bets_made
                .get(&get_mock_user_alice_principal_id())
                .unwrap()
                .amount,
            100
        );
        assert_eq!(
            bets_made
                .get(&get_mock_user_alice_principal_id())
                .unwrap()
                .bet_direction,
            BetDirection::Hot
        );
        assert_eq!(room.room_bets_total_pot, 100);
        assert_eq!(hot_or_not_details.aggregate_stats.total_amount_bet, 100);
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_hot_bets,
            1
        );
        assert_eq!(
            hot_or_not_details.aggregate_stats.total_number_of_not_bets,
            0
        );

        let result = post.place_hot_or_not_bet(
            &get_mock_user_alice_principal_id(),
            100,
            &BetDirection::Hot,
            &SystemTime::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_tabulate_hot_or_not_outcome_for_slot_case_1() {
        let post_creation_time = SystemTime::now();
        let mut post = Post::new(
            0,
            &PostDetailsFromFrontend {
                description: "Doggos and puppers".into(),
                hashtags: vec!["doggo".into(), "pupper".into()],
                video_uid: "abcd#1234".into(),
                creator_consent_for_inclusion_in_hot_or_not: true,
            },
            &post_creation_time,
        );

        assert!(post.hot_or_not_details.is_some());

        let data_set: Vec<(u64, BetDirection, u64, u64)> = vec![
            (1, BetDirection::Not, 10, 18),
            (2, BetDirection::Hot, 100, 0),
            (3, BetDirection::Hot, 100, 0),
            (4, BetDirection::Not, 100, 180),
            (5, BetDirection::Hot, 10, 0),
            (6, BetDirection::Not, 100, 180),
            (7, BetDirection::Not, 50, 90),
            (8, BetDirection::Not, 100, 180),
            (9, BetDirection::Hot, 50, 0),
            (10, BetDirection::Not, 50, 90),
            (11, BetDirection::Not, 100, 180),
            (12, BetDirection::Not, 10, 18),
            (13, BetDirection::Hot, 100, 0),
            (14, BetDirection::Not, 10, 18),
            (15, BetDirection::Hot, 50, 0),
            (16, BetDirection::Hot, 10, 0),
            (17, BetDirection::Hot, 10, 0),
            (18, BetDirection::Hot, 100, 0),
            (19, BetDirection::Not, 10, 18),
            (20, BetDirection::Hot, 50, 0),
            (21, BetDirection::Hot, 10, 0),
            (22, BetDirection::Not, 50, 90),
            (23, BetDirection::Not, 50, 90),
            (24, BetDirection::Hot, 100, 0),
            (25, BetDirection::Not, 50, 90),
            (26, BetDirection::Not, 10, 18),
            (27, BetDirection::Not, 10, 18),
            (28, BetDirection::Not, 50, 90),
            (29, BetDirection::Hot, 50, 0),
            (30, BetDirection::Not, 100, 180),
            (31, BetDirection::Not, 50, 90),
            (32, BetDirection::Not, 50, 90),
            (33, BetDirection::Hot, 100, 0),
            (34, BetDirection::Not, 10, 18),
            (35, BetDirection::Not, 10, 18),
            (36, BetDirection::Not, 100, 180),
            (37, BetDirection::Hot, 10, 0),
            (38, BetDirection::Not, 100, 180),
            (39, BetDirection::Not, 50, 90),
            (40, BetDirection::Hot, 100, 0),
            (41, BetDirection::Hot, 50, 0),
            (42, BetDirection::Not, 10, 18),
            (43, BetDirection::Hot, 50, 0),
            (44, BetDirection::Not, 10, 18),
            (45, BetDirection::Not, 10, 18),
            (46, BetDirection::Hot, 100, 0),
            (47, BetDirection::Hot, 50, 0),
            (48, BetDirection::Hot, 50, 0),
            (49, BetDirection::Not, 100, 180),
            (50, BetDirection::Hot, 10, 0),
            (51, BetDirection::Not, 50, 90),
            (52, BetDirection::Hot, 10, 0),
            (53, BetDirection::Not, 50, 90),
            (54, BetDirection::Not, 10, 18),
            (55, BetDirection::Hot, 100, 0),
            (56, BetDirection::Hot, 50, 0),
            (57, BetDirection::Not, 50, 90),
            (58, BetDirection::Not, 10, 18),
            (59, BetDirection::Not, 50, 90),
            (60, BetDirection::Hot, 10, 0),
            (61, BetDirection::Not, 10, 18),
            (62, BetDirection::Not, 50, 90),
            (63, BetDirection::Not, 50, 90),
            (64, BetDirection::Not, 10, 18),
            (65, BetDirection::Not, 10, 18),
            (66, BetDirection::Not, 100, 180),
            (67, BetDirection::Hot, 100, 0),
            (68, BetDirection::Not, 10, 18),
            (69, BetDirection::Not, 10, 18),
            (70, BetDirection::Not, 50, 90),
            (71, BetDirection::Not, 100, 180),
            (72, BetDirection::Not, 10, 18),
            (73, BetDirection::Not, 10, 18),
            (74, BetDirection::Hot, 10, 0),
            (75, BetDirection::Not, 10, 18),
        ];

        data_set
            .iter()
            .for_each(|(user_id, bet_direction, bet_amount, _)| {
                let result = post.place_hot_or_not_bet(
                    &Principal::self_authenticating(&user_id.to_ne_bytes()),
                    *bet_amount,
                    bet_direction,
                    &post_creation_time,
                );
                assert!(result.is_ok());
            });
    }
}
