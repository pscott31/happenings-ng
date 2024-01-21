use leptos::*;

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
      <section class="section is-medium">
      <div class="container">
        <div class="columns is-vcentered">
          <div class="column has-text-centered">
            <h1 class="title">404 Page Not Found</h1>
            <p class="subtitle">An unexpected error has occurred, sorry about that!</p>
            <a class="button" href="/">Home</a>
          </div>
          <div class="column has-text-centered">
            <img src="/static/sad-panda.jpg" />
          </div>
        </div>
      </div>
    </section>
    }
}
