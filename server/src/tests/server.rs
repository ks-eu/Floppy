use super::{Users, UsersError};

#[test]
fn validates_capacity() {
    let mut empty = Users::new(0).expect("zero capacity is valid");
    assert!(empty.usernames().is_empty());
    assert_eq!(empty.count(), &0);
    assert_eq!(empty.max_count(), &0);
    assert_eq!(empty.first_free_id(), &0);
    assert!(matches!(empty.insert("Alice"), Err(UsersError::AtCapacity)));
    assert!(empty.usernames().is_empty());
    assert_eq!(empty.count(), &0);
    assert_eq!(empty.first_free_id(), &0);

    let largest = Users::new(65_535).expect("maximum capacity is valid");
    assert_eq!(largest.max_count(), &65_535);
    assert!(matches!(
        Users::new(65_536),
        Err(UsersError::InvalidMaxCount { max_count: 65_536 })
    ));
}

#[test]
fn allocates_sequentially_and_checks_exact_names() {
    let mut users = Users::new(2).expect("valid capacity");
    assert_eq!(users.insert("Alice ").expect("insert Alice"), 0);
    let username = users.username(0).expect("registered user");
    assert_eq!(username, "Alice ");
    assert_eq!(Some(username), users.usernames().get(&0).map(String::as_str));
    assert_eq!(users.username(1), None);

    assert!(matches!(users.insert("Alice "), Err(UsersError::DuplicateUsername)));
    assert_eq!(users.count(), &1);
    assert_eq!(users.first_free_id(), &1);
    assert_eq!(users.usernames().len(), 1);
    assert_eq!(users.insert("alice ").expect("insert case variant"), 1);

    assert!(matches!(users.insert("Alice "), Err(UsersError::AtCapacity)));
    assert_eq!(users.count(), &2);
    assert_eq!(users.first_free_id(), &2);
    assert_eq!(users.username(0), Some("Alice "));
    assert_eq!(users.username(1), Some("alice "));
}

#[test]
fn removals_reuse_the_lowest_available_id() {
    let mut users = Users::new(5).expect("valid capacity");
    for name in ["Alice", "Bob", "Carol", "Dave", "Eve"] {
        users.insert(name).expect("insert user");
    }

    users.remove(3).expect("remove Dave");
    assert_eq!(users.first_free_id(), &3);
    users.remove(1).expect("remove Bob");
    assert_eq!(users.username(1), None);
    assert!(matches!(users.remove(1), Err(UsersError::UserNotFound { id: 1 })));
    assert_eq!(users.count(), &3);
    assert_eq!(users.usernames().len(), 3);
    assert_eq!(users.first_free_id(), &1);

    assert_eq!(users.insert("Bob").expect("reuse ID 1"), 1);
    assert_eq!(users.first_free_id(), &3);
    assert_eq!(users.insert("Frank").expect("reuse ID 3"), 3);
    assert_eq!(users.count(), &5);
    assert_eq!(users.usernames().len(), 5);
    assert_eq!(users.first_free_id(), &5);
}

#[test]
fn fills_the_last_slot_at_maximum_capacity() {
    let mut users = Users::new(65_535).expect("maximum capacity is valid");
    users.usernames = (0..65_534).map(|id| (id, id.to_string())).collect();
    users.count = users.usernames.len();
    users.first_free_id = users.count;

    assert_eq!(users.insert("last").expect("insert last user"), 65_534);
    assert_eq!(users.count(), &65_535);
    assert_eq!(users.first_free_id(), &65_535);
    assert!(matches!(users.insert("next"), Err(UsersError::AtCapacity)));
}
