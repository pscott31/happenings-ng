use crate::{generic_id::{GenericId, TableName}, ticket::{TicketType, TicketTypes}};
use chrono::{DateTime, Local, Utc};
use leptos::ServerFnError;
use macros::generate_new;
use serde::{Deserialize, Serialize};

pub type EventId = GenericId<Event>;
impl TableName for Event {
    const TABLE_NAME: &'static str = "event";
}
#[generate_new]
// #[generate_db]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    pub id: EventId,
    pub name: String,
    pub tagline: String,
    pub default_ticket_type: TicketType,
    pub additional_ticket_types: Vec<TicketType>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct DbEvent {
    pub id: surrealdb::sql::Thing,
    pub name: String,
    pub tagline: String,
    pub default_ticket_type: TicketType,
    pub additional_ticket_types: Vec<TicketType>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[cfg(not(target_arch = "wasm32"))]
impl From<DbEvent> for Event {
    fn from(item: DbEvent) -> Self {
        Self {
            id: item.id.into(),
            name: item.name,
            tagline: item.tagline,
            default_ticket_type: item.default_ticket_type,
            additional_ticket_types: item.additional_ticket_types,
            start: item.start,
            end: item.end,
        }
    }
}

impl Event {
    pub fn start_local(&self) -> DateTime<Local> { self.start.into() }
    pub fn end_local(&self) -> DateTime<Local> { self.end.into() }
    pub fn ticket_types(&self) -> TicketTypes {
        let mut all = Vec::new();
        all.push(self.default_ticket_type.clone());
        all.extend(self.additional_ticket_types.clone());
        all
    }
}

////////////////////////// Functions that run on the server //////////////////////////////////////

#[cfg(not(target_arch = "wasm32"))]
cfg_if::cfg_if! {
if #[cfg(not(target_arch = "wasm32"))] {
    use crate::{db, AppState};
    use leptos::use_context;
}}

#[leptos::server(CreateEvent, "/api", "Url", "create_event")]
pub async fn new_event(e: NewEvent) -> Result<String, ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

    let r: db::Record = app_state
        .db
        .create("event")
        .content(e)
        .await?
        .pop()
        .ok_or(ServerFnError::new("failed to create new event"))?;
    Ok(r.id.to_string())
}

#[leptos::server(ListEvents, "/api", "Url", "list_events")]
pub async fn list_events() -> Result<Vec<Event>, ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

    // TODO - get DbEvent then into Event?
    let events: Vec<Event> = app_state
        .db
        .query("SELECT meta::id(id) as id, * FROM event;")
        .await
        .map_err(|e| ServerFnError::new(format!("db query failed: {e:?}")))?
        .take(0)?;

    Ok(events)
}

#[leptos::server(GetEvent, "/api", "Url", "get_event")]
pub async fn get_event(id: EventId) -> Result<Event, ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerFnError::new("No server state"))?;

    let event: DbEvent = app_state
        .db
        .select(id)
        .await?
        .ok_or(ServerFnError::new("no event found"))?;

    Ok(event.into())
}

////////////////////////// Testy McTest Face //////////////////////////////////////

// #[cfg(test)]
// mod tests {

//     use super::*;
//     use macros::serverfn_test;
//     use rust_decimal::Decimal;
//     use std::collections::HashSet;

//     fn test_event(num: usize) -> NewEvent {
//         NewEvent {
//             name: format!("test event {}", num),
//             additional_ticket_types: vec![],
//             default_ticket_type: TicketType {
//                 name: "foo".to_string(),
//                 price: Decimal::ZERO,
//                 square_item_id: "arse".to_string(),
//                 square_catalog_version: 2,
//             },
//             tagline: "test event".to_string(),
//             start: DateTime::<Utc>::default(),
//             end: DateTime::<Utc>::default(),
//         }
//     }

//     #[serverfn_test]
//     async fn it_works() -> anyhow::Result<()> {
//         let ne1 = test_event(1);
//         let ne2 = test_event(2);

//         let id1 = new_event(ne1.clone()).await.unwrap();
//         let id2 = new_event(ne2.clone()).await.unwrap();

//         let e1 = ne1.to_event(id1.clone());
//         let e2 = ne2.to_event(id2.clone());
//         let events = list_events().await.unwrap();
//         assert_eq!(events.len(), 2);

//         let expected = HashSet::from([e1.clone(), e2.clone()]);
//         let actual: HashSet<Event> = events.into_iter().collect();
//         assert_eq!(expected.len(), 2);
//         assert_eq!(expected, actual);

//         let actual = get_event(id1.clone()).await.unwrap();
//         assert_eq!(e1, actual);
//         Ok(())
//     }
// }

