use pulse::Name;

mod auth_impl {
    extern "C" {
        #[link_name = "require_auth"]
        pub fn require_auth(name: u64);

        #[link_name = "has_auth"]
        pub fn has_auth(name: u64) -> bool;

        #[link_name = "require_recipient"]
        pub fn require_recipient(recipient: u64);

        #[link_name = "is_account"]
        pub fn is_account(name: u64) -> bool;

        #[link_name = "get_self"]
        pub fn get_self() -> u64;

        #[link_name = "current_receiver"]
        pub fn current_receiver() -> u64;
    }
}

#[inline]
pub fn require_auth(name: Name) {
    unsafe { auth_impl::require_auth(name.as_u64()) }
}

#[inline]
pub fn has_auth(name: Name) -> bool {
    unsafe { auth_impl::has_auth(name.as_u64()) }
}

#[inline]
pub fn require_recipient(recipient: Name) {
    unsafe { auth_impl::require_recipient(recipient.as_u64()) }
}

#[inline]
pub fn is_account(recipient: Name) -> bool {
    unsafe { auth_impl::is_account(recipient.as_u64()) }
}

#[inline]
pub fn get_self() -> Name {
    let result = unsafe { auth_impl::get_self() };
    Name::new(result)
}

#[inline]
pub fn current_receiver() -> Name {
    let receiver = unsafe { auth_impl::current_receiver() };
    Name::new(receiver)
}