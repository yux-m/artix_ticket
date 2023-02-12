use serde::Serialize;
use uuid::Uuid;
use strum_macros::{EnumString, Display};

#[derive(Serialize, EnumString, Display, Eq, PartialEq)]
pub enum TicketState {
    New,
    InProgress,
    Completed,
    Paused
}

#[derive(Serialize)]
pub struct Ticket {
    pub ticket_uuid: String,
    pub user_uuid: String,
    pub name: String,
    pub state: TicketState,
    pub priority: String,
    pub description: String,
    pub source_file: String,
    pub delivery: Option<String>
}

impl Ticket {
    pub fn new(user_uuid: String, name: String, description: String, priority: String, source_file: String) -> Ticket {
        Ticket {
            user_uuid,
            ticket_uuid: Uuid::new_v4().to_string(),
            name,
            state: TicketState::New,
            priority,
            description,
            source_file,
            delivery: None
        }
    }

    pub fn get_id(&self) -> String {
        return format!("{}_{}", self.user_uuid, self.ticket_uuid);
    }
}