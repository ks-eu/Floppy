use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc};

use crate::error::UsersError;
use crate::message::event::ServerEvent;


pub enum TryInsert {
    Success(u16),
    Fail(UsersError),
}

pub enum TryRemove {
    Success,
    Fail,
}

pub struct ConnectedUser {
    username: String,
    tx: Option<mpsc::Sender<ServerEvent>>,
}

impl ConnectedUser {
    pub fn new(username: String) -> Self {
        Self { username, tx: None }
    }

    pub fn set_sender(&mut self, tx: mpsc::Sender<ServerEvent>) {
        self.tx = Some(tx);
    }
}

pub struct UserState {
    usernames: HashMap<u16, ConnectedUser>,
    count: usize,
    max_count: usize,
    first_free_id: usize,
}

impl UserState {
    pub fn new(max_count: usize) -> Result<Self, UsersError> {
        if max_count > usize::from(u16::MAX) {
            return Err(UsersError::InvalidMaxCount{max_count});
        }

        Ok(Self {
            usernames: HashMap::new(),
            count: 0,
            max_count,
            first_free_id: 0,
        })
    }

    pub fn username(&self, id: u16) -> Option<&str> {
        self.usernames.get(&id).map(|user| user.username.as_str())
    }

    pub fn insert(&mut self, username: &str) -> Result<u16, UsersError> {
        if self.count == self.max_count {
            return Err(UsersError::AtCapacity);
        }
        if self.usernames.values().any(|user| user.username == username) {
            return Err(UsersError::DuplicateUsername);
        }

        let Ok(id) = u16::try_from(self.first_free_id) else {
            return Err(UsersError::AtCapacity);
        };
        self.usernames.insert(
            id,
            ConnectedUser::new(username.to_owned()),
        );
        self.count += 1;
        self.update_first_free_id();
        Ok(id)
    }

    pub fn set_sender(
        &mut self,
        id: u16,
        tx: mpsc::Sender<ServerEvent>,
    ) -> Result<(), UsersError> {
        let user = self
            .usernames
            .get_mut(&id)
            .ok_or(UsersError::UserNotFound{id})?;
        user.set_sender(tx);
        Ok(())
    }

    pub fn sender(&self, id: u16) -> Option<&mpsc::Sender<ServerEvent>> {
        self.usernames.get(&id).and_then(|user| user.tx.as_ref())
    }

    pub fn remove(&mut self, id: u16) -> Result<(), UsersError> {
        if self.usernames.remove(&id).is_none() {
            return Err(UsersError::UserNotFound{id});
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


pub async fn try_insert(
    users: Arc<RwLock<UserState>>,
    username: &str,
) -> TryInsert {
    let mut guard = users.write().await;
    let result = guard.insert(username);
    drop(guard);

    match result {
        Ok(id) => {
            return TryInsert::Success(id)
        }
        Err(error) => {
            return TryInsert::Fail(error)
        }
    }
}


pub async fn try_set_sender(
    users: Arc<RwLock<UserState>>,
    id: u16,
    tx: mpsc::Sender<ServerEvent>,
) -> Result<(), UsersError> {
    users.write().await.set_sender(id, tx)
}


pub async fn try_remove(
    users: Arc<RwLock<UserState>>,
    id: u16,
) -> TryRemove {
    let mut guard = users.write().await;
    let result = guard.remove(id);
    drop(guard);

    match result {
        Ok(_) => {
            return TryRemove::Success
        }
        Err(_) => {
            return TryRemove::Fail
        }
    }
}
