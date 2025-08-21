use pulse_serialization::Write;

use crate::core::{Checksum160, Checksum256, Checksum512, PublicKey, Signature};

mod action_impl {
    extern "C" {
        #[link_name = "assert_sha1"]
        pub fn assert_sha1(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void);

        #[link_name = "assert_ripemd160"]
        pub fn assert_ripemd160(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void);

        #[link_name = "assert_sha256"]
        pub fn assert_sha256(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void);

        #[link_name = "assert_sha512"]
        pub fn assert_sha512(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void);

        #[link_name = "sha1"]
        pub fn sha1(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void) -> u32;

        #[link_name = "ripemd160"]
        pub fn ripemd160(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void) -> u32;

        #[link_name = "sha256"]
        pub fn sha256(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void) -> u32;

        #[link_name = "sha512"]
        pub fn sha512(msg: *mut crate::c_void, len: u32, ptr: *mut crate::c_void) -> u32;

        #[link_name = "recover_key"]
        pub fn recover_key(
            digest: *mut crate::c_void,
            sig: *mut crate::c_void,
            pubkey: *mut crate::c_void,
        ) -> usize;

        #[link_name = "assert_recover_key"]
        pub fn assert_recover_key(
            digest: *mut crate::c_void,
            sig: *mut crate::c_void,
            pubkey: *mut crate::c_void,
        ) -> usize;
    }
}

#[inline]
pub fn assert_sha1(msg: &[u8], len: u32, hash: Checksum160) {
    unsafe { action_impl::assert_sha1(msg.as_ptr() as *mut _, len, hash.0.as_ptr() as *mut _) };
}

#[inline]
pub fn assert_ripemd160(msg: &[u8], len: u32, hash: Checksum160) {
    unsafe {
        action_impl::assert_ripemd160(msg.as_ptr() as *mut _, len, hash.0.as_ptr() as *mut _)
    };
}

#[inline]
pub fn assert_sha256(msg: &[u8], len: u32, hash: Checksum256) {
    unsafe { action_impl::assert_sha256(msg.as_ptr() as *mut _, len, hash.0.as_ptr() as *mut _) };
}

#[inline]
pub fn assert_sha512(msg: &[u8], len: u32, hash: Checksum512) {
    unsafe { action_impl::assert_sha512(msg.as_ptr() as *mut _, len, hash.0.as_ptr() as *mut _) };
}

#[inline]
pub fn sha1(msg: &[u8], len: u32) -> Checksum160 {
    let mut hash = Checksum160::default();
    unsafe { action_impl::sha1(msg.as_ptr() as *mut _, len, &mut hash as *mut _ as *mut _) };
    hash
}

#[inline]
pub fn ripemd160(msg: &[u8], len: u32) -> Checksum160 {
    let mut hash = Checksum160::default();
    unsafe { action_impl::ripemd160(msg.as_ptr() as *mut _, len, &mut hash as *mut _ as *mut _) };
    hash
}

#[inline]
pub fn sha256(msg: &[u8], len: u32) -> Checksum256 {
    let mut hash = Checksum256::default();
    unsafe { action_impl::sha256(msg.as_ptr() as *mut _, len, &mut hash as *mut _ as *mut _) };
    hash
}

#[inline]
pub fn sha512(msg: &[u8], len: u32) -> Checksum512 {
    let mut hash = Checksum512::default();
    unsafe { action_impl::sha512(msg.as_ptr() as *mut _, len, &mut hash as *mut _ as *mut _) };
    hash
}

#[inline]
pub fn recover_key(digest: &Checksum256, sig: &Signature) -> PublicKey {
    let sig_data = sig.pack().expect("failed to serialize signature");
    let pubkey_data = &[0u8; 33];
    unsafe {
        action_impl::recover_key(
            digest.0.as_ptr() as *mut _,
            sig_data.as_ptr() as *mut _,
            pubkey_data.as_ptr() as *mut _,
        )
    };

    return PublicKey::new(&pubkey_data[..33]);
}

#[inline]
pub fn assert_recover_key(digest: &Checksum256, sig: &Signature, pubkey: &PublicKey) {
    let sig_data = sig.pack().expect("failed to serialize signature");
    let pubkey_data = pubkey.pack().expect("failed to serialize public key");
    unsafe {
        action_impl::assert_recover_key(
            digest.0.as_ptr() as *mut _,
            sig_data.as_ptr() as *mut _,
            pubkey_data.as_ptr() as *mut _,
        )
    };
}
