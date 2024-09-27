
use crate::v9p::*;
use std::panic;
use std::backtrace::Backtrace;
use crate::wasi_err;
use crate::wasi::Pipe::Stderr;

//use libc::stat;

pub type ClockID = u32;
pub type Timestamp = u64;

// from https://www.jakubkonka.com/2020/04/28/rust-wasi-from-scratch.html
type Fd = u32;
type Size = usize;
type Rval = u32;

#[repr(C)]
pub struct Ciovec {
    buf: *const u8,
    buf_len: Size,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    // needed for stdin/stdout/stderr (mostly, for panic to be able to print)
    pub fn fd_write(fd: Fd, iovs_ptr: *const Ciovec, iovs_len: Size, nwritten: *mut Size) -> ErrorNumber;
    // needed to set file times
    pub fn clock_time_get(clock_id: ClockID, precision: Timestamp, time: *mut Timestamp) -> ErrorNumber;
}


// from https://github.com/rustwasm/console_error_panic_hook/blob/master/src/lib.rs
fn hook_impl(info: &panic::PanicHookInfo) {
    let backtrace = Backtrace::capture();

    // Print panic information
    if let Some(location) = info.location() {
        wasi_err!("Panic occurred in file '{}' at line {}", location.file(), location.line());
    } else {
        wasi_err!("Panic occurred but can't get location information...");
    }

    // Print panic payload (error message)
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        wasi_err!("Panic message: {}", message);
    } else {
        wasi_err!("Panic occurred without a message");
    }

    // Print the backtrace
    wasi_err!("Stack backtrace:\n{}", backtrace);
}
/// Set the `console.error` panic hook the first time this is called. Subsequent
/// invocations do nothing.
#[inline]
pub fn set_panic_hook() {
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(hook_impl));
    });
}


#[macro_export]
macro_rules! print_debug {
    ($($arg:tt)*) => {{
        let s = &format!($($arg)*);
        wasi_print_internal(Stdout, &(s.to_owned() + "\n"));
    }}
}


#[macro_export]
macro_rules! wasi_print {
    ($($arg:tt)*) => {{
        let s = &format!($($arg)*);
        wasi_print_internal(Stdout, &(s.to_owned() + "\n"));
    }}
}

#[macro_export]
macro_rules! wasi_err {
    ($($arg:tt)*) => {{
        let s = &format!($($arg)*);
        wasi_print_internal(Stderr, &(s.to_owned() + "\n"));
    }}
}

pub fn wasi_print_internal(pipe : Pipe, msg: &String) -> ErrorNumber {
    let str_bytes = &*msg.as_bytes();
    let ciovec = Ciovec {
        buf: str_bytes.as_ptr(),
        buf_len: str_bytes.len(),
    };
    let ciovecs = [ciovec];
    let mut nwritten = 0;
    unsafe {
        return fd_write(pipe as u32, ciovecs.as_ptr(), ciovecs.len(), &mut nwritten)
    }
}

// from wasm-forge/ic-wasi-polyfill

pub fn get_file_name<'a>(path: *const u8, path_len: usize) -> &'a str {
    let path_bytes = unsafe { std::slice::from_raw_parts(path, path_len) };
    let file_name = unsafe { std::str::from_utf8_unchecked(path_bytes) };

    file_name
}

use std::sync::{LazyLock, Mutex};

static GLOBAL_FS: LazyLock<Mutex<Virtio9p>> = LazyLock::new(|| Mutex::new(Virtio9p::new(None)));

const PIPE_MAX_FD : i32 = 2; 

#[repr(i32)]
pub enum Pipe {
    Stdin = 0,
    Stdout = 1,
    Stderr = 2   
}

// definitions and comments from https://github.com/wasm-forge/ic-wasi-polyfill/blob/main/src/wasi.rs#L2178C7-L2374C59
/// Read command-line argument data.
/// The size of the array should match that returned by `args_sizes_get`.
/// Each argument is expected to be `\0` terminated.
//pub fn args_get(arg0: i32, arg1: i32) -> i32;
/// Return command-line argument data sizes.
//pub fn args_sizes_get(arg0: i32, arg1: i32) -> i32;
/// Read environment variable data.
/// The sizes of the buffers should match that returned by `environ_sizes_get`.
/// Key/value pairs are expected to be joined with `=`s, and terminated with `\0`s.
//pub fn environ_get(arg0: i32, arg1: i32) -> i32;
/// Return environment variable data sizes.
//pub fn environ_sizes_get(arg0: i32, arg1: i32) -> i32;
/// Return the resolution of a clock.
/// Implementations are required to provide a non-zero value for supported clocks. For unsupported clocks,
/// return `errno::inval`.
/// Note: This is similar to `clock_getres` in POSIX.
//pub fn clock_res_get(arg0: i32, arg1: i32) -> i32;
/// Return the time value of a clock.
/// Note: This is similar to `clock_gettime` in POSIX.
//pub fn clock_time_get(arg0: i32, arg1: i64, arg2: i32) -> i32;




/// Provide file advisory information on a file descriptor.
// (predeclare an access pattern for file data)
/// Note: This is similar to `posix_fadvise` in POSIX.
// Parameters
// fd: The file descriptor to which the advice applies.
// offset: The offset from which the advice applies.
// len: The length from the offset to which the advice applies.
// advice: The advice to be given to the operating system
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_advise(fd: i32, offset: i64, len: i64, advice: i32) -> ErrorNumber {
    if fd < 0 {
        return ErrorNumber::EINVAL; // invalid file (negative) // posix_fadvise says we should return this if negative
    }
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        ErrorNumber::ESPIPE // invalid file (it's a pipe)
    } else if let Some(fd_file) = GLOBAL_FS.lock().unwrap().get_file_fd(fd) {
        // we don't currently use advise, just return success
        ErrorNumber::SUCCESS
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}


// fd_allocate
// Allocate extra space for a file descriptor.
// Description
// The fd_allocate function is used to allocate additional space for a file descriptor. It allows extending the size of a file or buffer associated with the file descriptor.
// Parameters
// fd: The file descriptor to allocate space for.
// offset: The offset from the start marking the beginning of the allocation.
// len: The length from the offset marking the end of the allocation.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_allocate(fd: i32, offset: i64, len: i64) -> ErrorNumber {
    if offset < 0 || len <= 0 {
        return ErrorNumber::EINVAL;
    }
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // invalid file (it's a pipe)
    }
    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.allocate(fd_file, offset, len);
        ErrorNumber::SUCCESS
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Close a file descriptor.
// fd: The file descriptor mapping to an open file to close.
/// Note: This is similar to `close` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_close(fd: i32) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::SUCCESS // ignore closing pipes, technically this is undefined behavior but it's ok
    };
    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.close_fd(fd_file);
        ErrorNumber::SUCCESS
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_datasync(fd: i32) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        ErrorNumber::SUCCESS // also does nothing for pipes
    } else if let Some(fd_file) = GLOBAL_FS.lock().unwrap().get_file_fd(fd) {
        // we don't currently use advise, just return success
        ErrorNumber::SUCCESS
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}


// fd_fdstat_get
// Get metadata of a file descriptor.
// Description
// The fd_fdstat_get() function is used to retrieve the metadata of a file descriptor. It provides information about the state of the file descriptor, such as its rights, flags, and file type.
// In POSIX systems, file descriptors are small, non-negative integers used to represent open files, sockets, or other I/O resources. They serve as handles that allow processes to read from or write to these resources. The fd_fdstat_get() function allows applications to retrieve information about a specific file descriptor, gaining insights into its properties and characteristics.
// Parameters
// fd: The file descriptor whose metadata will be accessed.
// buf_ptr: A WebAssembly pointer to a memory location where the metadata will be written.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_fdstat_get(fd: i32, buf_ptr: *mut FdStat) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        unsafe {
            let stat = &mut *buf_ptr;
            stat.fs_filetype = FdFileType::CharacterDevice; // streams are character devices
            stat.fs_flags = FdFlags::empty();
            stat.fs_rights_base = Virtio9p::get_pipe_rights(fd_pipe);
            stat.fs_rights_inheriting = FdRights::empty();
        }
        return ErrorNumber::SUCCESS;
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        unsafe {
            fs.fd_stat(fd_file, &mut *buf_ptr);
        }
        ErrorNumber::SUCCESS
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}


/// Adjust the flags associated with a file descriptor.
/// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
// Parameters
// fd: The file descriptor to apply the new flags to.
// flags: The flags to apply to the file descriptor.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_fdstat_set_flags(fd: i32, flags: FdFlags) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set flags on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.fd_stat_set_flags(fd_file, flags)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Adjust the rights associated with a file descriptor.
/// This can only be used to remove rights, and returns `errno::notcapable` if called in a way that would attempt to add rights
// fd: The file descriptor to apply the new rights to.
// fs_rights_base: The base rights to apply to the file descriptor.
// fs_rights_inheriting: The inheriting rights to apply to the file descriptor.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_fdstat_set_rights(fd: i32, fs_rights_base: FdRights, fs_rights_inheriting: FdRights) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.fd_stat_set_rights(fd_file, fs_rights_base, fs_rights_inheriting)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
 
/// Return the attributes of an open file.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_filestat_get(fd: i32, stat: *mut FileStat) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        unsafe {
            fs.get_file_stat(fd_file, &mut *stat)
        }
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_filestat_set_size(fd: i32, size: i64) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.file_stat_set_size(fd_file, size as usize)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }

}
/// Adjust the timestamps of an open file or directory.
/// Note: This is similar to `futimens` in POSIX.
// fd: The file descriptor of the file to set the timestamp metadata.
// st_atim: The last accessed time to set.
// st_mtim: The last modified time to set.
// fst_flags: A bit-vector that controls which times to set.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_filestat_set_times(fd: i32, st_atim: Timestamp, st_mtim: Timestamp, fst_flags: FstFlags) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set time on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_file_fd(fd) {
        fs.file_stat_set_times(fd_file, st_atim, st_mtim, fst_flags)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/*

/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_pread(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Return a description of the given preopened file descriptor.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_prestat_get(arg0: i32, arg1: i32) -> i32;
/// Return a description of the given preopened file descriptor.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_prestat_dir_name(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_pwrite(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Read from a file descriptor.
/// Note: This is similar to `readv` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_read(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32;
/// Read directory entries from a directory.
/// When successful, the contents of the output buffer consist of a sequence of
/// directory entries. Each directory entry consists of a `dirent` object,
/// followed by `dirent::d_namlen` bytes holding the name of the directory
/// entry.
/// This function fills the output buffer as much as possible, potentially
/// truncating the last directory entry. This allows the caller to grow its
/// read buffer size in case it's too small to fit a single large directory
/// entry, or skip the oversized directory entry.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_readdir(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Atomically replace a file descriptor by renumbering another file descriptor.
/// Due to the strong focus on thread safety, this environment does not provide
/// a mechanism to duplicate or renumber a file descriptor to an arbitrary
/// number, like `dup2()`. This would be prone to race conditions, as an actual
/// file descriptor with the same number could be allocated by a different
/// thread at the same time.
/// This function provides a way to atomically renumber file descriptors, which
/// would disappear if `dup2()` were to be removed entirely.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_renumber(arg0: i32, arg1: i32) -> i32;
/// Move the offset of a file descriptor.
/// Note: This is similar to `lseek` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_seek(arg0: i32, arg1: i64, arg2: i32, arg3: i32) -> i32;
/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_sync(arg0: i32) -> i32;
/// Return the current offset of a file descriptor.
/// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_tell(arg0: i32, arg1: i32) -> i32;
/// Write to a file descriptor.
/// Note: This is similar to `writev` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_write(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32;
/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_create_directory(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_filestat_get(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32;
/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_filestat_set_times(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i64,
    arg5: i64,
    arg6: i32,
) -> i32;
/// Create a hard link.
/// Note: This is similar to `linkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_link(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    arg6: i32,
) -> i32;
/// Open a file or directory.
/// The returned file descriptor is not guaranteed to be the lowest-numbered
/// file descriptor not currently open; it is randomized to prevent
/// applications from depending on making assumptions about indexes, since this
/// is error-prone in multi-threaded contexts. The returned file descriptor is
/// guaranteed to be less than 2**31.
/// Note: This is similar to `openat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_open(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i64,
    arg6: i64,
    arg7: i32,
    arg8: i32,
) -> i32;
/// Read the contents of a symbolic link.
/// Note: This is similar to `readlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_readlink(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
) -> i32;
/// Remove a directory.
/// Return `errno::notempty` if the directory is not empty.
/// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_remove_directory(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_rename(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32)
    -> i32;
/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn path_symlink(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32;
/// Unlink a file.
/// Return `errno::isdir` if the path refers to a directory.
/// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
#[no_mangle]
#[inline(never)]
pub fn path_unlink_file(parent_fd: i32,
    path: *const u8,
    path_len: i32,
);
*/