use leptos::*;

#[derive(Debug, Clone)]
pub struct OptionalMaybeSignal<T: 'static>(Option<MaybeSignal<T>>);

impl<T: Clone> OptionalMaybeSignal<T> {
    // apub fn or<D: Into<MaybeSignal<T>>>(self, default: D) -> MaybeSignal<T> {
    //     match self.0 {
    //         Some(maybe_signal) => maybe_signal,
    //         None => default.into(),
    //     }
    // }

    pub fn or_default(self) -> MaybeSignal<T>
    where
        T: Default,
    {
        match self.0 {
            Some(maybe_signal) => maybe_signal,
            None => MaybeSignal::Static(T::default()),
        }
    }

    // pub fn map<U: 'static, F: Fn(T) -> U + 'static>(self, map: F) -> OptionalMaybeSignal<U> {
    //     match self.0 {
    //         Some(maybe_signal) => match maybe_signal {
    //             MaybeSignal::Static(v) => MaybeSignal::Static(map(v)).into(),
    //             MaybeSignal::Dynamic(sig) => {
    //                 MaybeSignal::Dynamic(Signal::derive(move || map(sig.get()))).into()
    //             }
    //         },
    //         None => OptionalMaybeSignal(None),
    //     }
    // }
}

impl<T: Copy> Copy for OptionalMaybeSignal<T> {}

impl<T> Default for OptionalMaybeSignal<T> {
    fn default() -> Self { Self(None) }
}

impl<T: 'static, I: Into<MaybeSignal<T>>> From<I> for OptionalMaybeSignal<T> {
    fn from(value: I) -> Self { Self(Some(value.into())) }
}

impl<T: Clone + Default> SignalGet for OptionalMaybeSignal<T> {
    type Value = T;

    fn get(&self) -> T {
        match &self.0 {
            Some(signal) => signal.get(),
            None => T::default(),
        }
    }

    fn try_get(&self) -> Option<T> {
        match &self.0 {
            Some(signal) => signal.try_get(),
            None => Some(T::default()),
        }
    }
}

impl<T: Clone + Default> SignalGetUntracked for OptionalMaybeSignal<T> {
    type Value = T;

    fn get_untracked(&self) -> T {
        match &self.0 {
            Some(signal) => signal.get_untracked(),
            None => T::default(),
        }
    }

    fn try_get_untracked(&self) -> Option<T> {
        match &self.0 {
            Some(signal) => signal.try_get_untracked(),
            None => Some(T::default()),
        }
    }
}

impl<T: IntoAttribute + Clone> IntoAttribute for OptionalMaybeSignal<T> {
    fn into_attribute(self) -> Attribute {
        match self.0 {
            Some(t) => t.into_attribute(), // Requires T to be Clone!
            None => Attribute::Option(None),
        }
    }

    fn into_attribute_boxed(self: Box<Self>) -> Attribute {
        match self.0 {
            Some(t) => t.into_attribute(), // Requires T to be Clone!
            None => Attribute::Option(None),
        }
    }
}

pub trait MaybeSignalExt<T: 'static> {
    fn map<U: 'static, F: Fn(T) -> U + 'static>(self, mapper: F) -> MaybeSignal<U>;
}

impl<T: Clone + 'static> MaybeSignalExt<T> for MaybeSignal<T> {
    fn map<U: 'static, F: Fn(T) -> U + 'static>(self, mapper: F) -> MaybeSignal<U> {
        match self {
            MaybeSignal::Static(v) => MaybeSignal::Static(mapper(v)),
            MaybeSignal::Dynamic(sig) => {
                MaybeSignal::Dynamic(Signal::derive(move || mapper(sig.get())))
            }
        }
    }
}

