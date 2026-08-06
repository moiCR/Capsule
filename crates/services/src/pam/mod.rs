use anyhow::{Context, Result};
use pam_sys::{
    PamConversation, PamFlag, PamHandle, PamMessage, PamMessageStyle, PamResponse, PamReturnCode,
    acct_mgmt, authenticate, end, start,
};
use std::ffi::CString;
use std::os::raw::{c_int, c_void};
use std::ptr;

struct PamAuthData<'a> {
    username: &'a str,
    password: &'a str,
}

struct PamHandleGuard {
    handle: *mut PamHandle,
    status: PamReturnCode,
}

impl Drop for PamHandleGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            let _ = end(unsafe { &mut *self.handle }, self.status);
        }
    }
}

extern "C" fn pam_conversation_cb(
    num_msg: c_int,
    msg: *mut *mut PamMessage,
    resp: *mut *mut PamResponse,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 || msg.is_null() || resp.is_null() || appdata_ptr.is_null() {
        return PamReturnCode::CONV_ERR as c_int;
    }

    let auth_data = unsafe { &*(appdata_ptr as *const PamAuthData) };
    let count = num_msg as usize;

    let res_mem =
        unsafe { libc::calloc(count, std::mem::size_of::<PamResponse>()) } as *mut PamResponse;
    if res_mem.is_null() {
        return PamReturnCode::CONV_ERR as c_int;
    }

    let msg_ptrs = msg as *const *const PamMessage;

    for i in 0..count {
        let msg_ptr = unsafe { *msg_ptrs.add(i) };
        if msg_ptr.is_null() {
            unsafe { free_pam_responses(res_mem, i) };
            return PamReturnCode::CONV_ERR as c_int;
        }

        let style_raw = unsafe { (*msg_ptr).msg_style };
        let msg_text = unsafe {
            if (*msg_ptr).msg.is_null() {
                ""
            } else {
                std::ffi::CStr::from_ptr((*msg_ptr).msg)
                    .to_str()
                    .unwrap_or("")
            }
        };

        let style = PamMessageStyle::from(style_raw);
        match style {
            PamMessageStyle::PROMPT_ECHO_OFF => {
                if let Ok(c_pass) = CString::new(auth_data.password) {
                    let dup = unsafe { libc::strdup(c_pass.as_ptr()) };
                    if dup.is_null() {
                        unsafe { free_pam_responses(res_mem, i) };
                        return PamReturnCode::CONV_ERR as c_int;
                    }
                    unsafe { (*res_mem.add(i)).resp = dup };
                } else {
                    unsafe { free_pam_responses(res_mem, i) };
                    return PamReturnCode::CONV_ERR as c_int;
                }
            }
            PamMessageStyle::PROMPT_ECHO_ON => {
                let lower_text = msg_text.to_lowercase();
                let resp_str = if lower_text.contains("user") || lower_text.contains("login") {
                    auth_data.username
                } else {
                    auth_data.password
                };
                if let Ok(c_resp) = CString::new(resp_str) {
                    let dup = unsafe { libc::strdup(c_resp.as_ptr()) };
                    if dup.is_null() {
                        unsafe { free_pam_responses(res_mem, i) };
                        return PamReturnCode::CONV_ERR as c_int;
                    }
                    unsafe { (*res_mem.add(i)).resp = dup };
                } else {
                    unsafe { free_pam_responses(res_mem, i) };
                    return PamReturnCode::CONV_ERR as c_int;
                }
            }
            PamMessageStyle::ERROR_MSG | PamMessageStyle::TEXT_INFO => {
                unsafe { (*res_mem.add(i)).resp = ptr::null_mut() };
            }
        }
    }

    unsafe { *resp = res_mem };
    PamReturnCode::SUCCESS as c_int
}

unsafe fn free_pam_responses(res_mem: *mut PamResponse, count: usize) {
    for j in 0..count {
        let resp_elem = unsafe { res_mem.add(j) };
        if unsafe { !(*resp_elem).resp.is_null() } {
            unsafe { libc::free((*resp_elem).resp as *mut c_void) };
        }
    }
    unsafe { libc::free(res_mem as *mut c_void) };
}

pub struct PamService;

impl PamService {
    pub fn authenticate_current_user(password: &str) -> Result<bool> {
        let username = std::env::var("USER").context("USER environment variable not found")?;
        Self::authenticate(&username, password)
    }

    pub fn authenticate(username: &str, password: &str) -> Result<bool> {
        let auth_data = PamAuthData { username, password };

        let conv = PamConversation {
            conv: Some(pam_conversation_cb),
            data_ptr: &auth_data as *const PamAuthData as *mut c_void,
        };

        let mut candidate_services = Vec::new();
        if let Ok(env_service) = std::env::var("CAPSULE_PAM_SERVICE") {
            candidate_services.push(env_service);
        }
        candidate_services.extend(vec!["capsule".to_string(), "su".to_string()]);

        for service in candidate_services {
            let mut handle: *mut PamHandle = ptr::null_mut();
            let start_res = start(&service, Some(username), &conv, &mut handle);

            if start_res != PamReturnCode::SUCCESS || handle.is_null() {
                eprintln!(
                    "[PAM] Failed to start service '{}': {:?}",
                    service, start_res
                );
                continue;
            }

            let mut guard = PamHandleGuard {
                handle,
                status: PamReturnCode::SUCCESS,
            };

            let auth_res = authenticate(unsafe { &mut *guard.handle }, PamFlag::NONE);
            guard.status = auth_res;

            eprintln!(
                "[PAM] authenticate for '{}' returned: {:?}",
                service, auth_res
            );

            if auth_res == PamReturnCode::SUCCESS {
                let acct_res = acct_mgmt(unsafe { &mut *guard.handle }, PamFlag::NONE);
                guard.status = acct_res;
                eprintln!("[PAM] acct_mgmt for '{}' returned: {:?}", service, acct_res);
                if acct_res == PamReturnCode::SUCCESS || acct_res == PamReturnCode::NEW_AUTHTOK_REQD
                {
                    return Ok(true);
                }
            } else {
                eprintln!(
                    "[PAM] Auth failed for service '{}': {:?}",
                    service, auth_res
                );
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pam_wrong_password() {
        let res = PamService::authenticate_current_user("invalid_password_xyz_123");
        println!("PAM wrong pass result: {:?}", res);
        assert_eq!(res.unwrap(), false);
    }
}
