extern crate libc;
extern crate soroban_env_host;

use soroban_env_host::budget::Budget;
use soroban_env_host::storage::{self, AccessType, SnapshotSource, Storage};
use soroban_env_host::xdr::ScUnknownErrorCode::{General, Xdr};
use soroban_env_host::xdr::{
    self, AccountId, HostFunction, LedgerEntry, LedgerKey, ReadXdr, ScHostStorageErrorCode,
    ScStatus, WriteXdr,
};
use soroban_env_host::{
    auth::RecordedAuthPayload, 
    Host, HostError, LedgerInfo
};
use std::convert::{TryFrom, TryInto};
use std::error;
use std::ffi::{CStr, CString};
use std::panic;
use std::ptr::null_mut;
use std::rc::Rc;
use std::slice;
use xdr::LedgerFootprint;

// TODO: we may want to pass callbacks instead of using global functions
extern "C" {
    // LedgerKey XDR in base64 string to LedgerEntry XDR in base64 string
    fn SnapshotSourceGet(
        handle: libc::uintptr_t,
        ledger_key: *const libc::c_char,
    ) -> *const libc::c_char;
    // LedgerKey XDR in base64 string to bool
    fn SnapshotSourceHas(handle: libc::uintptr_t, ledger_key: *const libc::c_char) -> libc::c_int;
    // Free Strings returned from Go functions
    fn FreeGoCString(str: *const libc::c_char);
}

struct CSnapshotSource {
    handle: libc::uintptr_t,
}

impl SnapshotSource for CSnapshotSource {
    fn get(&self, key: &LedgerKey) -> Result<LedgerEntry, HostError> {
        let key_xdr = key
            .to_xdr_base64()
            .map_err(|_| ScStatus::UnknownError(Xdr))?;
        let key_cstr = CString::new(key_xdr).map_err(|_| ScStatus::UnknownError(General))?;
        let res = unsafe { SnapshotSourceGet(self.handle, key_cstr.as_ptr()) };
        if res.is_null() {
            return Err(HostError::from(
                ScHostStorageErrorCode::AccessToUnknownEntry,
            ));
        }
        let res_cstr = unsafe { CStr::from_ptr(res) };
        let res_str = res_cstr
            .to_str()
            .map_err(|_| ScStatus::UnknownError(General))?;
        let entry =
            LedgerEntry::from_xdr_base64(res_str).map_err(|_| ScStatus::UnknownError(Xdr))?;
        unsafe { FreeGoCString(res) };
        Ok(entry)
    }

    fn has(&self, key: &LedgerKey) -> Result<bool, HostError> {
        let key_xdr = key
            .to_xdr_base64()
            .map_err(|_| ScStatus::UnknownError(Xdr))?;
        let key_cstr = CString::new(key_xdr).map_err(|_| ScStatus::UnknownError(Xdr))?;
        let res = unsafe { SnapshotSourceHas(self.handle, key_cstr.as_ptr()) };
        Ok(match res {
            0 => false,
            _ => true,
        })
    }
}

#[repr(C)]
pub struct CLedgerInfo {
    pub protocol_version: u32,
    pub sequence_number: u32,
    pub timestamp: u64,
    pub network_id: *mut u8,
    pub base_reserve: u32,
}

impl From<CLedgerInfo> for LedgerInfo {
    fn from(c: CLedgerInfo) -> Self {
        let network_id = unsafe { slice::from_raw_parts(c.network_id, 32) };
        Self {
            protocol_version: c.protocol_version,
            sequence_number: c.sequence_number,
            timestamp: c.timestamp,
            network_id: network_id.try_into().unwrap(),
            base_reserve: c.base_reserve,
        }
    }
}

fn storage_footprint_to_ledger_footprint(
    foot: &storage::Footprint,
) -> Result<LedgerFootprint, xdr::Error> {
    let mut read_only: Vec<LedgerKey> = Vec::new();
    let mut read_write: Vec<LedgerKey> = Vec::new();
    for (k, v) in &foot.0 {
        match v {
            AccessType::ReadOnly => read_only.push((**k).clone()),
            AccessType::ReadWrite => read_write.push((**k).clone()),
        }
    }
    Ok(LedgerFootprint {
        read_only: read_only.try_into()?,
        read_write: read_write.try_into()?,
    })
}

#[repr(C)]
pub struct CPreflightResult {
    pub error: *mut libc::c_char, // Error string in case of error, otherwise null
    pub result: *mut libc::c_char, // SCVal XDR in base64
    pub footprint: *mut libc::c_char, // LedgerFootprint XDR in base64
    pub auth_ptr: *const CRecordedAuthPayload, // Auth payloads
    pub auth_len: usize,
    pub auth_cap: usize,
    pub cpu_instructions: u64,
    pub memory_bytes: u64,
}

#[repr(C)]
pub struct CRecordedAuthPayload {
    pub address: Option<*mut libc::c_char>, // Option<Address> XDR in base64
    pub nonce: Option<u64>,
    pub invocation: *mut libc::c_char, // xdr::AuthorizedInvocation XDR in base64
}

fn preflight_error(str: String) -> *mut CPreflightResult {
    let c_str = CString::new(str).unwrap();
    // transfer ownership to caller
    // caller needs to invoke free_preflight_result(result) when done
    Box::into_raw(Box::new(CPreflightResult {
        error: c_str.into_raw(),
        result: null_mut(),
        footprint: null_mut(),
        auth_ptr: null_mut(),
        auth_len: 0,
        auth_cap: 0,
        cpu_instructions: 0,
        memory_bytes: 0,
    }))
}

#[no_mangle]
pub extern "C" fn preflight_host_function(
    handle: libc::uintptr_t, // Go Handle to forward to SnapshotSourceGet and SnapshotSourceHasconst
    hf: *const libc::c_char, // HostFunction XDR in base64
    source_account: *const libc::c_char, // AccountId XDR in base64
    ledger_info: CLedgerInfo,
) -> *mut CPreflightResult {
    // catch panics before they reach foreign callers (which otherwise would result in
    // undefined behavior)
    let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        preflight_host_function_or_maybe_panic(handle, hf, source_account, ledger_info)
    }));
    match res {
        Err(panic) => match panic.downcast::<String>() {
            Ok(panic_msg) => preflight_error(format!(
                "panic during preflight_host_function() call: {}",
                panic_msg
            )),
            Err(_) => preflight_error(
                "panic during preflight_host_function() call: unknown cause".to_string(),
            ),
        },
        // transfer ownership to caller
        // caller needs to invoke free_preflight_result(result) when done
        Ok(r) => match r {
            Ok(r2) => Box::into_raw(Box::new(r2)),
            Err(e) => preflight_error(format!("{}", e)),
        },
    }
}

fn preflight_host_function_or_maybe_panic(
    handle: libc::uintptr_t, // Go Handle to forward to SnapshotSourceGet and SnapshotSourceHasconst
    hf: *const libc::c_char, // HostFunction XDR in base64
    source_account: *const libc::c_char, // AccountId XDR in base64
    ledger_info: CLedgerInfo,
) -> Result<CPreflightResult, Box<dyn error::Error>> {
    let hf_cstr = unsafe { CStr::from_ptr(hf) };
    let hf = HostFunction::from_xdr_base64(hf_cstr.to_str()?)?;
    let source_account_cstr = unsafe { CStr::from_ptr(source_account) };
    let source_account = AccountId::from_xdr_base64(source_account_cstr.to_str()?)?;
    let src = Rc::new(CSnapshotSource { handle });
    let storage = Storage::with_recording_footprint(src);
    let budget = Budget::default();
    let host = Host::with_storage_and_budget(storage, budget);

    host.set_source_account(source_account);
    host.set_ledger_info(ledger_info.into());
    host.switch_to_recording_auth();

    // Run the preflight.
    let result = host.invoke_function(hf)?;

    let auth: Vec<CRecordedAuthPayload> = host
        .get_recorded_auth_payloads()?
        .iter()
        .map(|a| a.try_into())
        .collect::<Result<Vec<CRecordedAuthPayload>, Box<dyn error::Error>>>()?;
    let (auth_ptr, auth_len, auth_cap) = (auth.as_ptr(), auth.len(), auth.capacity());

    // Recover, convert and return the storage footprint and other values to C.
    let (storage, budget, _) = host.try_finish().unwrap();

    let fp = storage_footprint_to_ledger_footprint(&storage.footprint)?;
    let fp_cstr = CString::new(fp.to_xdr_base64()?)?;
    let result_cstr = CString::new(result.to_xdr_base64()?)?;

    Ok(CPreflightResult {
        error: null_mut(),
        result: result_cstr.into_raw(),
        footprint: fp_cstr.into_raw(),
        auth_ptr,
        auth_len,
        auth_cap,
        cpu_instructions: budget.get_cpu_insns_count(),
        memory_bytes: budget.get_mem_bytes_count(),
    })
}

impl TryFrom<&RecordedAuthPayload> for CRecordedAuthPayload {
    type Error = Box<dyn error::Error>;

    fn try_from(p: &RecordedAuthPayload) -> Result<Self, Self::Error> {
        let address = match &p.address {
            None => None,
            Some(a) => Some(CString::new(a.to_xdr_base64()?)?.into_raw())
        };
        Ok(Self {
            address,
            nonce: p.nonce,
            invocation: CString::new(p.invocation.to_xdr_base64()?)?.into_raw(),
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_preflight_result(result: *mut CPreflightResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        if !(*result).error.is_null() {
            let _ = CString::from_raw((*result).error);
        }
        if !(*result).result.is_null() {
            let _ = CString::from_raw((*result).result);
        }
        if !(*result).footprint.is_null() {
            let _ = CString::from_raw((*result).footprint);
        }
        let _ = Box::from_raw(result);
    }
}
