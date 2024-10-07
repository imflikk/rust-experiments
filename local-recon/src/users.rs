use std::ptr::null_mut;
use windows::core::PCWSTR;
use windows::Win32::NetworkManagement::NetManagement::{NERR_Success, NetUserEnum, NetUserGetLocalGroups, NetLocalGroupGetMembers, 
    FILTER_NORMAL_ACCOUNT, MAX_PREFERRED_LENGTH, USER_INFO_0, LG_INCLUDE_INDIRECT, LOCALGROUP_USERS_INFO_0,
    LOCALGROUP_MEMBERS_INFO_2,
};

// Can't get this to work from lib.rs for some reason
//use crate::decode_wide_nul_to_string;

// Reference for most of these, but still slightly confusing with Windows API
// https://github.com/secur30nly/netuser-rs/tree/main


fn encode_string_to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn decode_wide_nul_to_string(ptr_wide_string: *mut u16) -> Result<String, std::string::FromUtf16Error> {
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

pub fn get_user_groups(username: &str) -> Result<Vec<String>, u32> {
    let wide_username_nul = encode_string_to_wide(username);
    let mut buffer = null_mut();
    let mut entries_read = 0;
    let mut total_entries = 0;
    let rc;

    let local_groups_slice = unsafe {
        rc = NetUserGetLocalGroups(
            None,
            PCWSTR::from_raw(wide_username_nul.as_ptr()),
            0,
            LG_INCLUDE_INDIRECT, // the function also returns the names of the local groups in which the user is indirectly a member
            &mut buffer,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
        );
        if rc != NERR_Success {
            //println!("NetUserGetLocalGroups failed with error code: {}", rc);
            return Err(rc);
        }

        // Check if there are no local groups, otherwise it will panic at an empty slice
        if entries_read == 0 {
            return Ok(Vec::new());
        } else {
            std::slice::from_raw_parts(
                buffer as *const u8 as *const LOCALGROUP_USERS_INFO_0,
                entries_read as usize,
            )
        }
        
    };

    let mut local_groups = Vec::<String>::with_capacity(local_groups_slice.len());
    for group in local_groups_slice {
        local_groups.push(decode_wide_nul_to_string(group.lgrui0_name.0).unwrap());
    }

    Ok(local_groups)
}

pub fn get_members_of_group(group_name: &str) -> Result<Vec<String>, u32> {
    let wide_group_name_nul = encode_string_to_wide(group_name);
    let mut buffer = null_mut();
    let mut entries_read = 0;
    let mut total_entries = 0;
    let rc;

    let members_slice = unsafe {
        rc = NetLocalGroupGetMembers(
            None,
            PCWSTR::from_raw(wide_group_name_nul.as_ptr()),
            2, // This needs to be 2 instead of 0 to get domain names along with SID
            &mut buffer,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            Some(null_mut()),
        );
        if rc != NERR_Success {
            return Err(rc);
        }

        if entries_read == 0 {
            return Ok(Vec::new());
        } else {
            std::slice::from_raw_parts(
                buffer as *const u8 as *const LOCALGROUP_MEMBERS_INFO_2,
                entries_read as usize,
            )
        }
    };

    let mut members = Vec::<String>::with_capacity(members_slice.len());
    for member in members_slice {
        
        let name = decode_wide_nul_to_string(member.lgrmi2_domainandname.0);
        match name {
            Ok(n) => members.push(n),
            Err(_) => members.push("Error decoding name".to_string()),
        }
    }

    Ok(members)
}


