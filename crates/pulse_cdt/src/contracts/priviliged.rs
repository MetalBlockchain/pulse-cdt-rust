use core::ops::Deref;

use crate::core::Name;

mod priviliged_impl {
    extern "C" {
        #[link_name = "is_privileged"]
        pub fn is_privileged(account: u64) -> bool;

        #[link_name = "set_privileged"]
        pub fn set_privileged(account: u64, privileged: bool);

        #[link_name = "get_resource_limits"]
        pub fn get_resource_limits(
            account: u64,
            ram_bytes_ptr: *mut i64,
            net_weight_ptr: *mut i64,
            cpu_weight_ptr: *mut i64,
        );

        #[link_name = "set_resource_limits"]
        pub fn set_resource_limits(account: u64, ram_bytes: i64, net_weight: i64, cpu_weight: i64);
    }
}

/// Checks whether the specified account has privileged status.
///
/// # Parameters
///
/// - `account`: The name of the account to check.
///
/// # Returns
///
/// - `true` if the account is privileged.  
/// - `false` if the account is not privileged.
#[inline]
pub fn is_privileged(account: Name) -> bool {
    unsafe { priviliged_impl::is_privileged(account.raw()) }
}

/// Sets the privileged status of an account.
///
/// # Parameters
///
/// - `account`: The name of the account whose privileged status is being set.
/// - `privileged`: A boolean indicating whether the account should be privileged (`true`) or not (`false`).
#[inline]
pub fn set_privileged(account: Name, privileged: bool) {
    unsafe { priviliged_impl::set_privileged(account.raw(), privileged) }
}

/// Retrieves the resource limits for a given account.
///
/// # Parameters
///
/// - `account`: The name of the account whose resource limits are being queried.
#[inline]
pub fn get_resource_limits(account: Name) -> (i64, i64, i64) {
    let mut ram_bytes = 0i64;
    let ram_bytes_ptr: *mut i64 = &mut ram_bytes as *mut i64;
    let mut net_weight = 0i64;
    let net_weight_ptr: *mut i64 = &mut net_weight as *mut i64;
    let mut cpu_weight = 0i64;
    let cpu_weight_ptr: *mut i64 = &mut cpu_weight as *mut i64;

    unsafe {
        priviliged_impl::get_resource_limits(
            account.raw(),
            ram_bytes_ptr,
            net_weight_ptr,
            cpu_weight_ptr,
        )
    }

    (ram_bytes, net_weight, cpu_weight)
}

/// Sets the resource limits for a given account.
///
/// # Parameters
///
/// - `account`: The name of the account whose resource limits are being configured.
/// - `ram_bytes`: The RAM limit in absolute bytes.
/// - `net_weight`: The proportionate share of network resources,
///   calculated as `(weight / total_weight_of_all_accounts)`.
/// - `cpu_weight`: The proportionate share of CPU resources,
///   calculated as `(weight / total_weight_of_all_accounts)`.
#[inline]
pub fn set_resource_limits(account: Name, ram_bytes: i64, net_weight: i64, cpu_weight: i64) {
    unsafe {
        priviliged_impl::set_resource_limits(account.raw(), ram_bytes, net_weight, cpu_weight)
    }
}
