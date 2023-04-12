use rand::seq::SliceRandom;

use crate::game::{Players, Pairs};

#[derive(Debug)]
pub enum ShuffleError {
    TooFewPeople,
    DuplicatesDetected,
}
pub fn shuffle_people(people: &Players) -> Result<Pairs, ShuffleError> {
    let mut result = vec!();
    if people.len() < 2 {
        return Err(ShuffleError::TooFewPeople);
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
        result.push((players[i], players[i+1]));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Slice, Rng};

    use crate::{parser::ID_LENGTH, game::{Players, Pairs}};

    use super::{shuffle_people, ShuffleError};

    #[derive(Debug)]
    enum TestResult {
        String(String),
        PairValidityError(PairValidityError),
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_two_people() -> Result<(), TestResult> {
        // let mut ids = vec!();
        let ids = generate_user_ids(2).into();

        match shuffle_people(&ids) {
            Ok(shuffled) => {
                println!("Players: {:?}.", shuffled);
                match check_pairs_validity(shuffled, 2) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({:?}).", error))),
        }
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_three_people() -> Result<(), TestResult> {
        // let mut ids = vec!();
        let ids = generate_user_ids(3).into();

        match shuffle_people(&ids) {
            Ok(shuffled) => {
                println!("Players: {:?}.", shuffled);
                match check_pairs_validity(shuffled, 3) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({:?}).", error))),
        }
    }

    #[test]
    fn test_shuffle_people_properly_shuffles_hundred_people() -> Result<(), TestResult> {
        // let mut ids = vec!();
        let ids = generate_user_ids(100).into();

        match shuffle_people(&ids) {
            Ok(shuffled) => {
                println!("Players: {:?}.", shuffled);
                match check_pairs_validity(shuffled, 100) {
                    Ok(_) => Ok(()),
                    Err(error) => Err(TestResult::PairValidityError(error)),
                }
            }
            Err(error) => Err(TestResult::String(format!("Got an error ({:?}).", error))),
        }
    }

    #[test]
    fn test_shuffle_errors_on_no_people() -> Result<(), String> {
        // let mut ids = vec!();
        let ids = generate_user_ids(0).into();

        match shuffle_people(&ids) {
            Err(ShuffleError::TooFewPeople) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({:?}).", shuffled)),
            Err(error) => Err(format!("A wrong error was returned ({:?}).", error)),
        }
    }

    #[test]
    fn test_shuffle_errors_on_one_person() -> Result<(), String> {
        // let mut ids = vec!();
        let ids = generate_user_ids(1).into();

        match shuffle_people(&ids) {
            Err(ShuffleError::TooFewPeople) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({:?}).", shuffled)),
            Err(error) => Err(format!("A wrong error was returned ({:?}).", error)),
        }
    }

    #[test]
    fn test_shuffle_errors_on_duplicate() -> Result<(), String> {
        // let mut ids = vec!();
        let mut ids: Players = generate_user_ids(3).into();
        ids.push(ids[0]);

        match shuffle_people(&ids) {
            Err(ShuffleError::DuplicatesDetected) => Ok(()),
            Ok(shuffled) => Err(format!("Got shuffled people ({:?}).", shuffled)),
            Err(error) => Err(format!("A wrong error was returned ({:?}).", error)),
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
        pairs: Pairs, expected_pair_count: usize
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
}
