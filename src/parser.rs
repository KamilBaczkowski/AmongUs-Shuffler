use std::str::Chars;

use serenity::model::prelude::UserId;
use tracing::{info, debug};

#[derive(Debug)]
pub enum ShuffleParseError {
    MessageTooShort,
    NotShuffleMessage,
}

const SHUFFLE_KEYWORD: &str = "!shuffle";
const SHUFFLE_KEYWORD_LENGTH: usize = SHUFFLE_KEYWORD.len();
const MENTION_LENGTH: usize = 21; // looks like this: <@285136304914563075>
pub const ID_LENGTH: usize = MENTION_LENGTH - 3; // Remove <, @ and > from the above.

#[tracing::instrument(
    name = "Parsing message",
)]
pub fn parse_shuffle_message(message: String) -> Result<Vec<UserId>, ShuffleParseError> {
    if message.len() < SHUFFLE_KEYWORD_LENGTH {
        debug!(length = message.len(), "Message is too short.");
        return Err(ShuffleParseError::MessageTooShort);
    }

    let mut message = message.chars();

    if message.by_ref().take(SHUFFLE_KEYWORD_LENGTH).collect::<String>() != SHUFFLE_KEYWORD {
        debug!("Message doesn't start with keyword.");
        return Err(ShuffleParseError::NotShuffleMessage);
    }

    info!(message = debug(&message), "Message will be processed.");

    let mut people: Vec<UserId> = vec!();
    loop {
        let next = message.next();
        if next.is_none() {
            debug!("Got no more to consume.");
            break;
        }

        let next = next.unwrap();
        debug!(character = String::from(next), "Got a new character.");
        if next != '<' {
            continue;
        }

        let mention = parse_possible_mention(&mut message);
        match mention {
            Err(_) => (),
            Ok(mentioned) => {
                people.push(mentioned);
            }
        }
    }

    info!(mentions = tracing::field::debug(&people), "Got mentions.");
    Ok(people)
}

#[derive(Debug)]
enum MentionParseError {
    NotAMention,
    TooShort,
    IncorrectLength,
    IllegalCharacter,
}

#[tracing::instrument(
    name="Parsing mention",
    skip(message),
)]
fn parse_possible_mention(message: &mut Chars) -> Result<UserId, MentionParseError> {
    let chars_left = message.clone().count();
    if chars_left < MENTION_LENGTH - 1 /* -1 because < is already consumed */ {
        debug!(
            chars_left, string = message.clone().collect::<String>(),
            "The string is too short to be a mention."
        );
        return Err(MentionParseError::TooShort);
    }

    let next = message.next().unwrap();
    debug!(character = String::from(next), "Got a character.");

    match next {
        '@' => {
            debug!("This may be a mention.");
        },
        _ => {
            debug!("This is not a mention.");
            return Err(MentionParseError::NotAMention)
        },
    }

    let mut mention = String::from("");
    for character in message {
        match character {
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                debug!(character = String::from(character), "Got another digit for ID.");
                mention.push(character);
            },
            '>' => {
                debug!("ID ends here.");
                break;
            },
            _ => {
                debug!(character = String::from(character), "Invalid character encountered.");
                return Err(MentionParseError::IllegalCharacter);
            }
        }
    }

    if mention.len() != ID_LENGTH {
        debug!(length = mention.len(), "The ID does not have the correct length.");
        return Err(MentionParseError::IncorrectLength);
    }

    let user_id: u64 = mention.parse().unwrap();
    let user_id: UserId = user_id.into();

    info!(ID = mention, "Got ID.");
    Ok(user_id)
}

#[cfg(test)]
mod tests {
    use rand::{distributions::{Slice}, Rng};

    use super::*;

    // Tests for mentions parser.
    #[test]
    fn test_parse_possible_mention_valid_mention() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let message = format!("@{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Ok(mention) => {
                if mention == id {
                    Ok(())
                } else {
                    Err(format!("Wrong ID ({}) was returned.", id))
                }
            },
            error => Err(
                format!("An error ({:?}) was returned instead of a valid ID.", error)
            ),
        }
    }

    #[test]
    fn test_parse_possible_mention_mention_a_lot_too_short() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH / 2);
        let message = format!("@{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::TooShort) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_mention_one_char_too_short() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH - 1);
        let message = format!("@{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::TooShort) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_id_a_lot_too_long() -> Result<(), String> {
        let id = "123456789098765432123456789"; // Not generating since it would be too large.
        let message = format!("@{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::IncorrectLength) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}.", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_id_one_char_too_long() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH + 1);
        let message = format!("@{}>", id);
        let mut message = message.chars();

        println!("{}, {:?}, {:?}", ID_LENGTH + 1, id, message);
        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::IncorrectLength) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}.", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_missing_at_length_ok() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH + 1); // +1 since the @ is missing.
        let message = format!("{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::NotAMention) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}.", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_missing_at_length_not_ok() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let message = format!("{}>", id);
        let mut message = message.chars();

        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::TooShort) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}.", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_chars_instead_of_digits_length_ok() -> Result<(), String> {
        let mut message = "@abcdefghijklmnopqr>".chars();
        let result = parse_possible_mention(&mut message);
        match result {
            Err(MentionParseError::IllegalCharacter) => Ok(()),
            Ok(mention) => Err(format!("Got a mention {}.", mention)),
            error => {
                Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
            }
        }
    }

    #[test]
    fn test_parse_possible_mention_one_char_instead_of_a_digit() -> Result<(), String> {
        // Loop changes one character to a letter and checks whether error is returned.
        let id = generate_mention_id(ID_LENGTH);
        let mut id = format!("@{}>", id).chars().collect::<Vec<char>>();
        for i in 1..id.len() - 1 /* -1 because '>' is there */ {
            let old_char = id[i];

            let character = 'a' as u32 + i as u32;
            id[i] = char::from_u32(character).unwrap();

            let message = id.clone().into_iter().collect::<String>();
            let mut message = message.chars();

            let result = parse_possible_mention(&mut message);
            match result {
                Err(MentionParseError::IllegalCharacter) => (),
                Ok(mention) => return Err(format!("Got a mention {}.", mention)),
                error => return Err(
                    format!("A wrong error ({:?}) was returned ({:?}).", error, message)
                ),
            };
            id[i] = old_char;
        }
        Ok(())
    }

    #[test]
    fn test_parse_possible_mention_multiple_chars_instead_of_digits() -> Result<(), String> {
        // Loop changes three characters to a letter and checks if an error is returned.
        let id = generate_mention_id(ID_LENGTH);
        let mut id = format!("@{}>", id).chars().collect::<Vec<char>>();
        for i in 1..id.len() - 3 /* -3 because '>' and three chars are changed, not 1 */ {
            let old_chars: Vec<char> = id[i..i+3].into();

            let character = 'a' as u32 + i as u32;
            id[i] = char::from_u32(character).unwrap();
            id[i+1] = char::from_u32(character).unwrap();
            id[i+2] = char::from_u32(character).unwrap();

            let message = id.clone().into_iter().collect::<String>();
            let mut message = message.chars();

            let result = parse_possible_mention(&mut message);
            match result {
                Err(MentionParseError::IllegalCharacter) => (),
                Ok(mention) => return Err(format!("Got a mention {}", mention)),
                Err(error) => {
                    return Err(format!("A wrong error ({:?}) was returned ({:?}).", error, message))
                },
            };

            id[i] = old_chars[0];
            id[i+1] = old_chars[1];
            id[i+2] = old_chars[2];
        }
        Ok(())
    }

    // Whole command tests.
    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_one_mention() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let message = format!("{} <@{}>", SHUFFLE_KEYWORD, id);
        let result = parse_shuffle_message(message);
        match result {
            Ok(mentions) => {
                if mentions.len() != 1 {
                    return Err(format!("Got wrong number ({}) of mentions.", mentions.len()));
                }
                if mentions[0] != id {
                    return Err(format!("Got invalid mention ({}).", id));
                }
                Ok(())
            }
            Err(error) => Err(format!("An error ({:?}) was returned.", error)),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_two_mentions() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let id2 = generate_mention_id(ID_LENGTH);
        let message = format!("{} <@{}> <@{}>", SHUFFLE_KEYWORD, id, id2);
        let result = parse_shuffle_message(message);
        match result {
            Ok(mentions) => {
                if mentions.len() != 2 {
                    return Err(format!("Got wrong number ({}) of mentions.", mentions.len()));
                }
                if mentions[0] != id {
                    return Err(String::from("Got invalid mention 1."));
                }
                if mentions[1] != id2 {
                    return Err(String::from("Got invalid mention 2."));
                }
                Ok(())
            }
            Err(error) => Err(format!("An error ({:?}) was returned.", error)),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_ten_mentions() -> Result<(), String> {
        let mut ids = vec!();
        let mut message = format!("{} ", SHUFFLE_KEYWORD);
        for _ in 0..10 {
            let id = generate_mention_id(ID_LENGTH);
            ids.push(id.clone());
            message = format!("{} <@{}> ", message, id);
        }
        let result = parse_shuffle_message(message);
        match result {
            Ok(mentions) => {
                for (index, mention) in mentions.iter().enumerate() {
                    if *mention != ids[index] {
                        return Err(format!(
                            "Invalid mention {}, expected {}, got {}.",
                            index, ids[index], *mention
                        ));
                    }
                }
                Ok(())
            }
            Err(_) => Err(String::from("An error was returned.")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_message_too_short() -> Result<(), String> {
        let message = String::from("!shuff ");
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::MessageTooShort) => Ok(()),
            Ok(mentions) => Err(format!("Got mentions {:?}", mentions)),
            Err(error) => Err(
                format!("A wrong error ({:?}) was returned ({:?}).", error, message)
            ),
        }
    }

    #[test]
    fn test_parse_shuffle_message_invalid_shuffle_command() -> Result<(), String> {
        let message = String::from("!shufffle ");
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::NotShuffleMessage) => Ok(()),
            Ok(mentions) => Err(format!("Got mentions {:?}", mentions)),
            Err(error) => Err(
                format!("A wrong error ({:?}) was returned ({:?}).", error, message)
            ),
        }
    }

    // No 0 to not generate numbers with leading 0, simplifies a lot of things.
    const DIGITS: [char; 9] = ['1','2','3','4','5','6','7','8','9'];
    fn generate_mention_id(length: usize) -> u64 {
        let distribution = Slice::new(&DIGITS).unwrap();
        let rng = &mut rand::thread_rng();
        let result = rng.sample_iter(&distribution).take(length).collect::<String>();
        result.parse().unwrap()
    }
}
