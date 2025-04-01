use crate::Name;

#[derive(Clone, Debug, Default)]
pub struct Action<T> {
    /// Name of the account the action is intended for
    pub account: Name,
    /// Name of the action
    pub name: Name,
    /// List of permissions that authorize this action
    pub authorization: Vec<PermissionLevel>,
    /// Payload data
    pub data: T,
}

pub trait ActionFn: Clone {
    /// TODO docs
    const NAME: Name;
    /// TODO docs.
    fn call(self);
}
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct PermissionLevel {
    /// TODO docs
    pub actor: Name,
    /// TODO docs
    pub permission: Name,
}