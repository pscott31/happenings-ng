use crate::ticket::{TicketType, TicketTypes};
use chrono::{DateTime, Local, Utc};
use derive_builder::Builder;
use happenings_macro::{generate_db, generate_new};
use leptos::ServerFnError;
use serde::{Deserialize, Serialize};

#[generate_new]
#[generate_db]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Builder)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub tagline: String,
    pub default_ticket_type: TicketType,
    pub additional_ticket_types: Vec<TicketType>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
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
    use leptos::ServerFnError::ServerError;
}}

#[leptos::server(CreateEvent, "/api", "Url", "create_event")]
pub async fn new_event(e: NewEvent) -> Result<String, ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

    let r: db::Record = app_state
        .db
        .create("event")
        .content(e)
        .await?
        .pop()
        .ok_or(ServerError("failed to create new event".to_string()))?;
    return Ok(r.id.to_string());
}

#[leptos::server(ListEvents, "/api", "Url", "list_events")]
pub async fn list_events() -> Result<Vec<Event>, leptos::ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

    let events: Vec<Event> = app_state
        .db
        .query("SELECT type::string(id) as id, * FROM event;")
        .await
        .map_err(|_| ServerError("db query failed".to_string()))?
        .take(0)?;

    return Ok(events);
}

#[leptos::server(GetEvent, "/api", "Url", "get_event")]
pub async fn get_event(id: String) -> Result<Event, leptos::ServerFnError> {
    let app_state = use_context::<AppState>().ok_or(ServerError("No server state".to_string()))?;

    let thing =
        surrealdb::sql::thing(id.as_ref()).map_err(|_| ServerError("Bad id".to_string()))?;

    let mut events: Vec<Event> = app_state
        .db
        .query("SELECT type::string(id) as id, * FROM event where id=$req_id;")
        .bind(("req_id", thing))
        .await
        .map_err(|_| ServerError("db query failed".to_string()))?
        .take(0)?;

    let event = events
        .pop()
        .ok_or(ServerError("no event found".to_string()))?;

    return Ok(event);
}

////////////////////////// Testy McTest Face //////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use happenings_macro::serverfn_test;
    use std::collections::HashSet;

    fn test_event(num: usize) -> NewEvent {
        NewEventBuilder::default()
            .name(format!("test event {}", num))
            .build()
            .unwrap()
    }

    #[serverfn_test]
    async fn it_works() -> anyhow::Result<()> {
        let ne1 = test_event(1);
        let ne2 = test_event(2);

        let id1 = new_event(ne1.clone()).await.unwrap();
        let id2 = new_event(ne2.clone()).await.unwrap();

        let e1 = ne1.to_event(id1.clone());
        let e2 = ne2.to_event(id2.clone());
        let events = list_events().await.unwrap();
        assert_eq!(events.len(), 2);

        let expected = HashSet::from([e1.clone(), e2.clone()]);
        let actual: HashSet<Event> = events.into_iter().collect();
        assert_eq!(expected.len(), 2);
        assert_eq!(expected, actual);

        let actual = get_event(id1.clone()).await.unwrap();
        assert_eq!(e1, actual);
        Ok(())
    }
}

