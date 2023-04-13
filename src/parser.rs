use tracing::{info, debug};

#[derive(Debug)]
pub enum ShuffleParseError {
    MessageTooShort,
    NotShuffleMessage,
}

const SHUFFLE_KEYWORD: &str = "!shuffle ";
const SHUFFLE_KEYWORD_SHORT: &str = "!s ";

const SHUFFLE_KEYWORD_LENGTH: usize = SHUFFLE_KEYWORD.len();
const SHUFFLE_KEYWORD_SHORT_LENGTH: usize = SHUFFLE_KEYWORD_SHORT.len();

#[tracing::instrument(
    name = "Parsing message",
)]
pub fn parse_shuffle_message(message: String) -> Result<(), ShuffleParseError> {
    if message.len() < SHUFFLE_KEYWORD_SHORT_LENGTH {
        debug!(length = message.len(), "Message is too short.");
        return Err(ShuffleParseError::MessageTooShort);
    }

    let message = message.chars();

    // Check if the message contains the keyword.
    if message.clone().take(SHUFFLE_KEYWORD_SHORT_LENGTH).collect::<String>() != SHUFFLE_KEYWORD_SHORT
    && message.clone().take(SHUFFLE_KEYWORD_LENGTH).collect::<String>() != SHUFFLE_KEYWORD {
        debug!("Message doesn't start with keyword.");
        return Err(ShuffleParseError::NotShuffleMessage);
    }

    info!("Message will be processed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{distributions::{Slice}, Rng};

    use super::*;

    const MENTION_LENGTH: usize = 21; // looks like this: <@285136304914563075>
    pub const ID_LENGTH: usize = MENTION_LENGTH - 3; // Remove <, @ and > from the above.

    // Whole command tests.
    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_one_mention() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let message = format!("{SHUFFLE_KEYWORD}<@{id}>");
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_two_mentions() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let id2 = generate_mention_id(ID_LENGTH);
        let message = format!("{SHUFFLE_KEYWORD}<@{id}> <@{id2}>");
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_shuffle_command_ten_mentions() -> Result<(), String> {
        let mut ids = vec!();
        let mut message = format!("{SHUFFLE_KEYWORD}");
        for _ in 0..10 {
            let id = generate_mention_id(ID_LENGTH);
            ids.push(id.clone());
            message = format!("{message}<@{id}> ");
        }
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_message_too_short() -> Result<(), String> {
        let message = String::from("!shuff ");
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::NotShuffleMessage) => Ok(()),
            Ok(mentions) => Err(format!("Got mentions {mentions:?}")),
            Err(error) => Err(
                format!("A wrong error ({error:?}) was returned ({message:?}).")
            ),
        }
    }

    #[test]
    fn test_parse_shuffle_message_invalid_shuffle_command() -> Result<(), String> {
        let message = String::from("!shufffle ");
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::NotShuffleMessage) => Ok(()),
            Ok(mentions) => Err(format!("Got mentions {mentions:?}")),
            Err(error) => Err(
                format!("A wrong error ({error:?}) was returned ({message:?}).")
            ),
        }
    }

    // Short command tests.
    #[test]
    fn test_parse_shuffle_message_valid_short_shuffle_command_one_mention() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let message = format!("{SHUFFLE_KEYWORD_SHORT}<@{id}>");
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_short_shuffle_command_two_mentions() -> Result<(), String> {
        let id = generate_mention_id(ID_LENGTH);
        let id2 = generate_mention_id(ID_LENGTH);
        let message = format!("{SHUFFLE_KEYWORD_SHORT}<@{id}> <@{id2}>");
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_valid_short_shuffle_command_ten_mentions() -> Result<(), String> {
        let mut ids = vec!();
        let mut message = format!("{SHUFFLE_KEYWORD_SHORT}");
        for _ in 0..10 {
            let id = generate_mention_id(ID_LENGTH);
            ids.push(id.clone());
            message = format!("{message}<@{id}> ");
        }
        let result = parse_shuffle_message(message.clone());
        match result {
            Ok(_) => Ok(()),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_invalid_shuffle_command_no_space() -> Result<(), String> {
        let mut ids = vec!();
        let mut message = String::from("!shuffle");
        for _ in 0..10 {
            let id = generate_mention_id(ID_LENGTH);
            ids.push(id.clone());
            message = format!("{message}<@{id}> ");
        }
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::NotShuffleMessage) => Ok(()),
            Ok(_) => Err(String::from("Got an OK, instead of an error")),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
        }
    }

    #[test]
    fn test_parse_shuffle_message_invalid_short_shuffle_command_no_space() -> Result<(), String> {
        let mut ids = vec!();
        let mut message = String::from("!s");
        for _ in 0..10 {
            let id = generate_mention_id(ID_LENGTH);
            ids.push(id.clone());
            message = format!("{message}<@{id}> ");
        }
        let result = parse_shuffle_message(message.clone());
        match result {
            Err(ShuffleParseError::NotShuffleMessage) => Ok(()),
            Ok(_) => Err(String::from("Got an OK, instead of an error")),
            Err(error) => Err(format!("An error ({error:?}) was returned. {message}")),
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
