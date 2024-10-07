use windows::{
    core::{PCWSTR, HRESULT},
    Win32::System::Services::{
        EnumServicesStatusExW, OpenSCManagerW, CloseServiceHandle, SC_ENUM_PROCESS_INFO, SC_MANAGER_ENUMERATE_SERVICE,
        SERVICE_WIN32, SERVICE_STATE_ALL, ENUM_SERVICE_STATUS_PROCESSW, 
    },
};
use std::ptr::null_mut;

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

                for service in services {
                    // TODO: Add checks for service access and check for unquoted service paths
                    let service_name = decode_wide_nul_to_string(service.lpServiceName.0).unwrap();
                    let display_name = decode_wide_nul_to_string(service.lpDisplayName.0).unwrap();
                    println!("Service Name: {}, Display Name: {}", service_name, display_name);
                }
                break;
            }
        }

        // Close the service control manager handle
        CloseServiceHandle(scm_handle);
    }
}
