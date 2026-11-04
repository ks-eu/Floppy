use super::*;

mod join_tests {
    use super::*;

    #[test]
    fn test_new_join() {
        let join = Join::new("Alice").unwrap();
        assert_eq!(join.length, 5);
        assert_eq!(join.username, "Alice");
    }

    #[test]
    fn test_encode_join() {
        let join = Join::new("Alice").unwrap();
        assert_eq!(
            join.encode().unwrap(),
            vec![0x01, 0x05, 0x41, 0x6C, 0x69, 0x63, 0x65]
        );
    }

    #[test]
    fn test_empty_username() {
        let join = Join::new("").unwrap();
        assert_eq!(join.encode().unwrap(), vec![0x01, 0x00]);
    }

    #[test]
    fn test_username_too_long() {
        let username = "a".repeat(17);
        assert!(matches!(
            Join::new(&username),
            Err(ProtocolError::UsernameTooLong)
        ));
    }

    #[test]
    fn test_maximum_username_length() {
        let username = "a".repeat(16);
        let join = Join::new(&username).unwrap();
        assert_eq!(join.length, 16);
    }

    #[test]
    fn test_utf8_username() {
        let join = Join::new("é").unwrap();
        assert_eq!(join.length, 2);
        assert_eq!(join.encode().unwrap(), vec![0x01, 0x02, 0xC3, 0xA9]);
    }

    #[test]
    fn test_maximum_utf8_username() {
        let username = "é".repeat(16);
        let join = Join::new(&username).unwrap();
        assert_eq!(join.length, 32);
        assert_eq!(join.encode().unwrap().len(), 34);
    }

    #[test]
    fn test_length_mismatch() {
        let mut join = Join::new("Alice").unwrap();
        join.length = 4;
        assert!(matches!(
            join.encode(),
            Err(ProtocolError::LengthMismatch)
        ));
    }
}

mod join_success_tests {
    use super::*;

    #[test]
    fn test_encode_join_success() {
        let success = JoinSuccess {};
        assert_eq!(success.encode(), vec![0x02]);
    }
}

mod join_fail_tests {
    use super::*;

    #[test]
    fn test_new_join_fail() {
        let fail = JoinFail::new(0x04);
        assert_eq!(fail.id, 0x04);
    }

    #[test]
    fn test_encode_join_fail() {
        let fail = JoinFail::new(0x04);
        assert_eq!(fail.encode(), vec![0x03, 0x04]);
    }

    #[test]
    fn test_encode_join_fail_zero_code() {
        let fail = JoinFail::new(0x00);
        assert_eq!(fail.encode(), vec![0x03, 0x00]);
    }

    #[test]
    fn test_encode_join_fail_maximum_code() {
        let fail = JoinFail::new(0xFF);
        assert_eq!(fail.encode(), vec![0x03, 0xFF]);
    }
}

mod message_tests {
    use super::*;

    #[test]
    fn test_new_message() {
        let message = Message::new("Hello").unwrap();
        assert_eq!(message.length, 5);
        assert_eq!(message.msg, "Hello");
    }

    #[test]
    fn test_encode_message() {
        let message = Message::new("Hello").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![0x04, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
    }

    #[test]
    fn test_empty_message() {
        let message = Message::new("").unwrap();
        assert_eq!(message.encode().unwrap(), vec![0x04, 0x00, 0x00]);
    }

    #[test]
    fn test_message_too_long() {
        let msg = "a".repeat(256);
        assert!(matches!(
            Message::new(&msg),
            Err(ProtocolError::MessageTooLong)
        ));
    }

    #[test]
    fn test_maximum_message_length() {
        let msg = "a".repeat(255);
        let message = Message::new(&msg).unwrap();
        assert_eq!(message.length, 255);
    }

    #[test]
    fn test_utf8_message() {
        let message = Message::new("é").unwrap();
        assert_eq!(message.length, 2);
        assert_eq!(
            message.encode().unwrap(),
            vec![0x04, 0x00, 0x02, 0xC3, 0xA9]
        );
    }

    #[test]
    fn test_maximum_utf8_message() {
        let msg = "é".repeat(255);
        let message = Message::new(&msg).unwrap();
        assert_eq!(message.length, 510);
        assert_eq!(message.encode().unwrap().len(), 513);
    }

    #[test]
    fn test_length_mismatch() {
        let mut message = Message::new("Hello").unwrap();
        message.length = 4;
        assert!(matches!(
            message.encode(),
            Err(ProtocolError::LengthMismatch)
        ));
    }
}

mod client_message_tests {
    use super::*;

    #[test]
    fn test_new_client_message() {
        let message = ClientMessage::new(1234, "Hello").unwrap();
        assert_eq!(message.uuid, 1234);
        assert_eq!(message.length, 5);
        assert_eq!(message.msg, "Hello");
    }

    #[test]
    fn test_encode_client_message() {
        let message = ClientMessage::new(1234, "Hello").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![
                0x05, 0x04, 0xD2, 0x00, 0x05,
                0x48, 0x65, 0x6C, 0x6C, 0x6F
            ]
        );
    }

    #[test]
    fn test_empty_client_message() {
        let message = ClientMessage::new(1234, "").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![0x05, 0x04, 0xD2, 0x00, 0x00]
        );
    }

    #[test]
    fn test_message_too_long() {
        let msg = "a".repeat(256);
        assert!(matches!(
            ClientMessage::new(1234, &msg),
            Err(ProtocolError::MessageTooLong)
        ));
    }

    #[test]
    fn test_maximum_message_length() {
        let msg = "a".repeat(255);
        let message = ClientMessage::new(1234, &msg).unwrap();
        assert_eq!(message.length, 255);
    }

    #[test]
    fn test_utf8_message() {
        let message = ClientMessage::new(1234, "é").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![0x05, 0x04, 0xD2, 0x00, 0x02, 0xC3, 0xA9]
        );
    }

    #[test]
    fn test_from_message() {
        let message = Message::new("Hello").unwrap();
        let client_message = ClientMessage::from_message(message, 1234);

        assert_eq!(client_message.uuid, 1234);
        assert_eq!(client_message.length, 5);
        assert_eq!(client_message.msg, "Hello");
    }

    #[test]
    fn test_from_message_encoding() {
        let message = Message::new("Hello").unwrap();
        let client_message = ClientMessage::from_message(message, 1234);

        assert_eq!(
            client_message.encode().unwrap(),
            vec![
                0x05, 0x04, 0xD2, 0x00, 0x05,
                0x48, 0x65, 0x6C, 0x6C, 0x6F
            ]
        );
    }

    #[test]
    fn test_uuid_zero() {
        let message = ClientMessage::new(0, "Hello").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![0x05, 0x00, 0x00, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
    }

    #[test]
    fn test_uuid_maximum() {
        let message = ClientMessage::new(u16::MAX, "Hello").unwrap();
        assert_eq!(
            message.encode().unwrap(),
            vec![0x05, 0xFF, 0xFF, 0x00, 0x05, 0x48, 0x65, 0x6C, 0x6C, 0x6F]
        );
    }

    #[test]
    fn test_length_mismatch() {
        let mut message = ClientMessage::new(1234, "Hello").unwrap();
        message.length = 4;
        assert!(matches!(
            message.encode(),
            Err(ProtocolError::LengthMismatch)
        ));
    }
}

mod user_join_tests {
    use super::*;

    #[test]
    fn test_new_user_join() {
        let join = UserJoin::new("Alice", 1234).unwrap();
        assert_eq!(join.uuid, 1234);
        assert_eq!(join.length, 5);
        assert_eq!(join.username, "Alice");
    }

    #[test]
    fn test_encode_user_join() {
        let join = UserJoin::new("Alice", 1234).unwrap();
        assert_eq!(
            join.encode().unwrap(),
            vec![0x06, 0x04, 0xD2, 0x05, 0x41, 0x6C, 0x69, 0x63, 0x65]
        );
    }

    #[test]
    fn test_empty_username() {
        let join = UserJoin::new("", 1234).unwrap();
        assert_eq!(join.encode().unwrap(), vec![0x06, 0x04, 0xD2, 0x00]);
    }

    #[test]
    fn test_username_too_long() {
        let username = "a".repeat(17);
        assert!(matches!(
            UserJoin::new(&username, 1234),
            Err(ProtocolError::UsernameTooLong)
        ));
    }

    #[test]
    fn test_maximum_username_length() {
        let username = "a".repeat(16);
        let join = UserJoin::new(&username, 1234).unwrap();
        assert_eq!(join.length, 16);
    }

    #[test]
    fn test_utf8_username() {
        let join = UserJoin::new("é", 1234).unwrap();
        assert_eq!(
            join.encode().unwrap(),
            vec![0x06, 0x04, 0xD2, 0x02, 0xC3, 0xA9]
        );
    }

    #[test]
    fn test_maximum_utf8_username() {
        let username = "é".repeat(16);
        let join = UserJoin::new(&username, 1234).unwrap();
        assert_eq!(join.length, 32);
        assert_eq!(join.encode().unwrap().len(), 36);
    }

    #[test]
    fn test_uuid_zero() {
        let join = UserJoin::new("Alice", 0).unwrap();
        assert_eq!(join.encode().unwrap()[1..3], [0x00, 0x00]);
    }

    #[test]
    fn test_uuid_maximum() {
        let join = UserJoin::new("Alice", u16::MAX).unwrap();
        assert_eq!(join.encode().unwrap()[1..3], [0xFF, 0xFF]);
    }

    #[test]
    fn test_length_mismatch() {
        let mut join = UserJoin::new("Alice", 1234).unwrap();
        join.length = 4;
        assert!(matches!(
            join.encode(),
            Err(ProtocolError::LengthMismatch)
        ));
    }
}
