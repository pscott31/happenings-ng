use async_trait::async_trait;
use common::event::{list_events, Event};
use leptos::*;
use leptos_struct_table::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
struct BulmaTableClasses;

impl TableClassesProvider for BulmaTableClasses {
    fn new() -> Self { Self }
    fn table(&self, classes: &str) -> String { format!("table {}", classes) }
}
#[derive(TableComponent, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[table(
    classes_provider = "BulmaTableClasses",
    sortable = true,
    // row_renderer = "EventRowRenderer"
)]
pub struct EventRow {
    #[table(skip)]
    inner: Event,
    #[table(key, skip)]
    id: FieldGetter<String>,
    name: FieldGetter<String>,
    date: FieldGetter<String>,
    time: FieldGetter<String>,
    #[table(renderer = "ActionRenderer")]
    action: FieldGetter<String>,
}

#[allow(unused_variables)]
#[component]
pub fn ActionRenderer<F>(
    #[prop(into)] class: MaybeSignal<String>,
    #[prop(into)] value: MaybeSignal<String>,
    on_change: F,
    index: usize,
) -> impl IntoView
where
    F: Fn(String) + 'static,
{
    view! {
      <td class=class>
        <a href=format!("/events/{}/book", value()) class="button is-primary">
          Book Now
        </a>
      </td>
    }
}

impl EventRow {
    pub fn id(&self) -> String { self.inner.id.clone().into() }
    pub fn name(&self) -> String { self.inner.name.clone() }
    pub fn date(&self) -> String { self.inner.start_local().format("%d %B %Y").to_string() }
    pub fn time(&self) -> String { self.inner.start_local().format("%-I:%M %p").to_string() }
    pub fn action(&self) -> String { self.inner.id.clone().into() }
}

impl From<Event> for EventRow {
    fn from(u: Event) -> Self {
        EventRow {
            inner: u,
            id: Default::default(),
            date: Default::default(),
            name: Default::default(),
            time: Default::default(),
            action: Default::default(),
        }
    }
}

#[component]
pub fn Events() -> impl IntoView {
    let rows = create_rw_signal::<Vec<EventRow>>(vec![]);
    let _res = create_resource(
        || (),
        move |_| async move {
            let doofers: Vec<EventRow> = list_events()
                .await
                .unwrap_or(vec![])
                .into_iter()
                .map(|u| u.into())
                .collect();

            rows.set(doofers);
        },
    );
    view! {
      <section class="section">
        <div class="container">
          <h1 class="title">Events</h1>
          <EventRowTable items=rows/>
        </div>
      </section>
    }
}

