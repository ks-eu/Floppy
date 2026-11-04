use crate::error::ProtocolError;

#[derive(Debug)]
pub struct Join {
    length: u8,
    username: String,
}

#[derive(Debug)]
pub struct JoinSuccess {}

#[derive(Debug)]
pub struct JoinFail {
    id: u8,
}

#[derive(Debug)]
pub struct Message {
    length: u16,
    msg: String,
}

#[derive(Debug)]
pub struct ClientMessage {
    uuid: u16,
    length: u16,
    msg: String,
}

#[derive(Debug)]
pub struct UserJoin {
    uuid: u16,
    length: u8,
    username: String,
}

impl Join {
    pub fn new(username: &str) -> Result<Self, ProtocolError> {
        if username.chars().count() > 16 {
            return Err(ProtocolError::UsernameTooLong);
        }

        Ok(Self {
            length: username.len() as u8,
            username: username.to_string(),
        })
    }
    
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.username.len() != self.length as usize {
            return Err(ProtocolError::LengthMismatch);
        }
        if self.username.len() > 255 {
            return Err(ProtocolError::UsernameTooLong);
        }

        let mut buffer = Vec::<u8>::new();
        buffer.push(0x01);
        buffer.push(self.length);
        buffer.extend_from_slice(self.username.as_bytes());

        Ok(buffer)
    }
}

impl JoinSuccess {
    pub fn encode(&self) -> Vec<u8> {
        vec![0x02]
    }
}

impl JoinFail {
    pub fn new(code: u8) -> Self {
        Self {
            id: code,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        vec![0x03, self.id]
    }
}

impl Message {
    pub fn new(msg: &str) -> Result<Self, ProtocolError> {
        if msg.chars().count() > 255 {
            return Err(ProtocolError::MessageTooLong);
        }

        Ok(Self {
            length: msg.len() as u16,
            msg: msg.to_string(),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.msg.len() != self.length as usize {
            return Err(ProtocolError::LengthMismatch);
        }
        if self.msg.len() > 1020 {
            return Err(ProtocolError::MessageTooLong);
        }

        let mut buffer = Vec::<u8>::new();
        buffer.push(0x04);
        buffer.extend_from_slice(&self.length.to_be_bytes());
        buffer.extend_from_slice(&self.msg.as_bytes());

        Ok(buffer)
    }
}

impl ClientMessage {
    pub fn new(uuid: u16, msg: &str) -> Result<Self, ProtocolError> {
        if msg.chars().count() > 255 {
            return Err(ProtocolError::MessageTooLong);
        }
        
        Ok(Self {
            uuid: uuid,
            length: msg.len() as u16,
            msg: msg.to_string(),
        })
    }

    pub fn from_message(message: Message, uuid: u16) -> Self {
        Self {
            uuid: uuid,
            length: message.length,
            msg: message.msg,
        }
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.msg.len() != self.length as usize {
            return Err(ProtocolError::LengthMismatch);
        }
        if self.msg.len() > 1020 {
            return Err(ProtocolError::MessageTooLong);
        }

        let mut buffer = Vec::<u8>::new();
        buffer.push(0x05);
        buffer.extend_from_slice(&self.uuid.to_be_bytes());
        buffer.extend_from_slice(&self.length.to_be_bytes());
        buffer.extend_from_slice(&self.msg.as_bytes());

        Ok(buffer)
    }
}

impl UserJoin {
    pub fn new(username: &str, uuid: u16) -> Result<Self, ProtocolError> {
        if username.chars().count() > 16 {
            return Err(ProtocolError::UsernameTooLong);
        }

        Ok(Self {
            uuid: uuid,
            length: username.len() as u8,
            username: username.to_string(),
        })
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        if self.username.len() != self.length as usize {
            return Err(ProtocolError::LengthMismatch);
        }
        if self.username.len() > 255 {
            return Err(ProtocolError::UsernameTooLong);
        }

        let mut buffer = Vec::<u8>::new();
        buffer.push(0x06);
        buffer.extend_from_slice(&self.uuid.to_be_bytes());
        buffer.push(self.length);
        buffer.extend_from_slice(&self.username.as_bytes());

        Ok(buffer)
    }
}

#[cfg(test)]
#[path = "tests/header.rs"]
mod tests;
