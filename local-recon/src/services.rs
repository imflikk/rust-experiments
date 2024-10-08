use windows::{
    core::{PCWSTR, HRESULT},
    Win32::System::Services::{
        EnumServicesStatusExW, OpenSCManagerW, CloseServiceHandle, OpenServiceW, QueryServiceConfigW, SC_ENUM_PROCESS_INFO, SC_MANAGER_ENUMERATE_SERVICE,
        SERVICE_WIN32, SERVICE_STATE_ALL, ENUM_SERVICE_STATUS_PROCESSW, SERVICE_CHANGE_CONFIG, SERVICE_QUERY_CONFIG, QUERY_SERVICE_CONFIGW,
    },
};

use colored::Colorize;

// This is the same function as in the other files.  Can't get it loaded from lib.rs for some reason
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

pub fn get_services() {
    unsafe {
        // Open the service control manager
        let scm_handle = OpenSCManagerW(
            PCWSTR::null(),
            PCWSTR::null(),
            SC_MANAGER_ENUMERATE_SERVICE,
        ).expect("Failed to open the service control manager");

        // Prepare to enumerate services
        let mut bytes_needed = 0;
        let mut services_returned = 0;
        let mut resume_handle = 0;
        let mut buffer_size = 0;
        let mut buffer = Vec::new();

        loop {
            // First call to get the required buffer size
            let result = EnumServicesStatusExW(
                scm_handle,
                SC_ENUM_PROCESS_INFO,
                SERVICE_WIN32,
                SERVICE_STATE_ALL,
                if buffer_size > 0 { Some(&mut buffer) } else { None },
                &mut bytes_needed,
                &mut services_returned,
                Some(&mut resume_handle), // This expects an Option type, so it needs to be wrapped in Some()
                PCWSTR::null(),
            );

            if result.is_err() {
                let error = result.unwrap_err();
                // Check for the error "More data is available", if so reallocate buffer
                // This errors when not doing this check
                if error.code() == HRESULT(0x800700EAu32 as i32) {
                    // More data is available, reallocate buffer
                    buffer_size = bytes_needed as usize;
                    buffer.resize(buffer_size, 0);
                } else {
                    eprintln!("Failed to enumerate services: {:?}", error);
                    break;
                }
            } else {
                // Successfully retrieved data
                let services = std::slice::from_raw_parts(
                    buffer.as_ptr() as *const ENUM_SERVICE_STATUS_PROCESSW,
                    services_returned as usize,
                );

                println!("{:<40} | {:<70} | {:<10} | {:<6} | {}", "Service Name", "Display Name", "Can Modify", "Unquoted", "Binary Path");
                println!("{}", "-".repeat(160));
                for service in services {
                    let service_name = decode_wide_nul_to_string(service.lpServiceName.0).unwrap();
                    let display_name = decode_wide_nul_to_string(service.lpDisplayName.0).unwrap();
                    
                    // Check if the current user has access to modify the service
                    let service_handle_result = OpenServiceW(
                        scm_handle,
                        PCWSTR(service.lpServiceName.0),
                        SERVICE_CHANGE_CONFIG,
                    );

                    let can_modify = match service_handle_result {
                        Ok(service_handle) => {
                            let _ = CloseServiceHandle(service_handle);
                            true
                        }
                        Err(_) => false,
                    };

                    // Open the service to query its configuration
                    let service_handle_result = OpenServiceW(
                        scm_handle,
                        PCWSTR(service.lpServiceName.0),
                        SERVICE_QUERY_CONFIG,
                    );

                    let bin_path = match service_handle_result {
                        Ok(service_handle) => {
                            // Query the service configuration
                            let mut bytes_needed = 0;
                            let _ = QueryServiceConfigW(service_handle, None, 0, &mut bytes_needed);
                            let mut config_buffer = vec![0u8; bytes_needed as usize];
                            let success = QueryServiceConfigW(
                                service_handle,
                                Some(config_buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW),
                                bytes_needed,
                                &mut bytes_needed,
                            );

                            let bin_path = if success.is_ok() {
                                let config = &*(config_buffer.as_ptr() as *const QUERY_SERVICE_CONFIGW);
                                decode_wide_nul_to_string(config.lpBinaryPathName.0).unwrap()
                            } else {
                                "Unknown".to_string()
                            };
                            let _ = CloseServiceHandle(service_handle);
                            bin_path
                        }
                        Err(_) => "Unknown".to_string(),
                    };

                    let mut bin_path_unquoted = false;
                    // Check for unquoted service paths meeting the following criteria
                    // 1. The path does not contain quotes
                    // 2. The path contains a space
                    // 3. The space does not come after the first appearance of .exe
                    if !bin_path.contains('"') && bin_path.contains(' ') {
                        let exe_index = bin_path.to_lowercase().find(".exe");
                        let space_index = bin_path.find(' ');

                        // If space comes before .exe, then it's part of the initiating path
                        if space_index < exe_index {
                            bin_path_unquoted = true;
                        }
                    }
                    
                    if can_modify || bin_path_unquoted {
                        println!("{:<40} | {:<70} | {:<10} | {:<6} | {}", service_name.green(), display_name.green(), can_modify.to_string().green(), bin_path_unquoted.to_string().green(), bin_path.green());
                    } else {
                        println!("{:<40} | {:<70} | {:<10} | {:<6} | {}", service_name, display_name, can_modify, bin_path_unquoted, bin_path);
                    }
                }
                break;
            }
        }

        // Close the service control manager handle
        let _ = CloseServiceHandle(scm_handle);
    }
}
