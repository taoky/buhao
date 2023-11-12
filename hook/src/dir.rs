use log::info;
use redhook::hook;

hook! {
    unsafe fn readdir(dirp: *mut libc::DIR) -> *mut libc::dirent => my_readdir {
        let entry = redhook::real!(readdir)(dirp);
        if entry.is_null() {
            return entry;
        }
        let name = std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()).to_str().unwrap();
        info!("readdir: {}", name);
        entry
    }
}

hook! {
    unsafe fn readdir64(dirp: *mut libc::DIR) -> *mut libc::dirent64 => my_readdir64 {
        let entry = redhook::real!(readdir64)(dirp);
        if entry.is_null() {
            return entry;
        }
        let name = std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()).to_str().unwrap();
        info!("readdir64: {}", name);
        entry
    }
}
