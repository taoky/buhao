use log::{info, warn};
use redhook::hook;

use crate::{manager::ShadowFd, set_errno_code};

hook! {
    unsafe fn read(fd: i32, buf: *mut libc::c_void, count: usize) -> usize => my_read {
        if fd < crate::LOWER_FD_BOUND {
            return redhook::real!(read)(fd, buf, count)
        }
        let info: ShadowFd = match retrieve_fd_and_open_real!(fd as u64) {
            Some(info) => info,
            None => {
                warn!("read: invalid fd");
                set_errno_code(libc::EBADF);
                return usize::MAX;
            }
        };
        info!("read: {}, {}, {:?}", fd, info.path, info.real_fd);
        redhook::real!(read)(info.real_fd.unwrap(), buf, count)
        // redhook::real!(read)(fd, buf, count)
    }
}
