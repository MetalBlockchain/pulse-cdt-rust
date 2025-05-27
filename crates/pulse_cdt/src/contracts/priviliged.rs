use crate::core::name::Name;

mod priviliged_impl {
    extern "C" {
        #[link_name = "is_privileged"]
        pub fn is_privileged(account: u64) -> bool;

        #[link_name = "set_privileged"]
        pub fn set_privileged(account: u64, privileged: bool);

        #[link_name = "get_resource_limits"]
        pub fn get_resource_limits(account: u64, ram_bytes_ptr: i64, net_weight_ptr: i64, cpu_weight: i64);
    }
}

/**
 *  Check if an account is privileged
 *
 *  @ingroup privileged
 *  @param account - name of the account to be checked
 *  @return true if the account is privileged
 *  @return false if the account is not privileged
 */
#[inline]
pub fn is_privileged(account: Name) -> bool {
    unsafe { priviliged_impl::is_privileged(account.as_u64()) }
}

/**
 *  Set the privileged status of an account
 *
 *  @ingroup privileged
 *  @param account - name of the account whose privileged account to be set
 *  @param privileged - privileged status
 */
#[inline]
pub fn set_privileged(account: Name, privileged: bool) {
    unsafe { priviliged_impl::set_privileged(account.as_u64(), privileged) }
}
