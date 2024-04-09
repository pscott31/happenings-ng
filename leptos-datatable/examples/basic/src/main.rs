use leptos::*;
// use polars::{frame::DataFrame, series::Series};
// use polars::prelude::*;
use polars_core::prelude::*;
// use polars_lazy::prelude::*;

#[component]
fn App() -> impl IntoView {
    let s1 = Series::new("Fruit", &["Apple", "Apple", "Pear"]);
    let s2 = Series::new("Color", &["Red", "Yellow", "Green"]);
    let s3 = Series::new("Country", &["UK", "France", "UK"]);

    let df = DataFrame::new(vec![s1, s2, s3]).unwrap();

    let header = df
        .get_column_names()
        .into_iter()
        .map(String::from)
        .map(move |s| view! { <div class="dt-header">{s}</div> })
        .collect_view();

    let rows = df.iter().flatten().map(|row| {
        row.into_iter()
            .map(|s| view! { <div class="dt-data">{s}</div> })
            .collect_view()
    });

    view! {
      <div>
        <h1>"Hello, World!"</h1>
        <p>"Welcome to your new Leptos app bob!"</p>

        <div
          class="dt-container"
          style:grid-template-columns=move || format!("repeat({}, minmax(10px, 1fr))", df.shape().1)
        >
          {header}
          {rows}
        </div>
      </div>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App/> });
}

