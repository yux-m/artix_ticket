use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_config::SdkConfig;
use crate::model::ticket::{Ticket, TicketState};
use log::error;
use std::str::FromStr;
use std::collections::HashMap;

pub struct DDBRepository {  //interface between backend & dynamoDB
    client: Client,
    table_name: String
}

pub struct DDBError;

fn required_item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<String, DDBError> {
    match item_value(key, item) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(DDBError),
        Err(DDBError) => Err(DDBError)
    }
}

fn item_value(key: &str, item: &HashMap<String, AttributeValue>) -> Result<Option<String>, DDBError> {
    match item.get(key) {
        Some(value) => match value.as_s() {
            Ok(val) => Ok(Some(val.clone())),
            Err(_) => Err(DDBError)
        },
        None => Ok(None)
    }
}

fn item_to_ticket(item: &HashMap<String, AttributeValue>) -> Result<Ticket, DDBError> {
    let state: TicketState = match TicketState::from_str(required_item_value("state", item)?.as_str()) {
        Ok(value) => value,
        Err(_) => return Err(DDBError)
    };

    let delivery = item_value("delivery", item)?;

    Ok(Ticket {
        user_uuid: required_item_value("pK", item)?,
        ticket_uuid: required_item_value("sK", item)?,
        name: required_item_value("name", item)?,
        state,
        priority: required_item_value("priority", item)?,
        description: required_item_value("description", item)?,
        source_file: required_item_value("source_file", item)?,
        delivery
    })
}

impl DDBRepository {
    pub fn init(table_name: String, config: SdkConfig) -> DDBRepository {
        let client = Client::new(&config);
        DDBRepository {
            table_name,
            client
        }
    }

    pub async fn put_ticket(&self, ticket: Ticket) -> Result<(), DDBError> {
        let mut request = self.client.put_item()
            .table_name(&self.table_name)
            .item("pK", AttributeValue::S(String::from(ticket.user_uuid)))
            .item("sK", AttributeValue::S(String::from(ticket.ticket_uuid)))
            .item("name", AttributeValue::S(String::from(ticket.name)))
            .item("state", AttributeValue::S(ticket.state.to_string()))
            .item("priority", AttributeValue::S(String::from(ticket.priority)))
            .item("description", AttributeValue::S(String::from(ticket.description)))
            .item("source_file", AttributeValue::S(String::from(ticket.source_file)));

        if let Some(delivery) = ticket.result_file {
            request = request.item("delivery", AttributeValue::S(String::from(delivery)));
        }

        match request.send().await {
            Ok(_) => Ok(()),
            Err(_) => Err(DDBError)
        }
    }

    pub async fn get_ticket(&self, ticket_id: String) -> Option<Ticket> {
        let tokens:Vec<String> = ticket_id
            .split("_")
            .map(|x| String::from(x))
            .collect();
        let user_uuid = AttributeValue::S(tokens[0].clone());
        let ticket_uuid = AttributeValue::S(tokens[1].clone());

        let res = self.client
            .query()
            .table_name(&self.table_name)
            .key_condition_expression("#pK = :user_id and #sK = :ticket_uuid")
            .expression_attribute_names("#pK", "pK")
            .expression_attribute_names("#sK", "sK")
            .expression_attribute_values(":user_id", user_uuid)
            .expression_attribute_values(":ticket_uuid", ticket_uuid)
            .send()
            .await;

        return match res {
            Ok(output) => {
                match output.items {
                    Some(items) => {
                        let item = &items.first()?;
                        error!("{:?}", &item);
                        match item_to_ticket(item) {
                            Ok(ticket) => Some(ticket),
                            Err(_) => None
                        }
                    },
                    None => {
                        None
                    }
                }
            },
            Err(error) => {
                error!("{:?}", error);
                None
            }
        }
    }
}