use crate::model::ticket::Ticket;
use crate::model::ticket::TicketState;

use crate::repo::ddb::DDBRepository;
use actix_web::{
    get,
    post,
    put,
    error::ResponseError,
    web::Path,
    web::Json,
    web::Data,
    HttpResponse,
    http::{header::ContentType, StatusCode}
};
use serde::{Serialize, Deserialize};
use derive_more::{Display};


#[derive(Deserialize, Serialize)]
pub struct TicketIdentifier {
    ticket_id: String,
}

#[derive(Deserialize)]
pub struct TicketCompletionRequest {
    delivery: String
}

#[derive(Deserialize)]
pub struct SendTicketRequest {
    user_id: String,
    name: String,
    description: String,
    priority: String,
    source_file: String
}

#[derive(Debug, Display)]
pub enum TicketError {
    TicketNotFound,
    TicketUpdateFailure,
    TicketCreationFailure,
    BadTicketRequest
}

impl ResponseError for TicketError {
    fn status_code(&self) -> StatusCode {
        match self {
            TicketError::TicketNotFound => StatusCode::NOT_FOUND,
            TicketError::TicketUpdateFailure => StatusCode::FAILED_DEPENDENCY,
            TicketError::TicketCreationFailure => StatusCode::FAILED_DEPENDENCY,
            TicketError::BadTicketRequest => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}


#[get("/ticket/{ticket_id}")]
pub async fn get_ticket(
    ddb_repo: Data<DDBRepository>,
    ticket_identifier: Path<TicketIdentifier>
) -> Result<Json<Ticket>, TicketError> {
    let tckt = ddb_repo.get_ticket(
        ticket_identifier.into_inner().ticket_id
    ).await;

    match tckt {
        Some(tckt) => Ok(Json(tckt)),
        None => Err(TicketError::TicketNotFound)
    }
}

#[post("/ticket/new")]
pub async fn send_ticket(
    ddb_repo: Data<DDBRepository>,
    request: Json<SendTicketRequest>
) -> Result<Json<TicketIdentifier>, TicketError> {
    let ticket = Ticket::new (
        request.user_id.clone(),
        request.name.clone(),
        request.description.clone(),
        request.priority.clone(),
        request.source_file.clone(),
    );

    let ticket_identifier = ticket.get_id();
    match ddb_repo.put_ticket(ticket).await {
        Ok(()) => Ok(Json(TicketIdentifier { ticket_id: ticket_identifier })),
        Err(_) => Err(TicketError::TicketCreationFailure)
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    ticket_id: String,
    new_state: TicketState,
    delivery: Option<String>
) -> Result<Json<TicketIdentifier>, TicketError> {
    let mut ticket = match ddb_repo.get_ticket(
        ticket_id
    ).await {
        Some(ticket) => ticket,
        None => return Err(TicketError::TicketNotFound)
    };

    if !ticket.can_transition_to(&new_state) {
        return Err(TicketError::BadTicketRequest);
    };

    ticket.state = new_state;
    ticket.delivery = delivery;

    let ticket_identifier = ticket.get_global_id();
    match ddb_repo.put_ticket(ticket).await {
        Ok(()) => Ok(Json(TicketIdentifier { ticket_id: ticket_identifier })),
        Err(_) => Err(TicketError::TicketUpdateFailure)
    }
}

#[put("/ticket/{ticket_id}/start")]
pub async fn start_ticket(
    ddb_repo: Data<DDBRepository>,
    ticket_identifier: Path<TicketIdentifier>
) -> Result<Json<TicketIdentifier>, TicketError> {
    state_transition(
        ddb_repo,
        ticket_identifier.into_inner().ticket_id,
        TicketState::InProgress,
        None
    ).await
}

#[put("/ticket/{ticket_id}/pause")]
pub async fn pause_ticket(
    ddb_repo: Data<DDBRepository>,
    ticket_identifier: Path<TicketIdentifier>
) -> Result<Json<TicketIdentifier>, TicketError> {
    state_transition(
        ddb_repo,
        ticket_identifier.into_inner().ticket_id,
        TicketState::Paused,
        None
    ).await
}

#[put("/ticket/{ticket_id}/complete")]
pub async fn complete_ticket(
    ddb_repo: Data<DDBRepository>,
    ticket_identifier: Path<TicketIdentifier>,
    completion_request: Json<TicketCompletionRequest>
) -> Result<Json<TicketIdentifier>, TicketError> {
    state_transition(
        ddb_repo,
        ticket_identifier.into_inner().ticket_id,
        TicketState::Completed,
        Some(completion_request.delivery.clone())
    ).await
}