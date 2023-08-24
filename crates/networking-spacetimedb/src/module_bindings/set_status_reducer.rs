// THIS FILE IS AUTOMATICALLY GENERATED BY SPACETIMEDB. EDITS TO THIS FILE
// WILL NOT BE SAVED. MODIFY TABLES IN RUST INSTEAD.

#[allow(unused)]
use spacetimedb_sdk::{
    anyhow::{anyhow, Result},
    identity::Identity,
    reducer::{Reducer, ReducerCallbackId, Status},
    sats::{de::Deserialize, ser::Serialize},
    spacetimedb_lib,
    table::{TableIter, TableType, TableWithPrimaryKey},
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SetStatusArgs {
    pub status: String,
}

impl Reducer for SetStatusArgs {
    const REDUCER_NAME: &'static str = "set_status";
}

#[allow(unused)]
pub fn set_status(status: String) {
    SetStatusArgs { status }.invoke();
}

#[allow(unused)]
pub fn on_set_status(
    mut __callback: impl FnMut(&Identity, &Status, &String) + Send + 'static,
) -> ReducerCallbackId<SetStatusArgs> {
    SetStatusArgs::on_reducer(move |__identity, __status, __args| {
        let SetStatusArgs { status } = __args;
        __callback(__identity, __status, status);
    })
}

#[allow(unused)]
pub fn once_on_set_status(
    __callback: impl FnOnce(&Identity, &Status, &String) + Send + 'static,
) -> ReducerCallbackId<SetStatusArgs> {
    SetStatusArgs::once_on_reducer(move |__identity, __status, __args| {
        let SetStatusArgs { status } = __args;
        __callback(__identity, __status, status);
    })
}

#[allow(unused)]
pub fn remove_on_set_status(id: ReducerCallbackId<SetStatusArgs>) {
    SetStatusArgs::remove_on_reducer(id);
}