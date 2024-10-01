
use crate::v9p::*;
use std::panic;
use std::backtrace::Backtrace;
use crate::wasi_err;
use crate::wasi::Pipe::Stderr;

//use libc::stat;

pub type ClockID = u32;
pub type Timestamp = u64;


#[link(wasm_import_module = "wasi_snapshot_preview1")]
extern "C" {
    // needed to set file times
    pub fn clock_time_get(clock_id: ClockID, precision: Timestamp, time: *mut Timestamp) -> ErrorNumber;
    // needed for stdout/stderr (mostly, for panic to be able to print)
    pub fn fd_write(fd: i32, iovs_ptr: *const Ciovec, iovs_len: i32, nwritten: *mut usize) -> ErrorNumber;
    // needed for stdin
    pub fn fd_read(fd: i32, iovs: *const Ciovec, iovs_len: i32, nread: *mut usize) -> ErrorNumber;
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
    match pipe {
        Pipe::Stdin => ErrorNumber::EOPNOTSUPP,
        Pipe::Stdout => unsafe { fd_write(pipe as i32, ciovecs.as_ptr(), ciovecs.len() as i32, &mut nwritten) },
        Pipe::Stderr => unsafe { fd_write(pipe as i32, ciovecs.as_ptr(), ciovecs.len() as i32, &mut nwritten) }
    }
}

// modified from wasm-forge/ic-wasi-polyfill

pub unsafe fn get_str<'a>(path: *const u8, path_len: usize) -> &'a str {
    let path_bytes = std::slice::from_raw_parts(path, path_len);
    let res_str = std::str::from_utf8_unchecked(path_bytes);
    res_str
}

use std::sync::{LazyLock, Mutex};

static GLOBAL_FS: LazyLock<Mutex<Virtio9p>> = LazyLock::new(|| {
    set_panic_hook();
    Mutex::new(Virtio9p::new(None))
});


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
pub extern "C" fn fd_advise(fd: i32, _offset: i64, _len: i64, _advice: i32) -> ErrorNumber {
    if fd < 0 {
        return ErrorNumber::EINVAL; // invalid file (negative) // posix_fadvise says we should return this if negative
    }
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        ErrorNumber::ESPIPE // invalid file (it's a pipe)
    } else if let Some(_fd_file) = GLOBAL_FS.lock().unwrap().get_fd(fd) {
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
pub extern "C" fn __fs_custom_fd_allocate(fd: i32, offset: i64, len: i64) -> ErrorNumber {
    if offset < 0 || len <= 0 {
        return ErrorNumber::EINVAL;
    }
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // invalid file (it's a pipe)
    }
    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        fs.allocate(fd_file, offset, len)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Close a file descriptor.
// fd: The file descriptor mapping to an open file to close.
/// Note: This is similar to `close` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_close(fd: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::SUCCESS // ignore closing pipes, technically this is undefined behavior but it's ok
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        fs.close_fd(fd_file)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Synchronize the data of a file to disk.
/// Note: This is similar to `fdatasync` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_datasync(fd: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        ErrorNumber::SUCCESS // also does nothing for pipes
    } else if let Some(_fd_file) = GLOBAL_FS.lock().unwrap().get_fd(fd) {
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
pub extern "C" fn __fs_custom_fd_fdstat_get(fd: i32, buf_ptr: *mut FdStat) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        let stat = unsafe { &mut *buf_ptr };
        stat.fs_filetype = FdFileType::CharacterDevice; // streams are character devices
        stat.fs_flags = FdFlags::empty();
        stat.fs_rights_base = Virtio9p::get_pipe_rights(fd_pipe);
        stat.fs_rights_inheriting = FdRights::empty();
        return ErrorNumber::SUCCESS;
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let stat = unsafe { &mut *buf_ptr };
        fs.fd_stat(fd_file, stat)
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
pub extern "C" fn __fs_custom_fd_fdstat_set_flags(fd: i32, flags: FdFlags) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set flags on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
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
pub extern "C" fn __fs_custom_fd_fdstat_set_rights(fd: i32, fs_rights_base: FdRights, fs_rights_inheriting: FdRights) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        fs.fd_stat_set_rights(fd_file, fs_rights_base, fs_rights_inheriting)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
 
/// Return the attributes of an open file.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_filestat_get(fd: i32, stat: *mut FileStat) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let stat_ref = unsafe { &mut *stat };
        fs.get_file_stat(fd_file, stat_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Adjust the size of an open file. If this increases the file's size, the extra bytes are filled with zeros.
/// Note: This is similar to `ftruncate` in POSIX.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_filestat_set_size(fd: i32, size: i64) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set rights on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
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
pub extern "C" fn __fs_custom_fd_filestat_set_times(fd: i32, st_atim: Timestamp, st_mtim: Timestamp, fst_flags: FstFlags) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE; // can't set time on pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        fs.file_stat_set_times(fd_file, st_atim, st_mtim, fst_flags)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}


/// Read from a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `preadv` in POSIX.
// fd: The file descriptor of the file to read from.
// iovs: A pointer to an array of __wasi_iovec_t structures describing the buffers where the data will be stored.
// iovs_len: The number of vectors (__wasi_iovec_t) in the iovs array.
// offset: The file cursor indicating the starting position from which data will be read.
// nread: A pointer to store the number of bytes read.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_pread(fd: i32, iovs: *const Ciovec, iovs_len: i32, offset: i64, nread: *mut usize) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::EOPNOTSUPP; // we don't support pread for pipe
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        // modified from https://github.com/wasm-forge/ic-wasi-polyfill/blob/bd7bf38e665d0147ddee1ba428052456978a4028/src/lib.rs#L291C5-L292C80
        let dst_io_vec = iovs as *const DstBuf;
        let dst_io_vec = unsafe { std::slice::from_raw_parts(dst_io_vec, iovs_len as usize) };
        let nread_ref = unsafe { &mut *nread };
        // because we specify offset, the fd's offset won't be used or modified
        fs.read_vec(fd_file, dst_io_vec, Some(offset as usize), nread_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Return a description of the given preopened file descriptor.
// fd: The preopened file descriptor to query.
// buf: A pointer to a Prestat structure where the metadata will be written.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_prestat_get(fd: i32, buf : *mut PreStat) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let buf_ref = unsafe {&mut *buf};
        fs.prestat_get(fd_file, buf_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Return a description of the given preopened file descriptor.
// fd: The preopened file descriptor to query.
// path: A pointer to a buffer where the directory name will be written.
// max_len: The maximum length of the buffer.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_prestat_dir_name(fd: i32, path: *mut u8, max_len: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let dst_arr = unsafe { std::slice::from_raw_parts_mut(path, max_len as usize) };
        fs.prestat_dir_name(fd_file, dst_arr)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Write to a file descriptor, without using and updating the file descriptor's offset.
/// Note: This is similar to `pwritev` in POSIX.
// fd: The file descriptor of the file to write to.
// iovs: A pointer to an array of __wasi_ciovec_t structures describing the buffers from which data will be read.
// iovs_len: The number of vectors (__wasi_ciovec_t) in the iovs array.
// offset: The offset indicating the position at which the data will be written.
// nwritten: A pointer to store the number of bytes written.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_pwrite(fd: i32, iovs: *const Ciovec, iovs_len: i32, offset: i64, nwritten: *mut usize) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::EOPNOTSUPP; // we don't support pwrite for iostream
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let casted_vec = iovs as *const SrcBuf;
        let writing_vec: &[SrcBuf] = unsafe { std::slice::from_raw_parts(casted_vec, iovs_len as usize) };
        let nwritten_ref = unsafe { &mut *nwritten };
        // because we specify offset, the fd's offset won't be used or modified
        fs.write_vec(fd_file, writing_vec, Some(offset as usize), nwritten_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Read from a file descriptor.
/// Note: This is similar to `readv` in POSIX.
// fd: The file descriptor of the file to read from.
// iovs: A pointer to an array of __wasi_iovec_t structures describing the buffers where the data will be stored.
// iovs_len: The number of vectors (__wasi_iovec_t) in the iovs array.
// nread: A pointer to store the number of bytes read.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_read(fd: i32, iovs: *const Ciovec, iovs_len: i32, nread: *mut usize) -> ErrorNumber {
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return match fd_pipe {
            // we forward this to standard wasi
            Pipe::Stdin => unsafe { fd_read(fd, iovs, iovs_len, nread) },
            Pipe::Stdout => ErrorNumber::EOPNOTSUPP,
            Pipe::Stderr => ErrorNumber::EOPNOTSUPP
        };
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        // from https://github.com/wasm-forge/ic-wasi-polyfill/blob/bd7bf38e665d0147ddee1ba428052456978a4028/src/lib.rs#L291C5-L292C80
        let dst_buf = iovs as *const DstBuf;
        let dst_io_vec = unsafe { std::slice::from_raw_parts(dst_buf, iovs_len as usize) };
        let nread_ref = unsafe {&mut *nread};
        fs.read_vec(fd_file, dst_io_vec, None, nread_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Read directory entries from a directory.
/// When successful, the contents of the output buffer consist of a sequence of
/// directory entries. Each directory entry consists of a `dirent` object,
/// followed by `dirent::d_namlen` bytes holding the name of the directory
/// entry.
/// This function fills the output buffer as much as possible, potentially
/// truncating the last directory entry. This allows the caller to grow its
/// read buffer size in case it's too small to fit a single large directory
/// entry, or skip the oversized directory entry.
// fd: The file descriptor of the directory to read from.
// buf: A pointer to the buffer where directory entries will be stored.
// buf_len: The length of the buffer in bytes.
// cookie: The directory cookie indicating the position to start reading from.
// bufused: A pointer to store the number of bytes stored in the buffer.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_readdir(fd: i32, buf: *mut DirectoryEntry, buf_len: i32, cookie: i64, bufused: *mut usize) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    if buf_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let dir_entries: &mut [DirectoryEntry] = unsafe {
            std::slice::from_raw_parts_mut(buf, buf_len as usize)
        };
        let bufused_ref = unsafe { &mut *bufused };
        fs.read_dir(fd_file, dir_entries, cookie as usize, bufused_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Atomically replace a file descriptor by renumbering another file descriptor.
/// Due to the strong focus on thread safety, this environment does not provide
/// a mechanism to duplicate or renumber a file descriptor to an arbitrary
/// number, like `dup2()`. This would be prone to race conditions, as an actual
/// file descriptor with the same number could be allocated by a different
/// thread at the same time.
/// This function provides a way to atomically renumber file descriptors, which
/// would disappear if `dup2()` were to be removed entirely.
// from: The file descriptor to copy.
// to: The location to copy the file descriptor to.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_renumber(from: i32, to: i32) -> ErrorNumber {
    if let Some(_fd_pipe_from) = Virtio9p::get_pipe_fd(from) {
        return ErrorNumber::ESPIPE; // what are u doing
    }
    if let Some(_fd_pipe_to) = Virtio9p::get_pipe_fd(to) {
        return ErrorNumber::ESPIPE; // what are u doing
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file_from) = fs.get_fd(from) {
        if let Some(fd_file_to) = fs.get_fd(to) {
            fs.renumber(fd_file_from, fd_file_to)
        }
        else {
            ErrorNumber::EBADF
        }
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Move the offset of a file descriptor.
/// Note: This is similar to `lseek` in POSIX.
// fd: The file descriptor to update.
// offset: The number of bytes to adjust the offset by.
// whence: The position that the offset is relative to.
// newoffset: A WebAssembly memory pointer where the new offset will be stored.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_seek(fd: i32, offset: i64, whence: SeekWhence, newoffset: *mut usize) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let newoffset_ref = unsafe { &mut *newoffset };
        fs.seek(fd_file, offset, whence, newoffset_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Synchronize the data and metadata of a file to disk.
/// Note: This is similar to `fsync` in POSIX.
// fd: The file descriptor to sync.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_sync(fd: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(_fd_file) = fs.get_fd(fd) {
        ErrorNumber::SUCCESS // don't need to do anything, it's always syncronized
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Return the current offset of a file descriptor.
/// Note: This is similar to `lseek(fd, 0, SEEK_CUR)` in POSIX.
// fd: The file descriptor to access.
// offset: A wasm pointer to a Filesize where the offset will be written.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_tell(fd: i32, offset: *mut usize) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return ErrorNumber::ESPIPE;
    }

    let fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let offset_ref = unsafe { &mut *offset };
        fs.tell(fd_file, offset_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Write to a file descriptor.
/// Note: This is similar to `writev` in POSIX.
// fd: The file descriptor (opened with writing permission) to write to.
// iovs: A wasm pointer to an array of __wasi_ciovec_t structures, each describing a buffer to write data from.
// iovs_len: The length of the iovs array.
// nwritten: A wasm pointer to an M::Offset value where the number of bytes written will be written.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_fd_write(fd: i32, iovs: *const Ciovec, iovs_len: i32, nwritten: *mut usize) -> ErrorNumber{
    if let Some(fd_pipe) = Virtio9p::get_pipe_fd(fd) {
        return match fd_pipe {
            Pipe::Stdin => ErrorNumber::EOPNOTSUPP,
            // we forward this to standard WASI
            Pipe::Stdout => unsafe { fd_write(fd, iovs, iovs_len, nwritten) },
            Pipe::Stderr => unsafe { fd_write(fd, iovs, iovs_len, nwritten) }
        };
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(fd_file) = fs.get_fd(fd) {
        let casted_vec = iovs as *const SrcBuf;
        let writing_vec: &[SrcBuf] = unsafe {
            std::slice::from_raw_parts(casted_vec, iovs_len as usize)
        };
        let nwritten_ref = unsafe {&mut *nwritten};
        // because we specify offset, the fd's offset won't be used or modified
        fs.write_vec(fd_file, writing_vec, None, nwritten_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Create a directory.
/// Note: This is similar to `mkdirat` in POSIX.
// parent_fd: The file descriptor representing the directory that the path is relative to.
// path: A wasm pointer to a null-terminated string containing the path data.
// path_len: The length of the path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_create_directory(parent_fd: i32, path: *const u8, path_len: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        fs.create_directory(parent_dir_fd, path_str)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Return the attributes of a file or directory.
/// Note: This is similar to `stat` in POSIX.
// fd: The file descriptor representing the directory that the path is relative to.
// symlink_flags: Flags to control how the symlink path is understood.
// path: A wasm pointer to a null-terminated string containing the file path.
// path_len: The length of the path string.
// result: A wasm pointer to a Filestat object where the metadata will be stored.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_filestat_get(parent_fd: i32,
    symlink_flags: SymlinkLookupFlags,
    path: *const u8,
    path_len: i32,
    result: *mut FileStat) -> ErrorNumber
{
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        let result_ref = unsafe { &mut *result };
        fs.path_file_stat_get(parent_dir_fd, symlink_flags, path_str, result_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}

/// Adjust the timestamps of a file or directory.
/// Note: This is similar to `utimensat` in POSIX.
// parent_fd: The file descriptor representing the directory that the path is relative to.
// flags: Flags to control how the path is understood.
// path: A wasm pointer to a null-terminated string containing the file path.
// path_len: The length of the path string.
// st_atim: The timestamp that the last accessed time attribute is set to.
// st_mtim: The timestamp that the last modified time attribute is set to.
// fst_flags: A bitmask controlling which attributes are set.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_filestat_set_times(
    parent_fd: i32,
    symlink_flags: SymlinkLookupFlags,
    path: *const u8,
    path_len: i32,
    atim: Timestamp,
    mtim: Timestamp,
    fst_flags: FstFlags,
) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path = unsafe { get_str(path, path_len as usize) };
        fs.path_file_stat_set_times(parent_dir_fd, symlink_flags, path, atim, mtim, fst_flags)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Create a hard link.
/// Note: This is similar to `linkat` in POSIX.
// old_parent_fd: The file descriptor representing the directory that the old_path is relative to.
// old_flags: Flags to control how the old_path is understood.
// old_path: A wasm pointer to a null-terminated string containing the old file path.
// old_path_len: The length of the old_path string.
// new_parent_fd: The file descriptor representing the directory that the new_path is relative to.
// new_path: A wasm pointer to a null-terminated string containing the new file path.
// new_path_len: The length of the new_path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_link(
    old_parent_fd: i32,
    old_flags: SymlinkLookupFlags,
    old_path: *const u8,
    old_path_len: i32,
    new_parent_fd: i32,
    new_path: *const u8,
    new_path_len: i32
) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(old_parent_fd) {
        return ErrorNumber::ESPIPE;
    }
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(new_parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if old_path_len < 0 || new_path_len < 0 {
        return ErrorNumber::EINVAL;
    }


    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(old_parent_dir_fd) = fs.get_fd(old_parent_fd) {
        if let Some(new_parent_dir_fd) = fs.get_fd(new_parent_fd) {
            let old_path_str = unsafe { get_str(old_path, old_path_len as usize) };
            let new_path_str = unsafe { get_str(new_path, new_path_len as usize) };
            fs.link(old_parent_dir_fd,
                old_flags,
                old_path_str,
                new_parent_dir_fd,
                new_path_str)
        }
        else {
            ErrorNumber::EBADF // new parent doesn't exist
        }
    } else {
        ErrorNumber::EBADF // old parent doesn't exist
    }
}
/// Open a file or directory.
/// The returned file descriptor is not guaranteed to be the lowest-numbered
/// file descriptor not currently open; it is randomized to prevent
/// applications from depending on making assumptions about indexes, since this
/// is error-prone in multi-threaded contexts. The returned file descriptor is
/// guaranteed to be less than 2**31.
/// Note: This is similar to `openat` in POSIX.
// parent_fd: The file descriptor representing the directory that the file is located in.
// parent_fd_flags: Flags specifying how the path will be resolved.
// path: A wasm pointer to a null-terminated string containing the path of the file or directory to open.
// path_len: The length of the path string.
// o_flags: Flags specifying how the file will be opened.
// fs_rights_base: The rights of the created file descriptor.
// fs_rights_inheriting: The rights of file descriptors derived from the created file descriptor.
// fs_flags: The flags of the file descriptor.
// fd: A wasm pointer to a WasiFd variable where the new file descriptor will be stored.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_open(
    parent_fd: i32,
    parent_fd_flags: SymlinkLookupFlags,
    path: *const u8,
    path_len: i32,

    oflags: FileOpenFlags,
    fs_rights_base: FdRights,
    fs_rights_inheriting: FdRights,

    fdflags: FdFlags,
    res: *mut FileDescriptorID
) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        let fd_out_ref = unsafe { &mut *res };
        fs.path_open(parent_dir_fd, parent_fd_flags, path_str,
            oflags, fs_rights_base, fs_rights_inheriting,
            fdflags, fd_out_ref)
    } else {
        ErrorNumber::EBADF // invalid file (file doesn't exist)
    }
}
/// Read the contents of a symbolic link.
/// Note: This is similar to `readlinkat` in POSIX.
// parent_fd: The file descriptor representing the base directory from which the symlink is understood.
// path: A wasm pointer to a null-terminated string containing the path to the symlink.
// path_len: The length of the path string.
// buf: A wasm pointer to a buffer where the target path of the symlink will be written.
// buf_len: The available space in the buffer pointed to by buf.
// buf_used: A wasm pointer to a variable where the number of bytes written to the buffer will be stored.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_readlink(
    parent_fd: i32,
    path: *const u8,
    path_len: i32,
    buf: *mut u8,
    buf_len: i32,
    buf_used: *mut usize,
) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if buf_len < 0 || path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        let out_buf = unsafe { std::slice::from_raw_parts_mut(buf, buf_len as usize) };
        let out_buf_used = unsafe { &mut *buf_used };
        fs.path_read_link(parent_dir_fd, path_str, out_buf, out_buf_used)
    }
    else {
        ErrorNumber::EBADF
    }
}
/// Remove a directory.
/// Return `errno::notempty` if the directory is not empty.
/// Note: This is similar to `unlinkat(fd, path, AT_REMOVEDIR)` in POSIX.
// parent_fd: The file descriptor representing the base directory from which the path is resolved.
// path: A wasm pointer to a null-terminated string containing the path of the directory to remove.
// path_len: The length of the path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_remove_directory(parent_fd: i32, path: *const u8, path_len: i32) -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        fs.path_unlink_dir(parent_dir_fd, path_str)
    }
    else {
        ErrorNumber::EBADF
    }
}
/// Rename a file or directory.
/// Note: This is similar to `renameat` in POSIX.
// old_parent_fd: The file descriptor representing the base directory for the source path.
// old_path: A wasm pointer to a null-terminated string containing the source path of the file or directory to be renamed.
// old_path_len: The length of the old_path string.
// new_parent_fd: The file descriptor representing the base directory for the target path.
// new_path: A wasm pointer to a null-terminated string containing the target path with the new name for the file or directory.
// new_path_len: The length of the new_path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_rename(old_parent_fd: i32,
    old_path: *const u8,
    old_path_len: i32,
    new_parent_fd: i32,
    new_path: *const u8,
    new_path_len: i32)
    -> ErrorNumber {
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(old_parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(new_parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if old_path_len < 0 || new_path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(old_parent_dir_fd) = fs.get_fd(old_parent_fd) {
        if let Some(new_parent_dir_fd) = fs.get_fd(new_parent_fd) {
            let old_path_str = unsafe { get_str(old_path, old_path_len as usize) };
            let new_path_str = unsafe { get_str(new_path, new_path_len as usize) };
            fs.rename(old_parent_dir_fd,
                old_path_str,
                new_parent_dir_fd,
                new_path_str)    
        }
        else {
            ErrorNumber::EBADF
        }
    }
    else {
        ErrorNumber::EBADF
    }   
}
/// Create a symbolic link.
/// Note: This is similar to `symlinkat` in POSIX.
// old_path: A wasm pointer to a null-terminated string containing the source path of the symlink.
// old_path_len: The length of the old_path string.
// fd: The file descriptor representing the base directory from which the paths are understood.
// new_path: A wasm pointer to a null-terminated string containing the target path where the symlink will be created.
// new_path_len: The length of the new_path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_symlink(
    old_path: *const u8,
    old_path_len: i32,
    parent_fd: i32,
    new_path: *const u8,
    new_path_len: i32) -> ErrorNumber
{
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if old_path_len < 0 || new_path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let old_path_str = unsafe { get_str(old_path, old_path_len as usize) };
        let new_path_str = unsafe { get_str(new_path, new_path_len as usize) };
        fs.symlink(parent_dir_fd, old_path_str, new_path_str)
    }
    else {
        ErrorNumber::EBADF
    }   
}
    
/// Unlink a file.
/// Return `errno::isdir` if the path refers to a directory.
/// Note: This is similar to `unlinkat(fd, path, 0)` in POSIX.
// parent_fd: The file descriptor representing the base directory from which the path is understood.
// path: A wasm pointer to a null-terminated string containing the path of the file to be unlinked.
// path_len: The length of the path string.
#[no_mangle]
#[inline(never)]
pub extern "C" fn __fs_custom_path_unlink_file(parent_fd: i32,
    path: *const u8,
    path_len: i32) -> ErrorNumber
{
    if let Some(_fd_pipe) = Virtio9p::get_pipe_fd(parent_fd) {
        return ErrorNumber::ESPIPE;
    }

    if path_len < 0 {
        return ErrorNumber::EINVAL;
    }

    let mut fs = GLOBAL_FS.lock().unwrap();
    if let Some(parent_dir_fd) = fs.get_fd(parent_fd) {
        let path_str = unsafe { get_str(path, path_len as usize) };
        fs.path_unlink_file(parent_dir_fd, path_str)
    }
    else {
        ErrorNumber::EBADF
    }    
}