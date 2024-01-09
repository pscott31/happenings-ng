use indexmap::IndexMap;
use leptos::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ReactiveList<T>(IndexMap<Uuid, RwSignal<T>>)
where
    T: 'static;

impl<T> ReactiveList<T> {
    pub fn new() -> Self { ReactiveList(IndexMap::new()) }
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &RwSignal<T>)> + '_ { self.0.iter() }
}

impl<T> From<ReactiveList<T>> for Vec<T>
where
    T: Clone,
{
    fn from(list: ReactiveList<T>) -> Self { list.0.iter().map(|(_, v)| v.get()).collect() }
}

impl<T> From<Vec<T>> for ReactiveList<T> {
    fn from(list: Vec<T>) -> Self {
        ReactiveList(
            list.into_iter()
                .map(|t| (Uuid::new_v4(), create_rw_signal(t)))
                .collect(),
        )
    }
}

// impl<T> IntoIterator for ReactiveList<T> {
//     type Item = (Uuid, RwSignal<T>);
//     type IntoIter = <IndexMap<uuid::Uuid, leptos::RwSignal<T>> as IntoIterator>::IntoIter;

//     fn into_iter(self) -> Self::IntoIter { self.0.iter() }
// }

pub trait TrackableList<T> {
    fn tracked_push(&self, guest: T);
    fn tracked_remove(&self, uid: Uuid);
    fn tracked_insert(&self, uid: Uuid, new: T);
}

impl<S, T> TrackableList<T> for S
where
    S: SignalUpdate<Value = ReactiveList<T>>,
    T: 'static,
{
    fn tracked_push(&self, guest: T) {
        self.update(|gs| {
            gs.0.insert(Uuid::new_v4(), create_rw_signal::<T>(guest));
        });
    }

    fn tracked_remove(&self, uid: Uuid) {
        self.update(|gs| {
            gs.0.shift_remove(&uid);
        });
    }

    fn tracked_insert(&self, uid: Uuid, new: T) {
        self.update(|gs| {
            gs.0.insert(uid, create_rw_signal::<T>(new));
        });
    }
}

// impl<T> From<ReactiveList<T>> for Vec<T> {
//     fn from(self: TrackableList<T>) -> Self { self.iter().map(|(_, v)| v.get()).collect() }
// }

// impl From<Vec<T>> for TrackableList<T> {
//     fn from(self: Vec<T>) -> Self {
//         self.iter().map(|(_, v)| v.get()).collect()

//         // let mut tickets = ReactiveList::<Ticket>::new();
//         // for ticket in booking().tickets {
//         //     tickets.insert(Uuid::new_v4(), create_rw_signal(ticket));
//         // }
//     }
// }

