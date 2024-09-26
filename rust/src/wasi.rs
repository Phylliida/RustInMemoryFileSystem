
use crate::filesystem::*;
use crate::v9p::*;
use std::panic;
use std::backtrace::Backtrace;
use crate::wasi_print;

//use libc::stat;

pub type ClockID = u32;
pub type Timestamp = u64;

// from https://www.jakubkonka.com/2020/04/28/rust-wasi-from-scratch.html
type Fd = u32;
type Size = usize;
type Errno = i32;
type Rval = u32;

#[repr(C)]
pub struct Ciovec {
    buf: *const u8,
    buf_len: Size,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    pub fn fd_write(fd: Fd, iovs_ptr: *const Ciovec, iovs_len: Size, nwritten: *mut Size) -> Errno;
    pub fn clock_time_get(clock_id: ClockID, precision: Timestamp, time: *mut Timestamp) -> i32;
}


// from https://github.com/rustwasm/console_error_panic_hook/blob/master/src/lib.rs
fn hook_impl(info: &panic::PanicHookInfo) {
    let backtrace = Backtrace::capture();

    // Print panic information
    if let Some(location) = info.location() {
        wasi_print!("Panic occurred in file '{}' at line {}", location.file(), location.line());
    } else {
        wasi_print!("Panic occurred but can't get location information...");
    }

    // Print panic payload (error message)
    if let Some(message) = info.payload().downcast_ref::<&str>() {
        wasi_print!("Panic message: {}", message);
    } else {
        wasi_print!("Panic occurred without a message");
    }

    // Print the backtrace
    wasi_print!("Stack backtrace:\n{}", backtrace);
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
macro_rules! wasi_print {
    ($($arg:tt)*) => {{
        let s = &format!($($arg)*);
        wasi_print_internal(&(s.to_owned() + "\n"));
    }}
}

pub fn wasi_print_internal(msg: &String) -> Errno {
    let str_bytes = &*msg.as_bytes();
    let ciovec = Ciovec {
        buf: str_bytes.as_ptr(),
        buf_len: str_bytes.len(),
    };
    let ciovecs = [ciovec];
    let mut nwritten = 0;
    unsafe {
        return fd_write(1, ciovecs.as_ptr(), ciovecs.len(), &mut nwritten)
    }
}

// from wasm-forge/ic-wasi-polyfill

pub fn get_file_name<'a>(path: *const u8, path_len: usize) -> &'a str {
    let path_bytes = unsafe { std::slice::from_raw_parts(path, path_len) };
    let file_name = unsafe { std::str::from_utf8_unchecked(path_bytes) };

    file_name
}

use std::sync::{LazyLock, Mutex};

static GLOBAL_FS: LazyLock<Mutex<FS>> = LazyLock::new(|| Mutex::new(FS::new(None)));

const PIPE_MAX_FD : i32 = 2; 


pub fn fd_to_index(fd: i32) -> Option<usize> {
    // first few fds are for stdout/stdin/stderr
    if fd <= PIPE_MAX_FD {
        None
    }
    else {
        // 3 corresponds to 0, etc.
        Some((fd - PIPE_MAX_FD - 1) as usize)
    }
}




#[repr(i32)]
pub enum Pipe {
    Stdin = 0,
    Stdout = 1,
    Stderr = 2   
}

pub fn fd_to_pipe(fd: i32) -> Option<Pipe> {
    match fd {
        0 => Some(Pipe::Stdin),
        1 => Some(Pipe::Stdout),
        2 => Some(Pipe::Stderr),
        _ => None,
    }
}

pub fn index_to_fd(index: usize) -> i32 {
    // 0 -> 3, 1 -> 4, ...
    return (index as i32) + PIPE_MAX_FD + 1;
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
pub extern "C" fn fd_advise(fd: i32, offset: i64, len: i64, advice: i32) -> i32 {
    if let Some(fd_index) = fd_to_index(fd) {
        if GLOBAL_FS.lock().unwrap().is_index_valid(fd_index) {
            // we don't currently use advise, just return success
            SUCCESS
        }
        else {
            EBADF // invalid file (file doesn't exist)
        }
    }
    else if let Some(fd_pipe) = fd_to_pipe(fd) {
        ESPIPE // invalid file (it's a pipe)
    }
    else {
        EINVAL // invalid file (negative)
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
pub extern "C" fn fd_allocate(fd: i32, offset: i64, len: i64) -> i32 {
    if offset < 0 || len <= 0 {
        return EINVAL; 
    }
    if let Some(fd_index) = fd_to_index(fd) {
        let mut fs = GLOBAL_FS.lock().unwrap();
        if fs.is_index_valid(fd_index) {
            if fs.get_size(fd_index) < (offset + len) as usize {
                fs.change_size(fd_index, (offset+len) as usize);
            }
            SUCCESS
        }
        else {
            EBADF // invalid file (file doesn't exist)
        }
    }
    else if let Some(fd_pipe) = fd_to_pipe(fd) {
        ESPIPE // invalid file (it's a pipe)
    }
    else {
        EBADF // invalid file (negative)
    }
}

/// Close (deletes?) a file descriptor.
// TODO: is this correct? maybe we should just return SUCCESS? I'm not sure
// fd: The file descriptor mapping to an open file to close.
/// Note: This is similar to `close` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_close(fd: i32) -> i32 {
    if let Some(fd_index) = fd_to_index(fd) {
        let mut fs = GLOBAL_FS.lock().unwrap();
        if fs.is_index_valid(fd_index) {
            fs.close_inode(fd_index);
            SUCCESS
        }
        else {
            EBADF // invalid file (file doesn't exist)
        }
    }
    else if let Some(fd_pipe) = fd_to_pipe(fd) {
        SUCCESS // ignore closing pipes, technically this is undefined behavior but it's ok
    }
    else {
        EBADF // invalid file (negative)
    }
}

/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn fd_datasync(fd: i32) -> i32 {
    if let Some(fd_index) = fd_to_index(fd) {
        if GLOBAL_FS.lock().unwrap().is_index_valid(fd_index) {
            // we don't currently use datasync, just return success
            SUCCESS
        }
        else {
            EBADF // invalid file (file doesn't exist)
        }
    }
    else if let Some(fd_pipe) = fd_to_pipe(fd) {
        SUCCESS // we don't do anything for pipes
    }
    else {
        EBADF // invalid file (negative)
    }
}
/*
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
pub extern "C" fn fd_fdstat_get(fd: i32, buf: *mut stat) -> i32 {
    if let Some(fd_index) = fd_to_index(fd) {
        if GLOBAL_FS.lock().unwrap().is_index_valid(fd_index) {
            // we don't currently use datasync, just return success
            SUCCESS
        }
        else {
            EBADF // invalid file (file doesn't exist)
        }
    }
    else if let Some(fd_pipe) = fd_to_pipe(fd) {
        SUCCESS // we don't do anything for pipes
    }
    else {
        EBADF // invalid file (negative)
    }
}


/// Adjust the flags associated with a file descriptor.
/// Note: This is similar to `fcntl(fd, F_SETFL, flags)` in POSIX.
pub fn fd_fdstat_set_flags(arg0: i32, arg1: i32) -> i32;
/// Adjust the rights associated with a file descriptor.
/// This can only be used to remove rights, and returns `errno::notcapable` if called in a way that would attempt to add rights
pub fn fd_fdstat_set_rights(arg0: i32, arg1: i64, arg2: i64) -> i32;
/// Return the attributes of an open file.
pub fn fd_filestat_get(arg0: i32, arg1: i32) -> i32;
/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
pub fn fd_filestat_set_size(arg0: i32, arg1: i64) -> i32;
/// Adjust the timestamps of an open file or directory.
/// Note: This is similar to `futimens` in POSIX.
pub fn fd_filestat_set_times(arg0: i32, arg1: i64, arg2: i64, arg3: i32) -> i32;
/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
pub fn fd_pread(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Return a description of the given preopened file descriptor.
pub fn fd_prestat_get(arg0: i32, arg1: i32) -> i32;
/// Return a description of the given preopened file descriptor.
pub fn fd_prestat_dir_name(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
pub fn fd_pwrite(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Read from a file descriptor.
/// Note: This is similar to `readv` in POSIX.
pub fn fd_read(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32;
/// Read directory entries from a directory.
/// When successful, the contents of the output buffer consist of a sequence of
/// directory entries. Each directory entry consists of a `dirent` object,
/// followed by `dirent::d_namlen` bytes holding the name of the directory
/// entry.
/// This function fills the output buffer as much as possible, potentially
/// truncating the last directory entry. This allows the caller to grow its
/// read buffer size in case it's too small to fit a single large directory
/// entry, or skip the oversized directory entry.
pub fn fd_readdir(arg0: i32, arg1: i32, arg2: i32, arg3: i64, arg4: i32) -> i32;
/// Atomically replace a file descriptor by renumbering another file descriptor.
/// Due to the strong focus on thread safety, this environment does not provide
/// a mechanism to duplicate or renumber a file descriptor to an arbitrary
/// number, like `dup2()`. This would be prone to race conditions, as an actual
/// file descriptor with the same number could be allocated by a different
/// thread at the same time.
/// This function provides a way to atomically renumber file descriptors, which
/// would disappear if `dup2()` were to be removed entirely.
pub fn fd_renumber(arg0: i32, arg1: i32) -> i32;
/// Move the offset of a file descriptor.
/// Note: This is similar to `lseek` in POSIX.
pub fn fd_seek(arg0: i32, arg1: i64, arg2: i32, arg3: i32) -> i32;
/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
pub fn fd_sync(arg0: i32) -> i32;
/// Return the current offset of a file descriptor.
/// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
pub fn fd_tell(arg0: i32, arg1: i32) -> i32;
/// Write to a file descriptor.
/// Note: This is similar to `writev` in POSIX.
pub fn fd_write(arg0: i32, arg1: i32, arg2: i32, arg3: i32) -> i32;
/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
pub fn path_create_directory(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
pub fn path_filestat_get(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32;
/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
pub fn path_filestat_set_times(
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
pub fn path_link(
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
pub fn path_open(
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
pub fn path_readlink(
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
pub fn path_remove_directory(arg0: i32, arg1: i32, arg2: i32) -> i32;
/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
pub fn path_rename(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32, arg5: i32)
    -> i32;
/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
pub fn path_symlink(arg0: i32, arg1: i32, arg2: i32, arg3: i32, arg4: i32) -> i32;
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