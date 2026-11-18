use std::collections::HashMap;
use crate::error::UsersError;

#[derive(Debug)]
pub struct Users {
    usernames: HashMap<u16, String>,
    count: usize,
    max_count: usize,
    first_free_id: usize,
}

impl Users {
    pub fn new(max_count: usize) -> Result<Self, UsersError> {
        if max_count > usize::from(u16::MAX) {
            return Err(UsersError::InvalidMaxCount { max_count });
        }

        Ok(Self {
            usernames: HashMap::new(),
            count: 0,
            max_count,
            first_free_id: 0,
        })
    }

    pub fn usernames(&self) -> &HashMap<u16, String> {
        &self.usernames
    }

    pub fn count(&self) -> &usize {
        &self.count
    }

    pub fn max_count(&self) -> &usize {
        &self.max_count
    }

    pub fn first_free_id(&self) -> &usize {
        &self.first_free_id
    }

    pub fn username(&self, id: u16) -> Option<&str> {
        self.usernames.get(&id).map(String::as_str)
    }

    pub fn insert(&mut self, username: &str) -> Result<u16, UsersError> {
        if self.first_free_id == self.max_count {
            return Err(UsersError::AtCapacity);
        }
        if self.usernames.values().any(|stored| stored == username) {
            return Err(UsersError::DuplicateUsername);
        }

        let Ok(id) = u16::try_from(self.first_free_id) else {
            return Err(UsersError::AtCapacity);
        };
        self.usernames.insert(id, username.to_owned());
        self.count += 1;
        self.update_first_free_id();
        Ok(id)
    }

    pub fn remove(&mut self, id: u16) -> Result<(), UsersError> {
        if self.usernames.remove(&id).is_none() {
            return Err(UsersError::UserNotFound { id });
        }

        self.count -= 1;
        self.first_free_id = self.first_free_id.min(usize::from(id));
        Ok(())
    }

    fn update_first_free_id(&mut self) {
        self.first_free_id = (0..self.max_count)
            .find(|id| {
                u16::try_from(*id)
                    .ok()
                    .is_some_and(|id| !self.usernames.contains_key(&id))
            })
            .unwrap_or(self.max_count);
    }
}

#[cfg(test)]
#[path = "tests/server.rs"]
mod tests;
