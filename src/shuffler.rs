use rand::seq::SliceRandom;
use tracing::info;

use crate::game::{Players, Pairs};

#[derive(Debug)]
pub enum ShuffleError {
    // Shuffling two or less people is no fun.
    TooFewPeople,
    // If we get a list with duplicates, shuffling them properly is harder, so I don't deal with that.
    DuplicatesDetected,
    // If there are more exclusions than players, then it may be impossible to properly shuffle.
    // Even in case of 3 players, a list of exclusions for every player still leaves one possible outcome.
    TooManyExclusions,
}
pub fn shuffle_people(people: &Players, avoid_pairs: &Pairs) -> Result<Pairs, ShuffleError> {
    let mut result = vec!();
    if people.len() < 3 {
        return Err(ShuffleError::TooFewPeople);
    }

    if avoid_pairs.len() > people.len() {
        return Err(ShuffleError::TooManyExclusions)
    }

    let mut players = people.clone();
    let count = players.len();
    // Sort and remove all duplicates.
    players.sort();
    players.dedup();
    // Check if anything was removed.
    if count != players.len() {
        return Err(ShuffleError::DuplicatesDetected);
    }

    let mut rng = rand::thread_rng();
    players.shuffle(&mut rng);
    players.push(players[0]);

    for i in 0..players.len() - 1 /* -1, because there's one extra pushed to the end. */ {
        let tuple = (players[i], players[i+1]);
        if avoid_pairs.contains(&tuple) {
            info!("Duplicate detected, shuffling people again.");
            return shuffle_people(people, avoid_pairs);
        }
        result.push(tuple);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rand::{distributions::Slice, Rng};
    use serenity::model::prelude::UserId;

    use crate::{game::{Players, Pairs}};

    use super::{shuffle_people, ShuffleError};

    const MENTION_LENGTH: usize = 21; // looks like this: <@285136304914563075>
    pub const ID_LENGTH: usize = MENTION_LENGTH - 3; // Remove <, @ and > from the above.

    #[derive(Debug)]
    enum TestResult {
        String(String),
        PairValidityError(PairValidityError),
        PairExclusionError(PairExclusionError),
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_three_people() -> Result<(), TestResult> {
        let ids = generate_user_ids(3).into();

        match shuffle_people(&ids, &vec!()) {
            Ok(shuffled) => {
                println!("Players: {:?}.", shuffled);
                match check_pairs_validity(&shuffled, 3) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({error:?})."))),
        }
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_three_people_with_exclusion() -> Result<(), TestResult> {
        let ids: Players = generate_user_ids(3).into();
        let exclusions: Pairs = vec!((ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[0]));

        match shuffle_people(&ids, &exclusions) {
            Ok(shuffled) => {
                println!("Players: {shuffled:?}.");
                println!("Exclusions: {exclusions:?}.");
                match check_pairs_validity(&shuffled, 3) {
                    Ok(_) => {
                        match check_exclusion_validity(shuffled, exclusions) {
                            Ok(_) => Ok(()),
                            Err(error) => Err(TestResult::PairExclusionError(error))
                        }
                    },
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({error:?})."))),
        }
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_hundred_people() -> Result<(), TestResult> {
        let ids = generate_user_ids(100).into();

        match shuffle_people(&ids, &vec!()) {
            Ok(shuffled) => {
                println!("Players: {shuffled:?}.");
                match check_pairs_validity(&shuffled, 100) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({error:?})."))),
        }
    }

    #[test]
    fn test_shuffle_errors_on_no_people() -> Result<(), String> {
        let ids = generate_user_ids(0).into();

        match shuffle_people(&ids, &vec!()) {
            Err(ShuffleError::TooFewPeople) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({shuffled:?}).")),
            Err(error) => Err(format!("A wrong error was returned ({error:?}).")),
        }
    }

    #[test]
    fn test_shuffle_errors_on_one_person() -> Result<(), String> {
        let ids = generate_user_ids(1).into();

        match shuffle_people(&ids, &vec!()) {
            Err(ShuffleError::TooFewPeople) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({shuffled:?}).")),
            Err(error) => Err(format!("A wrong error was returned ({error:?}).")),
        }
    }

    #[test]
    fn test_shuffle_errors_on_two_people() -> Result<(), String> {
        let ids = generate_user_ids(2).into();

        match shuffle_people(&ids, &vec!()) {
            Err(ShuffleError::TooFewPeople) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({shuffled:?}).")),
            Err(error) => Err(format!("A wrong error was returned ({error:?}).")),
        }
    }

    #[test]
    fn test_shuffle_errors_on_duplicate() -> Result<(), String> {
        let mut ids: Players = generate_user_ids(3).into();
        ids.push(ids[0]);

        match shuffle_people(&ids, &vec!()) {
            Err(ShuffleError::DuplicatesDetected) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({shuffled:?}).")),
            Err(error) => Err(format!("A wrong error was returned ({error:?}).")),
        }
    }

    // No 0 to not generate numbers with leading 0, simplifies a lot of things.
    const DIGITS: [char; 9] = ['1','2','3','4','5','6','7','8','9'];
    fn generate_user_ids(count: usize) -> Players {
        let mut ids = vec!();
        let distribution = Slice::new(&DIGITS).unwrap();
        let rng = &mut rand::thread_rng();
        while ids.len() != count {
            let result = rng.sample_iter(&distribution).take(ID_LENGTH).collect::<String>();
            let result = result.parse().unwrap();
            if ids.contains(&result) {
                continue;
            }
            ids.push(result);
        }
        ids
    }

    #[derive(Debug)]
    pub enum PairValidityError {
        PlayerAssignedToThemselves,
        PlayerWasChosenTwice,
        TooFewPairs,
    }
    fn check_pairs_validity(
        pairs: &Pairs, expected_pair_count: usize
    ) -> Result<(), PairValidityError> {
        if pairs.len() != expected_pair_count {
            return Err(PairValidityError::TooFewPairs);
        }

        let mut players = vec!();
        for (player, avatar) in pairs {
            if player == avatar {
                return Err(PairValidityError::PlayerAssignedToThemselves);
            }
            if players.contains(&player) {
                return Err(PairValidityError::PlayerWasChosenTwice);
            }
            players.push(player);
        }
        Ok(())
    }

    #[derive(Debug)]
    pub enum PairExclusionError {
        PlayerWasAssignedToExcludedPlayer,
    }
    fn check_exclusion_validity(pairs: Pairs, exclusions: Pairs) -> Result<(), PairExclusionError> {
        let exclusions: HashMap<UserId, UserId> = exclusions.iter()
            .map(|(k, v)| {(k.clone(), v.clone())}).collect();

        for (player, avatar) in pairs {
            if &avatar == exclusions.get(&player).unwrap() {
                return Err(PairExclusionError::PlayerWasAssignedToExcludedPlayer);
            }
        }
        // let excluded_by_player: HashMap<UserId, UserId> = HashMap::default();
        // for exclusion in exclusions {
        //     excluded_by_player
        // }
        Ok(())
    }
}
