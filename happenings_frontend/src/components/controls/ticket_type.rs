use happenings::ticket::{TicketType, TicketTypes};
use leptos::logging::*;
use leptos::*;

#[component]
pub fn TicketType(
    #[prop(into)] get: Signal<TicketType>,
    #[prop(into)] set: Callback<TicketType>,
) -> impl IntoView {
    let ticket_types = use_context::<StoredValue<TicketTypes>>().expect("there to be ticket types");

    let options = ticket_types()
        .clone()
        .into_iter()
        .map(|tt| {
            let is_selected = {
                let tt = tt.clone();
                move || tt.name == get().name
            };
            let option_text = format!("{} - £{}", tt.name, tt.price);
            view! {
              <option selected=is_selected value=tt.name>
                {option_text}
              </option>
            }
        })
        .collect_view();

    let on_change = move |ev| {
        log!("{}", event_target_value(&ev));
        ticket_types()
            .into_iter()
            .find(|tt| tt.name == event_target_value(&ev))
            .map(set);
    };

    view! {
      <div class="control">
        <div class="select">
          <select on:change=on_change>{options}</select>
        </div>
      </div>
    }
}

