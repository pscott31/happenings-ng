use common::event::SlotDetail;
use common::ticket::Ticket;
use leptos::html::Div;
use leptos::logging::*;
use leptos::*;
use leptos_icons::Icon;
use leptos_use::on_click_outside;
use web_sys::MouseEvent;

use crate::book_event::{ContextEvent, ContextSlotDetails};
use crate::reactive_list::ReactiveList;
use crate::slot_state_for_ticket;

#[component]
fn SlotItem<F>(
    slot_detail: SlotDetail,
    tickets: RwSignal<ReactiveList<Ticket>>,
    ticket: RwSignal<Ticket>,
    on_click: F,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let slot_name = store_value(slot_detail.name);
    let slots = expect_context::<ContextSlotDetails>().0;
    let availability = move || slot_state_for_ticket(slots, slot_name(), tickets(), Some(ticket));
    let label = move || availability().description();
    let can_take = move || availability().can_take();
    let is_active = move || availability().is_in_slot();
    let on_click = store_value(on_click);
    let item_text = move || format!("{} - {}", slot_name(), label());

    view! {
      <Show
        when=can_take
        fallback=move || {
            view! {
              <div class="dropdown-item" class:is-active=is_active>
                {item_text}
              </div>
            }
        }
      >

        <a
          href="#"
          class="dropdown-item"
          class:is-active=is_active
          on:click=move |_ev: MouseEvent| on_click.with_value(|f| f())
        >
          {item_text}
        </a>
      </Show>
    }
}

#[component]
pub fn SlotSelector(
    tickets: RwSignal<ReactiveList<Ticket>>, // All the tickets in the booking being created
    #[prop(into)] ticket: RwSignal<Ticket>,  // The ticket that this slot select is for
    #[prop(into)] get: Signal<Option<String>>,
    #[prop(into)] set: Callback<Option<String>>,
) -> impl IntoView {
    let slots = expect_context::<ContextSlotDetails>().0;
    let event = expect_context::<ContextEvent>().0;

    let description = move || event().slots.description;

    let (active, set_active) = create_signal(false);

    let items = move || {
        slots()
            .into_iter()
            .map(|slot_details| {
                let slot_name = store_value(slot_details.name.clone());
                view! {
                  <SlotItem
                    slot_detail=slot_details
                    tickets=tickets
                    ticket=ticket
                    on_click=move || {
                        set(Some(slot_name()));
                        set_active(false);
                    }
                  />
                }
            })
            .collect_view()
    };

    let target = create_node_ref::<Div>();
    let _ =
        on_click_outside(target, move |event| {
            log!("outside clicky {:?}", event);
            set_active(false);
        });
    let slot_is_some = Signal::derive(move || get().is_some());
    view! {
      <label class="label">{description}</label>
      <div class="dropdown" class:is-active=active>
        <div class="dropdown-trigger" on:click=move |_| set_active(true)>
          <button
            class="button is-outlined"
            class:is-danger=move || !slot_is_some()
            class:is-primary=slot_is_some
            aria-haspopup="true"
            aria-controls="dropdown-menu"
          >
            <span>{move || get().unwrap_or("Select a slot".to_string())}</span>
            <span class="icon is-small">
              <Icon icon=icondata::FaAngleDownSolid/>
            </span>
          </button>
        </div>
        <div class="dropdown-menu" node_ref=target id="dropdown-menu" role="menu">
          <div class="dropdown-content">{items}</div>
        </div>
      </div>
    }
}

