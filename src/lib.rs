use std::os::raw::{c_int, c_uint, c_long, c_ushort, c_void, c_char};

pub const PORT_SOURCE_AIO: c_int =      1;
pub const PORT_SOURCE_TIMER: c_int =    2;
pub const PORT_SOURCE_USER: c_int =     3;
pub const PORT_SOURCE_FD: c_int =       4;
pub const PORT_SOURCE_ALERT: c_int =    5;
pub const PORT_SOURCE_MQ: c_int =       6;
pub const PORT_SOURCE_FILE: c_int =     7;

pub const PORT_ALERT_SET: c_int =       0x01;
pub const PORT_ALERT_UPDATE: c_int =    0x02;
pub const PORT_ALERT_INVALID: c_int =   (PORT_ALERT_SET | PORT_ALERT_UPDATE);

/*
 * User watchable file events
 */
pub const FILE_ACCESS: c_int =          0x0000_0001;
pub const FILE_MODIFIED: c_int =        0x0000_0002;
pub const FILE_ATTRIB: c_int =          0x0000_0004;
pub const FILE_TRUNC: c_int =           0x0010_0000;
pub const FILE_NOFOLLOW: c_int =        0x1000_0000;

/*
 * File exception events
 */
/*
 * For the watched file:
 */
pub const FILE_DELETE: c_int =          0x0000_0010;
pub const FILE_RENAME_TO: c_int =       0x0000_0020;
pub const FILE_RENAME_FROM: c_int =     0x0000_0040;
/*
 * The file system on which the watched file resides was unmounted:
 */
pub const UNMOUNTED: c_int =            0x2000_0000;
/*
 * Some other file system was mounted over the watched file or directory:
 */
pub const MOUNTEDOVER: c_int =          0x4000_0000;

pub const FILE_EXCEPTION: c_int =       (UNMOUNTED | FILE_DELETE |
                                        FILE_RENAME_TO | FILE_RENAME_FROM |
                                        MOUNTEDOVER);

pub const ENOENT: i32 =                 2;
pub const EINTR: i32 =                  4;
pub const EBADF: i32 =                  9;
pub const EAGAIN: i32 =                 11;
pub const ENOMEM: i32 =                 12;
pub const EACCES: i32 =                 13;
pub const EFAULT: i32 =                 14;
pub const EBUSY: i32 =                  16;
pub const EINVAL: i32 =                 22;
pub const EMFILE: i32 =                 24;
pub const ENOTSUP: i32 =                48;
pub const ETIME: i32 =                  62;
pub const EBADFD: i32 =                 81;

#[derive(Debug)]
#[repr(C)]
pub struct PortEvent {
    portev_events: c_int,
    portev_source: c_ushort,
    _portev_pad: c_ushort,
    portev_object: *mut c_void,
    portev_user: *mut c_void,
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Timestruct {
    tv_sec: c_long,
    tv_nsec: c_long,
}

#[derive(Debug)]
#[repr(C)]
pub struct FileObj {
    fo_atime: Timestruct,
    fo_mtime: Timestruct,
    fo_ctime: Timestruct,
    _fo_pad: [*mut c_void; 3],
    fo_name: *const c_char,
}

#[link(name = "c")]
extern "C" {
    pub fn port_create() -> c_int;
    pub fn close(fd: c_int) -> c_int;

    pub fn port_associate(port: c_int, source: c_int, object: *const c_void,
        events: c_int, user: *mut c_void) -> c_int;
    pub fn port_dissociate(port: c_int, source: c_int, object: *const c_void)
        -> c_int;

    pub fn port_get(port: c_int, pe: *mut PortEvent,
        timeout: *const Timestruct) -> c_int;
    pub fn port_getn(port: c_int, pelist: *mut PortEvent, max: c_uint,
        nget: *mut c_uint, timeout: *const Timestruct) -> c_int;

    pub fn port_alert(port: c_int, flags: c_int, events: c_int,
        user: *mut c_void) -> c_int;

    pub fn port_send(port: c_int, events: c_int, user: *mut c_void) -> c_int;
    pub fn port_sendn(ports: *const c_int, errors: *const c_int, nent: c_uint,
        events: c_int, user: *mut c_void) -> c_int;

    pub fn ___errno() -> *mut c_int;
}

pub fn errno() -> i32 {
    unsafe {
        let errnop = ___errno();
        *errnop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_close_errno() {
        let e = errno();
        println!(" * pre close errno = {}", e);
        assert_eq!(unsafe { close(100000) }, -1);
        let e = errno();
        assert_eq!(e, EBADF);
        println!(" * post bad close errno = {}", e);
    }

    #[test]
    fn open_and_close_port() {
        let fd = unsafe { port_create() };
        if fd < 0 {
            panic!("could not create port");
        }
        println!(" * port fd: {}", fd);

        assert_eq!(unsafe { close(fd) }, 0);
    }

    #[test]
    fn test_no_events() {
        let fd = unsafe { port_create() };
        if fd < 0 {
            panic!("could not create port");
        }
        println!(" * port fd: {}", fd);

        const EVENTS: c_int = 1;
        let fd0 = fd;
        let t = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(750));

            let (r, e) = unsafe {
                let r = port_alert(fd0, PORT_ALERT_UPDATE, EVENTS,
                    std::ptr::null_mut());
                let e = errno();
                (r, e)
            };

            if r != 0 {
                panic!("port_alert failed: {}", e);
            }
        });

        let pe = Box::into_raw(Box::new(PortEvent {
            portev_events: 0,
            portev_source: 0,
            _portev_pad: 0,
            portev_object: std::ptr::null_mut(),
            portev_user: std::ptr::null_mut(),
        }));

        println!("* port get({}, {:?}) ...", fd, pe);
        let (r, e, pe) = unsafe {
            let r = port_get(fd, pe, std::ptr::null());
            let e = errno();
            (r, e, Box::from_raw(pe))
        };

        println!("r: {}, e: {}, pe: {:#?}", r, e, &pe);
        assert_eq!(pe.portev_events, EVENTS);
        assert_eq!(pe.portev_source, PORT_SOURCE_ALERT as u16);

        if r != 0 {
            panic!("port_get failed: {}", e);
        }

        println!("port_get ok!");
        t.join().unwrap();

        assert_eq!(unsafe { close(fd) }, 0);
    }
}
