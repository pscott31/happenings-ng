use leptos::{IntoView, ServerFnError, View};
use leptos_macro::view;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError {
    message: String,
}

impl std::error::Error for AppError {}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{}", self.message) }
}

impl IntoView for AppError {
    fn into_view(self) -> View {
        view! {
          <div class="columns is-vcentered">
            <div class="column has-text-centered">
              <img
                class="block"
                src="/static/sadcat.jpeg"
                style="
                height: 20rem;
                "
              />
              <div class="block">error: {self.message.to_string()}</div>
            </div>
          </div>
        }
        .into_view()
    }
}

// impl<E> From<E> for AppError
// where
//     E: Into<String>,
// {
//     fn from(err: E) -> Self {
//         Self {
//             message: err.into(),
//         }
//     }
// }

impl From<ServerFnError> for AppError {
    fn from(err: ServerFnError) -> Self {
        Self {
            message: format!("{:?}", err),
        }
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self { Self { message: err } }
}

