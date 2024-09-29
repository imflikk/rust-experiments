use windows::Win32::NetworkManagement::NetManagement::{NERR_Success, NetUserAdd, NetUserDel, NetUserEnum,FILTER_NORMAL_ACCOUNT, MAX_PREFERRED_LENGTH, USER_INFO_0};

// Can't get this to work from lib.rs for some reason
//use crate::decode_wide_nul_to_string;

// Reference for most of these, but still slightly confusing with Windows API
// https://github.com/secur30nly/netuser-rs/tree/main


pub fn get_local_users() -> Result<Vec<String>, u32> {
    //let servername = std::ptr::null_mut();
    let level = 0; // Return only account names
    let mut buf_ptr = std::ptr::null_mut::<u8>();
    let mut entries_read = 0;
    let mut total_entries = 0;
    let mut resume_handle = 0;

    unsafe {
        let rc = NetUserEnum(
            None,
            level,
            FILTER_NORMAL_ACCOUNT,
            &mut buf_ptr,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            Some(&mut resume_handle),
        );
        if rc != NERR_Success {
            return Err(rc);
        }
    }

    let accounts_slice = unsafe {
        std::slice::from_raw_parts(
            buf_ptr as *const u8 as *const USER_INFO_0,
            entries_read as usize,
        )
    };

    let mut accounts = Vec::<String>::with_capacity(entries_read as usize);
    for account in accounts_slice {
        accounts.push(decode_wide_nul_to_string(account.usri0_name.0).unwrap());
    }

    Ok(accounts)
}


pub fn decode_wide_nul_to_string(
    ptr_wide_string: *mut u16,
) -> Result<String, std::string::FromUtf16Error> {
    let mut decoded_string = Vec::<u16>::new();
    let mut i = 0;
    unsafe {
        while *ptr_wide_string.add(i) != 0 {
            decoded_string.push(*ptr_wide_string.add(i));
            i += 1;
        }
    }
    return String::from_utf16(&decoded_string);
}