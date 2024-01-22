use crate::utils::*;
use leptos::*;
use leptos_icons::{FaIcon, Icon};
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Color {
    #[default]
    Default,
    Primary,
    Secondary,
    Info,
    Success,
    Warning,
    Danger,
}

impl Color {
    fn class(&self) -> &'static str {
        match self {
            Color::Default => "",
            Color::Primary => "is-primary",
            Color::Secondary => "is-secondary",
            Color::Info => "is-info",
            Color::Success => "is-success",
            Color::Warning => "is-warning",
            Color::Danger => "is-danger",
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.write_str(self.class()) }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Size {
    Small,
    #[default]
    Normal,
    Medium,
    Large,
}

impl Size {
    fn class(&self) -> &'static str {
        match self {
            Size::Small => "is-small",
            Size::Normal => "",
            Size::Medium => "is-medium",
            Size::Large => "is-large",
        }
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { f.write_str(self.class()) }
}

#[component]
pub fn IconButton<F>(
    // todo this should work
    // #[prop(into)] on_click: Callback<()>,
    on_click: F,
    #[prop(into, optional)] icon: Option<FaIcon>,
    #[prop(into, optional)] color: Color,
    #[prop(into, optional)] size: Size,
    #[prop(into, optional)] class: OptionalMaybeSignal<String>,
    #[prop(optional)] children: Option<Children>,
    #[prop(into, optional)] disabled: OptionalMaybeSignal<bool>,
    #[prop(into, default = MaybeSignal::Static(false))] loading: MaybeSignal<bool>,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    let icon_view = icon.map(|i| {
        view! {
          <span class=format!("icon {}", size)>
            <Icon icon=Icon::from(i)/>
          </span>
        }
    });

    view! {
      <button
        disabled=move || disabled.or_default().get()
        class=format!("button {} {}", color, class.or_default().get())
        class:is-loading=loading
        on:click=move |_| on_click()
      >
        {icon_view}
        {children.map(|x| view! { <span>{x()}</span> })}
      </button>
    }
}

