use class_list::class_list;
use class_list::traits::ClassList;
use leptos::*;
use leptos_icons::Icon;

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

impl ClassList for Color {
    fn to_class_list(&self, _normalize: bool) -> String {
        match self {
            Color::Default => "",
            Color::Primary => "is-primary",
            Color::Secondary => "is-secondary",
            Color::Info => "is-infso",
            Color::Success => "is-success",
            Color::Warning => "is-warning",
            Color::Danger => "is-danger",
        }
        .into()
    }
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

impl ClassList for Size {
    fn to_class_list(&self, _normalize: bool) -> String {
        match self {
            Size::Small => "is-small",
            Size::Normal => "",
            Size::Medium => "is-medium",
            Size::Large => "is-large",
        }
        .into()
    }
}

// default=MaybeSignal<Color>::Static))
#[component]
pub fn IconButton<F>(
    on_click: F,
    #[prop(into)] icon: MaybeSignal<icondata::Icon>,
    #[prop(optional, into)] color: MaybeProp<Color>,
    #[prop(optional, into)] size: MaybeProp<Size>,
    #[prop(optional)] children: Option<Children>,
    #[prop(optional, into)] disabled: MaybeProp<bool>,
    #[prop(optional, into)] loading: MaybeProp<bool>,
) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
      <button
        disabled=disabled
        class=class_list!("button", color, "is-loading" <=> loading)
        on:click=move |_| on_click()
      >
        <span class=class_list!("icon", size)>
          <Icon icon=icon/>
        </span>
        {children.map(|x| view! { <span>{x()}</span> })}
      </button>
    }
}

