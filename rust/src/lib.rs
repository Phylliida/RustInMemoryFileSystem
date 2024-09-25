#![allow(unused_variables)]
#![allow(dead_code)]

// TODO: STATUS_ON_STORAGE STUFF
// TODO: clone_me rename to clone once I'm done with porting (For Uint8Array)
// TODO: Is it ok to clone FSLockRegion vec?


use serde::{Serialize, Deserialize};

type Number = i64;

use std::collections::HashMap;
use std::vec::Vec;
use async_trait::async_trait;
use std::sync::Arc;
use std::io::{Cursor, Read};  // Add Read here
use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::str::from_utf8;
use std::cmp::min;

pub fn print_debug(msg: String) {
    println!("{}", msg);
}

fn marshall_u8(val: u8, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u8(val);
    return cursor.position();
}

fn unmarshall_u8(data: &[u8], offset: u64) -> (u8, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u8();
    return (result.unwrap(), cursor.position());
}

fn marshall_u16(val: u16, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u16::<BigEndian>(val);
    return cursor.position();
}

fn unmarshall_u16(data: &[u8], offset: u64) -> (u16, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u16::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

fn marshall_u32(val: u32, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u32::<BigEndian>(val);
    return cursor.position();
}

fn unmarshall_u32(data: &[u8], offset: u64) -> (u32, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u32::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

fn marshall_u64(val: u64, data: &mut [u8], offset: u64) -> u64 {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let _ = cursor.write_u64::<BigEndian>(val);
    return cursor.position();
}

fn unmarshall_u64(data: &[u8], offset: u64) -> (u64, u64) {
    let mut cursor = Cursor::new(data);
    cursor.set_position(offset);
    let result = cursor.read_u64::<BigEndian>();
    return (result.unwrap(), cursor.position());
}

fn marshall_string(val: &String, data: &mut [u8], offset: u64) -> u64 {
    // write string length
    let as_bytes: &[u8] = val.as_bytes();
    let new_offset = marshall_u16(as_bytes.len() as u16, data, offset);
    let mut cursor = Cursor::new(data);
    cursor.set_position(new_offset);
    let _ = cursor.write_all(as_bytes);
    return cursor.position();
}

fn unmarshall_string(data: &[u8], offset: u64) -> (String, u64) {
    let (str_len, new_offset) = unmarshall_u16(data, offset);
    let mut cursor = Cursor::new(data);
    cursor.set_position(new_offset);
    let mut buffer = vec![0u8; str_len as usize];
    let result = cursor.read_exact(&mut buffer);
    debug_assert!(result.is_ok(), "Not enough bytes to read data");
    return (from_utf8(&buffer).unwrap().to_owned(), cursor.position());
}


fn marshall_qid(val: &QID, data: &mut [u8], offset: u64) -> u64 {
    let mut offset_tmp = offset;
    offset_tmp = marshall_u8(val.r#type, data, offset_tmp);
    offset_tmp = marshall_u32(val.version, data, offset_tmp);
    offset_tmp = marshall_u64(val.path, data, offset_tmp);
    return offset_tmp;
}

fn unmarshall_qid(data: &[u8], offset: u64) -> (QID, u64) {
    let offset_tmp = offset;
    let (r#type, offset_tmp2) = unmarshall_u8(data, offset_tmp);
    let (version, offset_tmp3) = unmarshall_u32(data, offset_tmp2);
    let (path, offset_tmp4) = unmarshall_u64(data, offset_tmp3);
    let res = QID {
        r#type: r#type,
        version: version,
        path: path
    };
    return (res, offset_tmp4);
}


// -------------------------------------------------
// --------------------- 9P ------------------------
// -------------------------------------------------
// Implementation of the 9p filesystem device following the
// 9P2000.L protocol ( https://code.google.com/p/diod/wiki/protocol )
// ported from v86 js emulator

// Feature bit (bit position) for mount tag.
const VIRTIO_9P_F_MOUNT_TAG : i32 = 0;
// Assumed max tag length in bytes.
const VIRTIO_9P_MAX_TAGLEN : i32 = 254;

const MAX_REPLYBUFFER_SIZE : i32 = 16 * 1024 * 1024;

// TODO
// flush

const SUCCESS : i32 = 0;

const EPERM  : i32 = 1;       /* Operation not permitted */
const ENOENT : i32 = 2;      /* No such file or directory */
const EEXIST : i32 = 17;      /* File exists */
const EINVAL : i32 = 22;     /* Invalid argument */
const EOPNOTSUPP : i32 = 95;  /* Operation is not supported */
const ENOTEMPTY : i32 = 39;  /* Directory not empty */
const EPROTO : i32 = 71;  /* Protocol error */


const P9_SETATTR_MODE : Number = 0x00000001;
const P9_SETATTR_UID : Number = 0x00000002;
const P9_SETATTR_GID : Number = 0x00000004;
const P9_SETATTR_SIZE : Number = 0x00000008;
const P9_SETATTR_ATIME : Number = 0x00000010;
const P9_SETATTR_MTIME : Number = 0x00000020;
const P9_SETATTR_CTIME : Number = 0x00000040;
const P9_SETATTR_ATIME_SET : Number = 0x00000080;
const P9_SETATTR_MTIME_SET : Number = 0x00000100;

const P9_STAT_MODE_DIR : Number = 0x80000000;
const P9_STAT_MODE_APPEND : Number = 0x40000000;
const P9_STAT_MODE_EXCL : Number = 0x20000000;
const P9_STAT_MODE_MOUNT : Number = 0x10000000;
const P9_STAT_MODE_AUTH : Number = 0x08000000;
const P9_STAT_MODE_TMP : Number = 0x04000000;
const P9_STAT_MODE_SYMLINK : Number = 0x02000000;
const P9_STAT_MODE_LINK : Number = 0x01000000;
const P9_STAT_MODE_DEVICE : Number = 0x00800000;
const P9_STAT_MODE_NAMED_PIPE : Number = 0x00200000;
const P9_STAT_MODE_SOCKET : Number = 0x00100000;
const P9_STAT_MODE_SETUID : Number = 0x00080000;
const P9_STAT_MODE_SETGID : Number = 0x00040000;
const P9_STAT_MODE_SETVTX : Number = 0x00010000;

const P9_LOCK_TYPE_RDLCK : i32 = 0;
const P9_LOCK_TYPE_WRLCK : i32 = 1;
const P9_LOCK_TYPE_UNLCK : i32 = 2;
const P9_LOCK_TYPES:  [&str; 3] = ["shared", "exclusive", "unlock"];

const P9_LOCK_FLAGS_BLOCK : i32 = 1;
const P9_LOCK_FLAGS_RECLAIM : i32 = 2;

const P9_LOCK_SUCCESS : i32 = 0;
const P9_LOCK_BLOCKED : i32 = 1;
const P9_LOCK_ERROR : i32 = 2;
const P9_LOCK_GRACE : i32 = 3;

const FID_NONE : i32 = -1;
const FID_INODE : i32 = 1;
const FID_XATTR : i32 = 2;













// -------------------------------------------------
// ----------------- FILESYSTEM---------------------
// -------------------------------------------------
// Implementation of a unix filesystem in memory.
// Ported to rust, from v86 js emulator

const S_IRWXUGO: i32 = 0x1FF;
const S_IFMT: i32 = 0xF000;
const S_IFSOCK: i32 = 0xC000;
const S_IFLNK: i32 = 0xA000;
const S_IFREG: i32 = 0x8000;
const S_IFBLK: i32 = 0x6000;
const S_IFDIR: i32 = 0x4000;
const S_IFCHR: i32 = 0x2000;

//var S_IFIFO  0010000
//var S_ISUID  0004000
//var S_ISGID  0002000
//var S_ISVTX  0001000

const O_RDONLY: i32 = 0x0000; // open for reading only
const O_WRONLY: i32 = 0x0001; // open for writing only
const O_RDWR: i32 = 0x0002; // open for reading and writing
const O_ACCMODE: i32 = 0x0003; // mask for above modes

const STATUS_INVALID: i32 = -0x1;
const STATUS_OK: i32 = 0x0;
const STATUS_ON_STORAGE: i32 = 0x2;
const STATUS_UNLINKED: i32 = 0x4;
const STATUS_FORWARDING: i32 = 0x5;


const JSONFS_VERSION: i32 = 3;


const JSONFS_IDX_NAME: i32 = 0;
const JSONFS_IDX_SIZE: i32 = 1;
const JSONFS_IDX_MTIME: i32 = 2;
const JSONFS_IDX_MODE: i32 = 3;
const JSONFS_IDX_UID: i32 = 4;
const JSONFS_IDX_GID: i32 = 5;
const JSONFS_IDX_TARGET: i32 = 6;
const JSONFS_IDX_SHA256: i32 = 6;

#[derive(Serialize, Deserialize, Debug)]
struct QIDCounter {
    last_qidnumber: u64
}

type EventFn = fn() -> ();

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    id: usize,
    #[serde(skip)]
    on_event: Option<EventFn>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UInt8Array {
    data: Box<[u8]>
}

impl UInt8Array {
    pub fn new(size: usize) -> Self {
        UInt8Array {
            data:  vec![0u8; size].into_boxed_slice(),
        }
    }

    pub fn clone_me(&self) -> UInt8Array {
        let mut new_data = vec![0u8; self.data.len()];
        new_data.copy_from_slice(
            &self.data[0..self.data.len()]
        );
        return UInt8Array {
            data: new_data.into_boxed_slice()
        };
    }
}


#[async_trait]
trait FileStorageInterface {
    /**
    * Read a portion of a file.
    * @param {string} sha256sum
    * @param {number} offset
    * @param {number} count
    * @return {!Promise<UInt8Array>} null if file does not exist.
    */
    async fn read(&mut self, sha256sum: String, offset: i32, count: i32) -> UInt8Array;
    /**
    * Add a read-only file to the filestorage.
    * @param {string} sha256sum
    * @param {!UInt8Array} data
    * @return {!Promise}
    */
    async fn cache(&mut self, sha256sum: String, data: UInt8Array) -> ();

    /**
    * Call this when the file won't be used soon, e.g. when a file closes or when this immutable
    * version is already out of date. It is used to help prevent accumulation of unused files in
    * memory in the long run for some FileStorage mediums.
    */
    async fn uncache(&mut self, sha256sum : String) -> ();
}

struct NotifyInfo {
    oldpath: String
}


struct SearchResult {
    id: Option<usize>,
    parentid: Option<usize>,
    name: String,
    forward_path: Option<String>
}
struct RecursiveListResult {
    parentid: usize,
    name: String
}




/**
 * @constructor
 * @param {!FileStorageInterface} storage
 * @param {{ last_qidnumber: number }=} qidcounter Another fs's qidcounter to synchronise with.
 */
 

// stub so serialization works
use std::fmt;
impl fmt::Debug for dyn FileStorageInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("")
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct FS{
    inodes : Vec<INode>,
    #[serde(skip)]
    events : Vec<Event>,
    #[serde(skip)]
    storage : Option<Arc<dyn FileStorageInterface>>,
    qidcounter : QIDCounter,
    inodedata : HashMap<usize, UInt8Array>,
    total_size : u64,
    used_size : u64,
    mounts : Vec<FSMountInfo>
}

impl FS {
    pub fn new(storage: Arc<dyn FileStorageInterface>, qidcounter : Option<QIDCounter>) -> FS {
        let mut result = FS {
            inodes : Vec::new(),
            events : Vec::new(),

            storage : Some(storage),

            qidcounter : match qidcounter {
                // The division was valid
                Some(x) => x,
                // The division was invalid
                None    => QIDCounter {
                    last_qidnumber: 0
                },
            },

            //this.tar = new TAR(this);

            inodedata : HashMap::new(),

            total_size : 256 * 1024 * 1024 * 1024,
            used_size : 0,

            mounts : Vec::new(),

            //RegisterMessage("LoadFilesystem", self.LoadFilesystem.bind(this) );
            //RegisterMessage("MergeFile", self.MergeFile.bind(this) );
            //RegisterMessage("tar",
            //    function(data) {
            //        SendToMaster("tar", self.tar.Pack(data));
            //    }.bind(this)
            //);
            //RegisterMessage("sync",
            //    function(data) {
            //        SendToMaster("sync", self.tar.Pack(data));
            //    }.bind(this)
            //);
        };

        result.create_directory(&"".to_owned(), None);

        result
    }
    /*

    public get_state() : Array<number | Array<INode> | Array<Array<number | UInt8Array>> | UInt8Array | FSMountInfo> {
        var state : Array<number | Array<INode> | Array<Array<number | UInt8Array>> | UInt8Array | FSMountInfo> = [];
    
        state[0] = self.inodes;
        state[1] = self.qidcounter.last_qidnumber;
        state[2] = [] as Array<Array<number | UInt8Array>>;
        for(const [id, data] of Object.entries(this.inodedata))
        {
            if((this.inodes[id].mode & S_IFDIR) === 0)
            {
                state[2].push([id, data] as Array<number | UInt8Array>);
            }
        }
        state[3] = self.total_size;
        state[4] = self.used_size;
        state = state.concat(this.mounts);
    
        return state;
    }
    

    public set_state(state : Array<number | Array<INode> | Array<Array<number | UInt8Array>> | FSMountInfo>) {
        self.inodes = state[0].map(state => { const inode = new Inode(0); inode.set_state(state); return inode; });
        self.qidcounter.last_qidnumber = state[1] as number;
        self.inodedata = {};
        for(let [key, value] of state[2])
        {
            if(value.buffer.byteLength !== value.byteLength)
            {
                // make a copy if we didn't get one
                value = value.slice();
            }
    
            self.inodedata[key] = value;
        }
        self.total_size = state[3] as number;
        self.used_size = state[4] as number;
        self.mounts = state.slice(5) as Array<FSMountInfo>;
    }
    */


    // -----------------------------------------------------

    pub fn add_event(&mut self, id : usize, on_event : EventFn) {
        let inode = &self.inodes[id];
        if inode.status == STATUS_OK || inode.status == STATUS_ON_STORAGE {
            on_event();
        }
        else if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            self.follow_fs_by_id_mut(mount_id)
                .add_event(foreign_id, on_event);
        }
        else
        {
            self.events.push(Event {id: id, on_event: Some(on_event)});
        }
    }
    

    pub fn handle_event(&mut self, id : usize) {
        let inode = &self.inodes[id];
        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            self.follow_fs_by_id_mut(mount_id)
                .handle_event(foreign_id);
        }
        //message.Debug("number of events: " + self.events.length);
        let mut newevents : Vec<Event> = Vec::new();
        for i in 0..self.events.len() {
            if self.events[i].id == id {
                self.events[i].on_event.unwrap()(); // unwrap than call
            } else {
                newevents.push(Event {
                    id: id,
                    on_event: Some(self.events[i].on_event.unwrap())
                });
            }
        }
        self.events = newevents;
    }


    /*
    public load_from_json(fs : Map<string, any>) : void {
        debug_assert!(fs, "Invalid fs passed to load_from_json");

        if(fs.get("version") !== JSONFS_VERSION)
        {
            throw "The filesystem JSON format has changed. " +
                "Please update your fs2json (https://github.com/copy/fs2json) and recreate the filesystem JSON.";
        }

        var fsroot = fs.get("fsroot");
        self.used_size = fs.get("size");

        for(var i = 0; i < fsroot.length; i++) {
            self.LoadRecursive(fsroot[i], 0);
        }

        //if(DEBUG)
        //{
        //    self.Check();
        //}
    }


    pub fun LoadRecursive(data : Map<String, any>, parentid : number) : void {
        let inode = self.create_inode();

        const name = data.get(JSONFS_IDX_NAME);
        inode.size = data.get(JSONFS_IDX_SIZE);
        inode.mtime = data.get(JSONFS_IDX_MTIME);
        inode.ctime = inode.mtime;
        inode.atime = inode.mtime;
        inode.mode = data.get(JSONFS_IDX_MODE);
        inode.uid = data.get(JSONFS_IDX_UID);
        inode.gid = data.get(JSONFS_IDX_GID);

        var ifmt = inode.mode & S_IFMT;

        if(ifmt === S_IFDIR)
        {
            self.PushInode(inode, parentid, name);
            self.LoadDir(this.inodes.length - 1, data.get(JSONFS_IDX_TARGET));
        }
        else if(ifmt === S_IFREG)
        {
            inode.status = STATUS_ON_STORAGE;
            inode.sha256sum = data.get(JSONFS_IDX_SHA256);
            debug_assert!(inode.sha256sum);
            self.PushInode(inode, parentid, name);
        }
        else if(ifmt === S_IFLNK)
        {
            inode.symlink = data.get(JSONFS_IDX_TARGET);
            self.PushInode(inode, parentid, name);
        }
        else if(ifmt === S_IFSOCK)
        {
            // socket: ignore
        }
        else
        {
            dbg_log("Unexpected ifmt: " + h(ifmt) + " (" + name + ")");
        }
    }

    public LoadDir(parentid : number, children : Array<Map<string, any>>) : void {
        for(var i = 0; i < children.length; i++) {
            self.LoadRecursive(children[i], parentid);
        }
    }
    */


    // -----------------------------------------------------
    
    /**
     * @private
     * @param {Inode} inode
     * @return {boolean}
     */
    pub fn should_be_linked(inode : &INode) -> bool {
        // Note: Non-root forwarder inode could still have a non-forwarder parent, so don't use
        // parent inode to check.
        return !FS::is_forwarder(inode) || (inode.foreign_id.is_some() && inode.foreign_id.unwrap() == 0);
    }

    /**
     * @private
     * @param {number} parentid
     * @param {number} idx
     * @param {string} name
     */
    pub fn link_under_dir(&mut self, parentid : usize, idx : usize, name : &String) {
        // Checks
        {
            debug_assert!(!FS::is_forwarder(&self.inodes[parentid]),
                "Filesystem: Shouldn't link under fowarder parents");
            debug_assert!(self.is_directory(parentid),
                "Filesystem: Can't link under non-directories");
            debug_assert!(FS::should_be_linked(&self.inodes[idx]),
                "Filesystem: Can't link across filesystems apart from their root");
            debug_assert!(self.inodes[idx].nlinks >= 0,
                "Filesystem: Found negative nlinks value of {}", self.inodes[idx].nlinks);
            debug_assert!(!self.inodes[parentid].direntries.contains_key(name),
                "Filesystem: Name '{}' is already taken", name);
        }
        let is_directory = self.is_directory(idx);

        {
            let parent_inode = &mut self.inodes[parentid];
            parent_inode.direntries.insert(name.clone(), idx);
            if is_directory {
                parent_inode.nlinks += 1;
            }
        }
        {
            let inode = &mut self.inodes[idx];
            inode.nlinks += 1;

            if is_directory
            {
                debug_assert!(!inode.direntries.contains_key(".."),
                    "Filesystem: Cannot link a directory twice");

                if !inode.direntries.contains_key(".") {
                    inode.nlinks += 1;
                }

                inode.direntries.insert(".".to_owned(), idx);

                inode.direntries.insert("..".to_owned(), parentid);
            }
        }
    }

    /**
     * @private
     * @param {number} parentid
     * @param {string} name
     */
    pub fn unlink_from_dir(&mut self, parentid : usize, name : &String) {
        
        debug_assert!(!FS::is_forwarder(&self.inodes[parentid]), "Filesystem: Can't unlink from forwarders");
        debug_assert!(self.is_directory(parentid), "Filesystem: Can't unlink from non-directories");

        let idx = self.search(parentid, name).unwrap();
        let is_directory = self.is_directory(idx);
        let parent_inode = &mut self.inodes[parentid];

        
        let exists = parent_inode.direntries.remove(name);
        if exists.is_none()
        {
            debug_assert!(false, "Filesystem: Can't unlink non-existent file: {}", name);
            return;
        }
        let remove_parent_links = {
            let inode = &mut self.inodes[idx];

            inode.nlinks -= 1;

            if is_directory
            {
                let up_dir = inode.direntries.get("..");
                debug_assert!(up_dir.is_some() && *up_dir.unwrap() == parentid,
                    "Filesystem: Found directory with bad parent id");

                inode.direntries.remove("..");
                true
            }
            else 
            {
                false
            }
        };
        

        if remove_parent_links {
            let parent_inode_2 = &mut self.inodes[parentid];
            parent_inode_2.nlinks -= 1;
        }

        let inode = &mut self.inodes[idx];

        debug_assert!(inode.nlinks >= 0,
            "Filesystem: Found negative nlinks value of {}", inode.nlinks);
    }
    
    pub fn push_inode(&mut self, mut inode : INode, parentid : Option<usize>, name : &String) {
        if parentid.is_some() {
            inode.fid = self.inodes.len();
            let inode_fid = inode.fid;
            self.inodes.push(inode);
            self.link_under_dir(parentid.unwrap(), inode_fid, name);
            return;
        } else {
            if self.inodes.len() == 0 { // if root directory
                inode.direntries.insert(".".to_owned(), 0);
                inode.direntries.insert("..".to_owned(), 0);
                inode.nlinks = 2;
                self.inodes.push(inode);
                return;
            }
        }

        debug_assert!(false, "Error in Filesystem: Pushed inode with name = {} has no parent", name)
    }
    
    /**
         * Clones given inode to new idx, effectively diverting the inode to new idx value.
         * Hence, original idx value is now free to use without losing the original information.
         * @private
         * @param {number} parentid Parent of target to divert.
         * @param {string} filename Name of target to divert.
         * @return {number} New idx of diversion.
         */
    pub fn divert(&mut self, parentid : usize, filename : &String) -> usize {
        let old_idx = self.search(parentid, filename);
        debug_assert!(old_idx.is_some(), "Filesystem divert: name ({}) not found", filename);
        let old_inode = &self.inodes[old_idx.unwrap()];
        
        debug_assert!(self.is_directory(old_idx.unwrap()) || old_inode.nlinks <= 1,
            "Filesystem: can't divert hardlinked file '{}' with nlinks={}",
            filename, old_inode.nlinks);

        // Shallow copy is alright.
        let new_inode = &mut INode::new(self.qidcounter.last_qidnumber);

        FS::copy_inode(old_inode, new_inode);
        new_inode.nlinks = old_inode.nlinks;
        new_inode.direntries = old_inode.direntries.clone();

        // precompute to make borrow checker happy
        let old_is_forwarder = FS::is_forwarder(old_inode);
        let old_should_be_linked = FS::should_be_linked(old_inode);
        let old_mount_id = old_inode.mount_id;
        let old_foreign_id = old_inode.foreign_id;
        let idx = self.inodes.len();

        new_inode.fid = idx;

        self.inodes.push(std::mem::replace(new_inode, INode::new(0)));



        // Relink references
        if old_is_forwarder
        {
            self.mounts[old_mount_id.unwrap()].backtrack.insert(old_foreign_id.unwrap(), idx);
        }
        if old_should_be_linked
        {
            self.unlink_from_dir(parentid, filename);
            self.link_under_dir(parentid, idx, filename);
        }

        let new_inode_ref = &self.inodes[idx];

        // Update children
        if self.is_directory(old_idx.unwrap()) && !old_should_be_linked
        {
            // need to make a running list to appease borrow checker
            let mut child_ids_to_connect : Vec<usize> = Vec::new();

            for (name, child_id) in new_inode_ref.direntries.iter()
            {
                if name == "." || name == ".." {
                    continue;
                }
                if self.is_directory(*child_id)
                {
                    child_ids_to_connect.push(*child_id);
                }
            }
            for child_id in child_ids_to_connect {
                self.inodes[child_id].direntries.insert("..".to_owned(), idx);
            }
        }

        // Relocate local data if any.
        if let Some(old_inode_data) = self.inodedata.remove(&old_idx.unwrap()) {
            self.inodedata.insert(idx, old_inode_data);
        }
        let old_inode_mut = &mut self.inodes[old_idx.unwrap()];
        // Retire old reference information.
        old_inode_mut.direntries = HashMap::new();
        old_inode_mut.nlinks = 0;

        return idx;
    }
    


    /**
     * Copy all non-redundant info.
     * References left untouched: local idx value and links
     * @private
     * @param {!Inode} src_inode
     * @param {!Inode} dest_inode
     */
    pub fn copy_inode(src_inode : &INode, dest_inode : &mut INode) {
        // dest_inode.direntries = src_inode.direntries
        dest_inode.status = src_inode.status;
        dest_inode.size = src_inode.size;
        dest_inode.uid = src_inode.uid;
        dest_inode.gid = src_inode.gid;
        // dest_inode.fid = src_inode.fid
        dest_inode.ctime = src_inode.ctime;
        dest_inode.atime = src_inode.atime;
        dest_inode.mtime = src_inode.mtime;
        dest_inode.major = src_inode.major;
        dest_inode.minor = src_inode.minor;
        dest_inode.symlink = src_inode.symlink.clone();
        dest_inode.mode = src_inode.mode;
        dest_inode.qid = QID {
            r#type: src_inode.qid.r#type,
            version: src_inode.qid.version,
            path: src_inode.qid.path
        };
        dest_inode.caps = src_inode.caps.as_ref().map(|x| x.clone_me());
        // dest_inode.nlinks = src_inode.nlinks
        dest_inode.sha256sum = src_inode.sha256sum.clone();
        dest_inode.locks = src_inode.locks.clone();
        dest_inode.mount_id = src_inode.mount_id;
        dest_inode.foreign_id = src_inode.foreign_id;
    }
    

    pub fn create_inode(&mut self) -> INode {
        //console.log("CreateInode", Error().stack);
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let millis = since_the_epoch.as_millis();
        // Divide by 1000 and round
        let now = (millis as f64 / 1000.0).round() as u64;
        self.qidcounter.last_qidnumber += 1;
        let mut inode = INode::new(self.qidcounter.last_qidnumber);
        
        inode.mtime = now;
        inode.atime = now;
        inode.ctime = now;
        return inode;
    }

    // Note: parentid = -1 for initial root directory.
    pub fn create_directory(&mut self, name: &String, parentid: Option<usize>) -> usize {
        
        if parentid.is_some() {
            let parent_node = &self.inodes[parentid.unwrap()];
            if FS::is_forwarder(parent_node)
            {
                let mount_id = parent_node.mount_id.unwrap();
                let foreign_parentid = parent_node.foreign_id;
                let foreign_fs: &mut FS = self.follow_fs_by_id_mut(mount_id);
                let foreign_id = foreign_fs.create_directory(name, foreign_parentid);
                return self.create_forwarder(mount_id, foreign_id);
            }
        }
        
        let mut x = self.create_inode();
        x.mode = 0x01FF | S_IFDIR;
        if parentid.is_some() {
            x.uid = self.inodes[parentid.unwrap()].uid;
            x.gid = self.inodes[parentid.unwrap()].gid;
            x.mode = (self.inodes[parentid.unwrap()].mode & 0x1FF) | S_IFDIR;
        }
        x.qid.r#type = (S_IFDIR >> 8) as u8;
        self.push_inode(x, parentid, name);
        self.notify_listeners(self.inodes.len()-1, &"newdir".to_owned());
        return self.inodes.len()-1;
    }
    

    pub fn create_file(&mut self, filename : &String, parentid : usize) -> usize {
        let parent_inode = &self.inodes[parentid];
        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let foreign_parentid = parent_inode.foreign_id.unwrap();
            let foreign_id = self.follow_fs_by_id_mut(parent_inode_mount_id)
                .create_file(filename, foreign_parentid);
            return self.create_forwarder(parent_inode_mount_id, foreign_id);
        }
        let mut x = self.create_inode();
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = (S_IFREG >> 8) as u8;
        x.mode = (self.inodes[parentid].mode & 0x1B6) | S_IFREG;
        self.push_inode(x, Some(parentid), filename);
        self.notify_listeners(self.inodes.len()-1, &"newfile".to_owned());
        return self.inodes.len()-1;
    }

    /*
    public CreateNode(filename: string, parentid: number, major: number, minor: number) : number {
        const parent_inode = self.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id =
                self.follow_fs(parent_inode).CreateNode(filename, foreign_parentid, major, minor);
            return self.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = self.CreateInode();
        x.major = major;
        x.minor = minor;
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = S_IFSOCK >> 8;
        x.mode = (this.inodes[parentid].mode & 0x1B6);
        self.PushInode(x, parentid, filename);
        return self.inodes.length-1;
    }

    public CreateSymlink(filename : string, parentid: number, symlink : string) : number {
        const parent_inode = self.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id =
                self.follow_fs(parent_inode).CreateSymlink(filename, foreign_parentid, symlink);
            return self.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = self.CreateInode();
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = S_IFLNK >> 8;
        x.symlink = symlink;
        x.mode = S_IFLNK;
        self.PushInode(x, parentid, filename);
        return self.inodes.length-1;
    }

    public CreateTextFile(filename : string, parentid : number, str : string) : number {
        const parent_inode = self.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = await
                self.follow_fs(parent_inode).CreateTextFile(filename, foreign_parentid, str);
            return self.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var id = self.CreateFile(filename, parentid);
        var x = self.inodes[id];
        var data = new UInt8Array(str.length);
        x.size = str.length;
        for(var j = 0; j < str.length; j++) {
            data[j] = str.charCodeAt(j);
        }
        await self.set_data(id, data);
        return id;
    }

    /**
     * @param {UInt8Array} buffer
     */
    public async CreateBinaryFile(filename : string, parentid : number, buffer : UInt8Array) : number {
        const parent_inode = self.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = await
                self.follow_fs(parent_inode).CreateBinaryFile(filename, foreign_parentid, buffer);
            return self.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var id = self.CreateFile(filename, parentid);
        var x = self.inodes[id];
        var data = new UInt8Array(buffer.length);
        data.set(buffer);
        await self.set_data(id, data);
        x.size = buffer.length;
        return id;
    }


    public OpenInode(id : number, mode : number) : boolean {
        var inode = self.inodes[id];
        if(this.is_forwarder(inode))
        {
            return self.follow_fs(inode).OpenInode(inode.foreign_id, mode);
        }
        if((inode.mode&S_IFMT) === S_IFDIR) {
            self.FillDirectory(id);
        }
        /*
        var type = "";
        switch(inode.mode&S_IFMT) {
            case S_IFREG: type = "File"; break;
            case S_IFBLK: type = "Block Device"; break;
            case S_IFDIR: type = "Directory"; break;
            case S_IFCHR: type = "Character Device"; break;
        }
        */
        //message.Debug("open:" + self.GetFullPath(id) +  " type: " + inode.mode + " status:" + inode.status);
        return true;
    }

    public async CloseInode(id : number) : Promise {
        //message.Debug("close: " + self.GetFullPath(id));
        var inode = self.inodes[id];
        if(this.is_forwarder(inode))
        {
            return await self.follow_fs(inode).CloseInode(inode.foreign_id);
        }
        if(inode.status === STATUS_ON_STORAGE)
        {
            self.storage.uncache(inode.sha256sum);
        }
        if(inode.status === STATUS_UNLINKED) {
            //message.Debug("Filesystem: Delete unlinked file");
            inode.status = STATUS_INVALID;
            await self.DeleteData(id);
        }
    }
    */
    /**
     * @return {!Promise<number>} 0 if success, or errno if failured.
     */
    pub fn rename(&mut self, olddirid: usize, oldname: &String, newdirid: usize, newname: &String) -> i32 {
        // message.Debug("Rename " + oldname + " to " + newname);
        if (olddirid == newdirid) && (oldname == newname) {
            return SUCCESS;
        }
        let oldid = self.search(olddirid, oldname);
        if oldid.is_none()
        {
            return ENOENT;
        }

        // For event notification near end of method.
        let oldpath = self.get_full_path(olddirid) + "/" + oldname;

        let newid = self.search(newdirid, newname);
        if newid.is_some() {
            let ret = self.unlink(newdirid, newname);
            if ret != SUCCESS {
                return ret;
            }
        }

        // precompute for borrow checker
        let olddir = &self.inodes[olddirid];
        let olddir_is_forwarder = FS::is_forwarder(olddir);
        let newdir = &self.inodes[newdirid];
        let newdir_is_forwarder = FS::is_forwarder(newdir);
        let olddir_mount_id = olddir.mount_id;
        let olddir_foreign_id = olddir.foreign_id;
        let newdir_mount_id = newdir.mount_id;
        let newdir_foreign_id = newdir.foreign_id;

        let idx = oldid.unwrap(); // idx contains the id which we want to rename
        //let inode = &mut self.inodes[idx];
        //let newdir = &self.inodes[newdirid];

        if !olddir_is_forwarder && !newdir_is_forwarder
        {
            // Move inode within current filesystem.

            self.unlink_from_dir(olddirid, oldname);
            self.link_under_dir(newdirid, idx, newname);

            self.inodes[idx].qid.version += 1;
        }
        else if olddir_is_forwarder
            && newdir_mount_id.is_some()
            && olddir_mount_id.unwrap() == newdir_mount_id.unwrap()
        {
            // Move inode within the same child filesystem.
            
            let ret = self.follow_fs_by_id_mut(olddir_mount_id.unwrap())
                .rename(olddir_foreign_id.unwrap(), oldname, newdir_foreign_id.unwrap(), newname);

            if ret != SUCCESS {
                return ret;
            }
        }
        else if self.is_a_root(idx)
        {
            // The actual inode is a root of some descendant filesystem.
            // Moving mountpoint across fs not supported - needs to update all corresponding forwarders.
            print_debug(format!("XXX: Attempted to move mountpoint ({}) - skipped", oldname));
            return EPERM;
        }
        else if !self.is_directory(idx) && self.get_inode(idx).nlinks > 1
        {
            // Move hardlinked inode vertically in mount tree.
            print_debug(format!("XXX: Attempted to move hardlinked file ({}) across filesystems - skipped", oldname));
            return EPERM;
        }
        else
        {
            // Jump between filesystems.

            // Can't work with both old and new inode information without first diverting the old
            // information into a new idx value.
            let diverted_old_idx = self.divert(olddirid, oldname);
            let old_real_inode = self.get_inode(idx);
            let old_real_inode_size = old_real_inode.size;

            // DATA WAS HERE


            if newdir_is_forwarder
            {
                // we'd like to do the copy_inode below with &self.get_inode(idx) but that takes immutable
                // and mutable ref to self which we can't do, so we copy data first
                let inode_source = &self.get_inode(idx);
                let new_inode = &mut INode::new(inode_source.qid.path);

                FS::copy_inode(inode_source, new_inode);
        
                // Create new inode.
                let diverted_old_idx_is_directory = self.is_directory(diverted_old_idx);
                let foreign_fs = self.follow_fs_by_id_mut(newdir_mount_id.unwrap());
                let foreign_id = if diverted_old_idx_is_directory
                    { foreign_fs.create_directory(newname, Some(newdir_foreign_id.unwrap())) }
                    else { foreign_fs.create_file(newname, newdir_foreign_id.unwrap()) };

                let new_real_inode = foreign_fs.get_inode_mutable(foreign_id);
                FS::copy_inode(new_inode, new_real_inode);

                // Point to this new location.
                self.set_forwarder(idx, newdir_mount_id.unwrap(), foreign_id);
            }
            else
            {
                // delete forwarder had to be inlined bc of borrow checker
                // self.delete_forwarder(&mut self.inodes[idx]);
                debug_assert!(FS::is_forwarder(&self.inodes[idx]), "Filesystem delete_forwarder: expected forwarder");

                let delete_forwarder_inode = &mut self.inodes[idx];
                delete_forwarder_inode.status = STATUS_INVALID;
                let delete_forwarder_inode_mount_id = delete_forwarder_inode.mount_id.unwrap();
                let delete_forwarder_inode_foreign_id = delete_forwarder_inode.foreign_id.unwrap();
                self.mounts[delete_forwarder_inode_mount_id].backtrack.remove(&delete_forwarder_inode_foreign_id);
                
                // copy data for borrow checker
                let inode_source = &self.get_inode(idx);
                let new_inode = &mut INode::new(inode_source.qid.path);
                FS::copy_inode(inode_source, new_inode);
        
                // Replace current forwarder with real inode.
                FS::copy_inode(new_inode, &mut self.inodes[idx]);

                // Link into new location in this filesystem.
                self.link_under_dir(newdirid, idx, newname);
            }

            // Rewrite data to newly created destination.
            self.change_size(idx, old_real_inode_size);
            
            // NOTE: MOVED THIS FROM ABOVE, undo if that breaks something (as far as I could tell it doesn't)
            let data_optional = self.read(diverted_old_idx, 0, old_real_inode_size);

            if let Some(data) = data_optional {
                if !data.is_empty() {
                    let len = data.len();
                    let owned_data = data.to_vec(); // Convert &[u8] to Vec<u8>
                    self.write_arr(idx, 0, len, Some(&owned_data));
                }
            }

            // Move children to newly created destination.
            if self.is_directory(idx)
            {
                for child_filename in self.get_children(diverted_old_idx)
                {
                    let ret = self.rename(diverted_old_idx, &child_filename, idx, &child_filename);
                    if ret != SUCCESS {
                        return ret;
                    }
                }
            }

            // Perform destructive changes only after migration succeeded.
            self.delete_data(diverted_old_idx);
            let ret = self.unlink(olddirid, oldname);
            if ret != SUCCESS {
                return ret;
            }
        }

        self.notify_listeners_with_info(idx,
            &"rename".to_owned(),
            NotifyInfo {
                oldpath: oldpath
            }
        );

        return SUCCESS;
    }

    pub fn write(&mut self, id : usize, offset : usize, count : usize, buffer : Option<&UInt8Array>) {
        self.write_arr(id, offset, count, buffer.map(|x| x.data.as_ref()));
    }

    pub fn write_arr(&mut self, id : usize, offset : usize, count : usize, buffer : Option<&[u8]>) {
        self.notify_listeners(id, &"write".to_owned());
        let inode = &self.inodes[id];

        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            self.follow_fs_by_id_mut(mount_id)
                .write_arr(foreign_id, offset, count, buffer);
            return;
        }

        let bigger_size = ((((offset + count) as f64)  * 3.0) / 2.0).floor() as usize;

        // gotta do everything using entry to avoid double mutable self accesses

        // Add if doesn't exist
        self.inodedata.entry(id)
            // Would love to only do a single entry, however this cannot happen before and_modify bc wrong return value :(
            .or_insert_with(|| UInt8Array::new(bigger_size));

        self.inodedata.entry(id)
            // resize if too small
            .and_modify(|data : &mut UInt8Array| {
                let old_size = data.data.len();
                if old_size < (offset + count) {
                    let mut new_data = vec![0u8; bigger_size].into_boxed_slice();
                    new_data[0..old_size].copy_from_slice(
                        &data.data[0..old_size]
                    );
                    data.data = new_data;
                }
            // write data
            }).and_modify(|data : &mut UInt8Array| {
                if buffer.is_some()
                {
                    data.data[offset..offset+count].copy_from_slice(
                        &buffer.unwrap()[0..count]);
                }        
            });

        self.inodes[id].size = offset + count;
    }
    
    pub fn read(&self, inodeid : usize, offset : usize, count : usize) -> Option<&[u8]> {
        let inode = &self.inodes[inodeid];
        if FS::is_forwarder(&inode)
        {
            let foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_immutable(inode).read(foreign_id, offset, count);
        }

        return self.get_data(inodeid, offset, count);
    }
    
    pub fn search(&mut self, parentid : usize, name : &String) -> Option<usize> {
        let parent_inode = &self.inodes[parentid];

        if FS::is_forwarder(parent_inode)
        {
            let mount_id = parent_inode.mount_id.unwrap();
            let foreign_parentid = parent_inode.foreign_id.unwrap();
            let foreign_id = self.follow_fs_by_id_mut(mount_id).
                search(foreign_parentid, name);
            if foreign_id.is_some()
            {
                return Some(self.get_forwarder(mount_id, foreign_id.unwrap()));
            }
            else 
            {
                return None;
            }
        }

        return parent_inode.direntries.get(name).map(|&x| x);
    }
    

    pub fn count_used_inodes(&self) -> u64 {
        let mut count = self.inodes.len() as u64;
        for mount_info in &self.mounts
        {
            count += mount_info.fs.count_used_inodes();

            // Forwarder inodes don't count.
            count -= mount_info.backtrack.len() as u64;
        }
        return count;
    }

    pub fn count_free_inodes(&self) -> u64 {
        let mut count = 1024 * 1024;
        for mount_info in &self.mounts
        {
            count += mount_info.fs.count_free_inodes();
        }
        return count;
    }

    pub fn get_total_size(&self) -> u64 {
        let mut size = self.used_size;
        for mount_info in &self.mounts
        {
            size += mount_info.fs.get_total_size();
        }
        return size;
        //var size = 0;
        //for(var i=0; i<this.inodes.length; i++) {
        //    var d = self.inodes[i].data;
        //    size += d ? d.length : 0;
        //}
        //return size;
    }

    pub fn get_space(&self) -> u64 {
        let mut size = self.total_size;
        for mount_info in &self.mounts
        {
            size += mount_info.fs.get_space();
        }
        // Typo? I think this should be size?
        return self.total_size;
    }
    /**
     * XXX: Not ideal.
     * @param {number} idx
     * @return {string}
     */
    pub fn get_directory_name(&mut self, idx : usize) -> String {
        let parent_idx = self.get_parent(idx);
        // Root directory.
        if parent_idx.is_none() {
            return "".to_owned();
        }

        let parent_inode = &self.inodes[parent_idx.unwrap()];

        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let foreign_id = self.inodes[idx].foreign_id.unwrap();
            return self.follow_fs_by_id_mut(parent_inode_mount_id)
                .get_directory_name(foreign_id);
        }

        for (name, childid) in parent_inode.direntries.iter()
        {
            if *childid == idx {
                return name.clone();
            }
        }

        debug_assert!(false, "Filesystem: Found directory inode whose parent doesn't link to it");
        return "".to_owned();
    }

    pub fn get_full_path(&mut self, idx : usize) -> String {
        debug_assert!(self.is_directory(idx), "Filesystem: Cannot get full path of non-directory inode");

        let mut path : Vec<String> = Vec::new();

        let mut working_idx = idx;

        while working_idx != 0 {
            path.push(self.get_directory_name(idx));
            path.push("/".to_owned());
            let parent_idx = self.get_parent(working_idx);
            if parent_idx.is_some() {
                working_idx = parent_idx.unwrap()
            }
            else {
                break;
            }
        }
        path.reverse();
        return path[1..].join("").to_owned();
    }
    
    /**
     * @param {number} parentid
     * @param {number} targetid
     * @param {string} name
     * @return {number} 0 if success, or errno if failured.
     */
    pub fn link(&mut self, parentid : usize, targetid : usize, name : &String) -> i32 {
        if self.is_directory(targetid)
        {
            return EPERM;
        }

        let parent_inode = &self.inodes[parentid];
        let inode = &self.inodes[targetid];

        if FS::is_forwarder(parent_inode)
        {
            if !FS::is_forwarder(inode) || inode.mount_id.unwrap() != parent_inode.mount_id.unwrap()
            {
                print_debug("XXX: Attempted to hardlink a file into a child filesystem - skipped".to_owned());
                return EPERM;
            }
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let parent_inode_foreign_id = parent_inode.foreign_id.unwrap();
            let inode_foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_by_id_mut(parent_inode_mount_id)
                .link(parent_inode_foreign_id, inode_foreign_id, name);
        }

        if FS::is_forwarder(inode)
        {
            print_debug("XXX: Attempted to hardlink file across filesystems - skipped".to_owned());
            return EPERM;
        }

        self.link_under_dir(parentid, targetid, name);
        return SUCCESS;
    }

    pub fn unlink(&mut self, parentid : usize, name : &String) -> i32 {
        if name == "." || name == ".."
        {
            // Also guarantees that root cannot be deleted.
            return EPERM;
        }
        let idx_cond = self.search(parentid, name);

        if idx_cond.is_none() {
            return ENOENT;
        }

        let idx = idx_cond.unwrap();

        let inode = &self.inodes[idx];
        let parent_inode = &self.inodes[parentid];
        //message.Debug("Unlink " + inode.name);

        // forward if necessary
        if FS::is_forwarder(parent_inode)
        {
            debug_assert!(FS::is_forwarder(inode), "Children of forwarders should be forwarders");

            let foreign_parentid = parent_inode.foreign_id;
            let mount_id = parent_inode.mount_id;
            return self.follow_fs_by_id_mut(mount_id.unwrap())
                .unlink(foreign_parentid.unwrap(), name);

            // Keep the forwarder dangling - file is still accessible.
        }

        if self.is_directory(idx) && !self.is_empty(idx)
        {
            return ENOTEMPTY;
        }

        self.unlink_from_dir(parentid, name);
        
        let inode_mut : &mut INode = &mut self.inodes[idx];
        if inode_mut.nlinks == 0
        {
            // don't delete the content. The file is still accessible
            inode_mut.status = STATUS_UNLINKED;
            self.notify_listeners(idx, &"delete".to_owned());
        }
        return SUCCESS;
    }
    
    pub fn delete_data(&mut self, idx : usize) {
        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            self.follow_fs_by_id_mut(mount_id)
                .delete_data(foreign_id);
        }
        self.inodedata.remove(&idx);
        let mut_inode = &mut self.inodes[idx];
        mut_inode.size = 0;
    }

    /**
     * @private
     * @param {number} idx
     * @return {!Promise<UInt8Array>} The buffer that contains the file contents, which may be larger
     *      than the data itself. To ensure that any modifications done to this buffer is reflected
     *      to the file, call set_data with the modified buffer.
     */
    pub fn get_buffer(&self, idx : usize) -> Option<&UInt8Array> {
        debug_assert!(idx < self.inodes.len(), "Filesystem get_buffer: idx {} does not point to an inode", idx);
        return self.inodedata.get(&idx);
    }

    pub fn get_buffer_mut(&mut self, idx : usize) -> Option<&mut UInt8Array> {
        debug_assert!(idx < self.inodes.len(), "Filesystem get_buffer: idx {} does not point to an inode", idx);
        return self.inodedata.get_mut(&idx);
    }
    /**
     * @private
     * @param {number} idx
     * @param {number} offset
     * @param {number} count
     * @return {!Promise<UInt8Array>}
     */
    pub fn get_data(&self, idx : usize, offset : usize, count : usize) -> Option<&[u8]> {
        debug_assert!(idx < self.inodes.len(), "Filesystem get_data: idx {} does not point to an inode", idx);
        
        if self.inodedata.contains_key(&idx)
        {
            return Some(&self.inodedata[&idx].data[offset..offset + count]);
        }
        else
        {
            return None;
        }
    }
    /**
     * @private
     * @param {number} idx
     * @param {UInt8Array} buffer
     */
    pub fn set_data(&mut self, idx : usize, buffer : UInt8Array) {
        // Current scheme: Save all modified buffers into local inodedata.
        self.inodedata.insert(idx, buffer);
    }
    
    /**
     * @param {number} idx
     * @return {!Inode}
     */
    pub fn get_inode(&self, idx : usize) -> &INode {
        //debug_assert!(!isNaN(idx), "Filesystem GetInode: NaN idx");
        debug_assert!(idx < self.inodes.len(), "Filesystem GetInode: out of range idx: {}", idx);

        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_by_id(mount_id)
                .get_inode(foreign_id);
        }

        return inode;
    }

    pub fn get_inode_mutable(&mut self, idx : usize) -> &mut INode {
        //debug_assert!(!isNaN(idx), "Filesystem GetInode: NaN idx");
        debug_assert!(idx < self.inodes.len(), "Filesystem GetInode: out of range idx: {}", idx);

        let inode_immutable : &INode = &self.inodes[idx];
        if FS::is_forwarder(inode_immutable)
        {
            let mount_id = inode_immutable.mount_id.unwrap();
            let foreign_id = inode_immutable.foreign_id.unwrap();
            return self.follow_fs_by_id_mut(mount_id)
                .get_inode_mutable(foreign_id);
        }
        let inode: &mut INode = &mut self.inodes[idx];

        return inode;
    }
    pub fn change_size(&mut self, idx : usize, newsize : usize) {
        let inode : &mut INode = self.get_inode_mutable(idx);
        //message.Debug("change size to: " + newsize);
        if newsize == inode.size {
            return;
        }
        inode.size = newsize;

        // use entry to avoid multiple mutable accesses to self
        self.inodedata.entry(idx)
            // resize if exists
            .and_modify(|data : &mut UInt8Array| {
                let old_size = data.data.len();
                let mut new_data = vec![0u8; newsize].into_boxed_slice();
                let copy_size = min(old_size, newsize);
                new_data[0..copy_size].copy_from_slice(
                    &data.data[0..copy_size]
                );
                data.data = new_data;
            })
            // insert if not exists
            .or_insert_with(|| UInt8Array::new(newsize));
        
    }

    pub fn search_path(&mut self, path : &String) -> SearchResult {
        //path = path.replace(/\/\//g, "/");
        let path_fixed = path.replace("//", "/");
        let mut walk: Vec<&str> = path_fixed.split('/').collect();
        if walk.len() > 0 && walk[walk.len() - 1].len() == 0 {
            let _  = walk.pop();
        }
        if walk.len() > 0 && walk[0].len() == 0 {
            let _ = walk.remove(0);
        }
        let n = walk.len();

        let mut parentid : Option<usize> = None;
        let mut id : Option<usize> = Some(0);
        let mut forward_path : Option<String> = None;
        for i in 0..n {
            parentid = id;
            if parentid.is_some() {
                id = self.search(parentid.unwrap(), &walk[i].to_owned());
                if forward_path.is_none() 
                    && FS::is_forwarder(&self.inodes[parentid.unwrap()])
                {
                    forward_path = Some("/".to_owned() + &walk[i..].join("/"));
                }
            }
            else {
                id = None;
            }
            if id.is_none() {
                if i < n-1 {
                    return SearchResult {
                        id: None,
                        parentid: None,
                        name: walk[i].to_owned(),
                        forward_path: forward_path
                    };
                }
                return SearchResult {
                    id: None,
                    parentid: parentid,
                    name: walk[i].to_owned(),
                    forward_path: forward_path
                };
            }
        }
        return SearchResult {
            id: id,
            parentid: parentid,
            name: walk[walk.len() - 1].to_owned(),
            forward_path: forward_path
        };
    }
    // -----------------------------------------------------


    /**
     * @param {number} dirid
     * @param {Array<{parentid: number, name: string}>} list
     */
    pub fn get_recursive_list(&mut self, dirid : usize, list : &mut Vec<RecursiveListResult>) {
        if FS::is_forwarder(&self.inodes[dirid])
        {
            let mount_id = self.inodes[dirid].mount_id.unwrap();
            let foreign_dirid = self.inodes[dirid].foreign_id.unwrap();
            let foreign_start = list.len();
            let foreign_fs = self.follow_fs_by_id_mut(mount_id);
            
            foreign_fs.get_recursive_list(foreign_dirid, list);
            for i in foreign_start..list.len()
            {
                list[i].parentid = self.get_forwarder(mount_id, list[i].parentid);
            }
            return;
        }
        // we need to do make subdir array to make borrow checker happy since get_forwarder is mut
        // theoretically we could use recursive closure and avoid allocation, but don't do that https://smallcultfollowing.com/babysteps/blog/2013/04/30/the-case-of-the-recurring-closure/ https://stackoverflow.com/questions/16946888/is-it-possible-to-make-a-recursive-closure-in-rust
        let mut subdirs : Vec<usize> = Vec::new();
        for (name, id) in self.inodes[dirid].direntries.iter()
        {
            if name != "." && name != ".."
            {
                list.push( RecursiveListResult {
                    parentid: dirid,
                    name: name.clone() 
                });

                if self.is_directory(*id)
                {
                    subdirs.push(*id);
                }
            }
        }
        for subdir_id in subdirs {
            self.get_recursive_list(subdir_id, list)
        }
    }
    

    pub fn recursive_delete(&mut self, path : &String) {
        let mut to_delete : Vec<RecursiveListResult> = Vec::new();
        let ids = self.search_path(path);
        if ids.id.is_none() {
            return;
        }

        self.get_recursive_list(ids.id.unwrap(), &mut to_delete);

        for i in (0..to_delete.len()).rev()
        {
            let ret = self.unlink(to_delete[i].parentid, &to_delete[i].name);
            debug_assert!(ret == SUCCESS, "Filesystem RecursiveDelete failed at parent={}, name='{}' with error code: {}", to_delete[i].parentid, to_delete[i].name, -ret);
        }
    }

    pub fn delete_node(&mut self, path : &String) {
        let ids = self.search_path(path);
        if ids.id.is_none() {
            return;
        } 

        if (self.inodes[ids.id.unwrap()].mode & S_IFMT) == S_IFREG{
            let ret = self.unlink(ids.parentid.unwrap(), &ids.name);
            debug_assert!(ret == SUCCESS, "Filesystem DeleteNode failed with error code: {}", -ret);
        }
        else if (self.inodes[ids.id.unwrap()].mode & S_IFMT) == S_IFDIR {
            self.recursive_delete(path);
            let ret = self.unlink(ids.parentid.unwrap(), &ids.name);
            debug_assert!(ret == SUCCESS, "Filesystem DeleteNode failed with error code: {}", -ret);
        }
    }
    
    pub fn notify_listeners(&self, id: usize, action: &String) {

    }

    /** @param {*=} info */
    pub fn notify_listeners_with_info(&self, id: usize, action: &String, info: NotifyInfo) {
        //if(info==undefined)
        //    info = {};

        //var path = self.GetFullPath(id);
        //if (this.watchFiles[path] === true && action=='write') {
        //  message.Send("WatchFileEvent", path);
        //}
        //for (var directory of self.watchDirectories) {
        //    if (this.watchDirectories.hasOwnProperty(directory)) {
        //        var indexOf = path.indexOf(directory)
        //        if(indexOf === 0 || indexOf === 1)
        //            message.Send("WatchDirectoryEvent", {path: path, event: action, info: info});
        //    }
        //}
    }


    pub fn check(&mut self) {
        for i in 1..self.inodes.len()
        {
            if self.inodes[i].status == STATUS_INVALID {
                continue;
            } 

            let inode = self.get_inode(i);
            if inode.nlinks < 0 {
                print_debug(format!("Error in filesystem: negative nlinks={} at id ={}", inode.nlinks, i).to_owned());
            }

            if self.is_directory(i)
            {
                if self.is_directory(i) && self.get_parent(i).is_none() {
                    print_debug(format!("Error in filesystem: negative parent id {}", i).to_owned());
                }
                
                let inode_const = self.get_inode(i);
                for (name, id) in inode_const.direntries.iter()
                {
                    if name.len() == 0 {
                        print_debug(format!("Error in filesystem: inode with no name and id {}", id).to_owned());
                    }

                    for c in name.chars() {
                        if (c as u32) < 32 {
                            print_debug("Error in filesystem: Unallowed char in filename".to_owned());
                        }
                    }
                }
            }
        }
    }

    pub fn fill_directory(&mut self, dirid: usize) {
        let inode = &self.inodes[dirid];
        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            // XXX: The ".." of a mountpoint should point back to an inode in this fs.
            // Otherwise, ".." gets the wrong qid and mode.
            self.follow_fs_by_id_mut(mount_id)
                .fill_directory(foreign_id);
            return;
        }

        let mut size = 0;
        for name in inode.direntries.keys()
        {
            let as_bytes: &[u8] = name.as_bytes();
            size += 13 + 8 + 1 + 2 + as_bytes.len();
        }
        let mut uint8array = UInt8Array::new(size);

        let data: &mut [u8] = &mut *uint8array.data;
        
        let mut offset : u64 = 0x0;
        for (name, id) in inode.direntries.iter()
        {
            let as_bytes: &[u8] = name.as_bytes();
            let child = self.get_inode(*id);
            let end_pos : u64 = offset+13+8+1+2+(as_bytes.len() as u64);
            offset = marshall_qid(&child.qid, data, offset);
            offset = marshall_u64(end_pos, data, offset);
            offset = marshall_u8((child.mode >> 12) as u8, data, offset);
            offset = marshall_string(&name, data, offset);
        }        
        self.inodes[dirid].size = size;
        
        self.inodedata.insert(dirid, uint8array);
    }

    pub fn round_to_direntry(&self, dirid: usize, offset_target: u64) -> u64 {
        debug_assert!(self.inodedata.contains_key(&dirid), "FS directory data for dirid={} should be generated", dirid);
        let data: &[u8] = &*self.inodedata[&dirid].data;
        debug_assert!(data.len() > 0, "FS directory should have at least an entry");

        if offset_target >= data.len() as u64
        {
            return data.len() as u64;
        }

        let mut offset : u64 = 0;
        loop
        {
            let (_qid, offset_after) = unmarshall_qid(data, offset);
            let (next_offset, _offset_after_2) = unmarshall_u64(data, offset_after);
            if next_offset > offset_target
            {
                break;
            }
            offset = next_offset;
        }

        return offset;
    }

    /**
     * @param {number} idx
     * @return {boolean}
     */
    pub fn is_directory(&self, idx : usize) -> bool {
        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            let foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_immutable(inode).is_directory(foreign_id);
        }
        return (inode.mode & S_IFMT) == S_IFDIR;
    }
    /**
     * @param {number} idx
     * @return {boolean}
     */
    pub fn is_empty(&self, idx : usize) -> bool {
        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            return self.follow_fs_immutable(&inode).is_empty(inode.foreign_id.unwrap());
        }
        for name in inode.direntries.keys()
        {
            if name != "." && name != ".."
            {
                return false;
            }
        }
        return true;
    }
    
    /**
     * @param {number} idx
     * @return {!Array<string>} List of children names
     */
    pub fn get_children(&self, idx : usize) -> Vec<String> {
        debug_assert!(self.is_directory(idx), "Filesystem: cannot get children of non-directory inode");
        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            return self.follow_fs_immutable(inode).get_children(inode.foreign_id.unwrap());
        }
        let mut children : Vec<String> = Vec::new();
        for name in inode.direntries.keys()
        {
            if name != "." && name != ".."
            {
                children.push(name.to_string());
            }
        }
        return children;
    }
    /**
     * @param {number} idx
     * @return {number} Local idx of parent
     */
    pub fn get_parent(&mut self, idx : usize) -> Option<usize> {
        debug_assert!(self.is_directory(idx), "Filesystem: cannot get parent of non-directory inode");

        let inode = &self.inodes[idx];

        if FS::should_be_linked(inode)
        {
            return inode.direntries.get(&"..".to_owned()).map(|&x| x);
        }
        else
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            let foreign_dirid = self.follow_fs_by_id_mut(mount_id)
                .get_parent(foreign_id);
            debug_assert!(foreign_dirid.is_some(), "Filesystem: should not have invalid parent ids");
            return Some(self.get_forwarder(mount_id, foreign_dirid.unwrap()));
        }
    }

    
    // -----------------------------------------------------

    // only support for security.capabilities
    // should return a  "struct vfs_cap_data" defined in
    // linux/capability for format
    // check also:
    //   sys/capability.h
    //   http://lxr.free-electrons.com/source/security/commoncap.c#L376
    //   http://man7.org/linux/man-pages/man7/capabilities.7.html
    //   http://man7.org/linux/man-pages/man8/getcap.8.html
    //   http://man7.org/linux/man-pages/man3/libcap.3.html
    pub fn prepare_caps(&mut self, id : usize) -> usize {
        let inode: &mut INode = self.get_inode_mutable(id);
        if let Some(caps) = inode.caps.as_ref() {
            return caps.data.len();
        }
        inode.caps = Some(UInt8Array::new(20));
        let caps: &mut [u8] = &mut *inode.caps.as_mut().unwrap().data;
        // format is little endian
        // note: getxattr returns -EINVAL if using revision 1 format.
        // note: getxattr presents revision 3 as revision 2 when revision 3 is not needed.
        // magic_etc (revision=0x02: 20 bytes)
        caps[0]  = 0x00;
        caps[1]  = 0x00;
        caps[2]  = 0x00;
        caps[3]  = 0x02;

        // lower
        // permitted (first 32 capabilities)
        caps[4]  = 0xFF;
        caps[5]  = 0xFF;
        caps[6]  = 0xFF;
        caps[7]  = 0xFF;
        // inheritable (first 32 capabilities)
        caps[8]  = 0xFF;
        caps[9]  = 0xFF;
        caps[10] = 0xFF;
        caps[11] = 0xFF;

        // higher
        // permitted (last 6 capabilities)
        caps[12] = 0x3F;
        caps[13] = 0x00;
        caps[14] = 0x00;
        caps[15] = 0x00;
        // inheritable (last 6 capabilities)
        caps[16] = 0x3F;
        caps[17] = 0x00;
        caps[18] = 0x00;
        caps[19] = 0x00;

        return caps.len();
    }

    // -----------------------------------------------------

    
     /**
     * @private
     * @param {number} idx Local idx of inode.
     * @param {number} mount_id Mount number of the destination fs.
     * @param {number} foreign_id Foreign idx of destination inode.
     */
    pub fn set_forwarder(&mut self, idx : usize, mount_id : usize, foreign_id : usize) {
        
        let inode = &mut self.inodes[idx];

        debug_assert!(inode.nlinks == 0,
            "Filesystem: attempted to convert an inode into forwarder before unlinking the inode");

        if FS::is_forwarder(inode) 
        {
            self.mounts[inode.mount_id.unwrap()].backtrack.remove(&inode.foreign_id.unwrap());
        }

        inode.status = STATUS_FORWARDING;
        inode.mount_id = Some(mount_id);
        inode.foreign_id = Some(foreign_id);

        self.mounts[mount_id].backtrack.insert(foreign_id, idx);
    }
    
    /**
     * @private
     * @param {number} mount_id Mount number of the destination fs.
     * @param {number} foreign_id Foreign idx of destination inode.
     * @return {number} Local idx of newly created forwarder.
     */
    pub fn create_forwarder(&mut self, mount_id : usize, foreign_id : usize) -> usize {
        let mut inode = self.create_inode();

        let idx = self.inodes.len();
        inode.fid = idx;
        self.inodes.push(inode);

        self.set_forwarder(idx, mount_id, foreign_id);
        return idx;
    }
    /**
     * @private
     * @param {Inode} inode
     * @return {boolean}
     */
    pub fn is_forwarder(inode : &INode) -> bool {
        return inode.status == STATUS_FORWARDING;
    }
    /**
     * Whether the inode it points to is a root of some filesystem.
     * @private
     * @param {number} idx
     * @return {boolean}
     */
    pub fn is_a_root(&self, idx : usize) -> bool {
        return self.get_inode(idx).fid == 0;
    }

    /**
     * Ensures forwarder exists, and returns such forwarder, for the described foreign inode.
     * @private
     * @param {number} mount_id
     * @param {number} foreign_id
     * @return {number} Local idx of a forwarder to described inode.
     */
    pub fn get_forwarder(&mut self, mount_id : usize, foreign_id : usize) -> usize {
        let mount = &self.mounts.get(mount_id);

        //debug_assert!(foreign_id >= 0, "Filesystem get_forwarder: invalid foreign_id: {}", foreign_id);
        debug_assert!(mount.is_some(), "Filesystem get_forwarder: invalid mount number: {}", mount_id);

        let result = mount.unwrap().backtrack.get(&foreign_id);

        if result.is_none()
        {
            // Create if not already exists.
            return self.create_forwarder(mount_id, foreign_id);
        }
        else
        {
            return *result.unwrap();
        }
    }
    

    /**
     * @private
     * @param {Inode} inode
     */
    pub fn delete_forwarder(&mut self, inode : &mut INode) {
        debug_assert!(FS::is_forwarder(inode), "Filesystem delete_forwarder: expected forwarder");

        inode.status = STATUS_INVALID;
        self.mounts[inode.mount_id.unwrap()].backtrack.remove(&inode.foreign_id.unwrap());
    }

    

    pub fn follow_fs(&mut self, inode: &INode) -> &FS {
        debug_assert!(FS::is_forwarder(inode),
            "Filesystem follow_fs: inode should be a forwarding inode");
        debug_assert!(inode.mount_id.is_some(), "Filesystem follow_fs: inode<id={}> should point to valid mounted FS", inode.fid);
    
        return self.follow_fs_by_id(inode.mount_id.unwrap());
    }

    /**
     * @private
     * @param {Inode} inode
     * @return {FS}
     */
    pub fn follow_fs_by_id(&self, mount_id : usize) -> &FS {
        let mount = self.mounts.get(mount_id);

        debug_assert!(mount.is_some(), "Filesystem follow_fs: mount_id:{} should point to valid mounted FS", mount_id);
        
        return &mount.unwrap().fs;
    }

    pub fn follow_fs_by_id_mut(&mut self, mount_id : usize) -> &mut FS {
        let mount = self.mounts.get_mut(mount_id);

        debug_assert!(mount.is_some(), "Filesystem follow_fs: mount_id:{} should point to valid mounted FS", mount_id);
        
        return &mut mount.unwrap().fs;
    }

    pub fn follow_fs_immutable(&self, inode : &INode) -> &FS {
        debug_assert!(FS::is_forwarder(inode),
            "Filesystem follow_fs: inode should be a forwarding inode");

        let msg : &str = "Filesystem follow_fs: inode<id={inode.fid}> should point to valid mounted FS";

        debug_assert!(inode.mount_id.is_some(), "{}", msg);
        
        let mount = self.mounts.get(inode.mount_id.unwrap());

        debug_assert!(mount.is_some(), "{}", msg);
        
        return &mount.unwrap().fs;
    }

    

    /**
     * Mount another filesystem to given path.
     * @param {string} path
     * @param {FS} fs
     * @return {number} inode id of mount point if successful, or -errno if mounting failed.
     */
    pub fn mount(&mut self, path : &String, fs : FS) -> (Option<usize>, i32)  {
        debug_assert!(fs.qidcounter.last_qidnumber == self.qidcounter.last_qidnumber,
            "Cannot mount filesystem whose qid numbers aren't synchronised with current filesystem.");

        let path_infos = self.search_path(path);

        if path_infos.parentid.is_none()
        {
            print_debug(format!("Mount failed: parent for path not found: {}", path));
            return (None, ENOENT);
        }
        if path_infos.id.is_none()
        {
            print_debug(format!("Mount failed: file already exists at path: {}", path));
            return (None, EEXIST);
        }
        if path_infos.forward_path.is_some()
        {
            let parent = &self.inodes[path_infos.parentid.unwrap()];
            let parent_mount_id = parent.mount_id.unwrap();
            let forward_path = path_infos.forward_path;
            let (mount_id, mount_err) = self.follow_fs_by_id_mut(parent_mount_id)
                .mount(&forward_path.unwrap(), fs);
            if mount_id.is_none() {
                return (None, mount_err);
            } 
            else {
                return (Some(self.get_forwarder(parent_mount_id, mount_id.unwrap())),
                SUCCESS);
            }
        }

        let mount_id = self.mounts.len();
        self.mounts.push(FSMountInfo::new(fs));

        let idx = self.create_forwarder(mount_id, 0);
        self.link_under_dir(path_infos.parentid.unwrap(), idx, &path_infos.name);

        return (Some(idx), SUCCESS);
    }
    /*
    /**
     * @param {number} type
     * @param {number} start
     * @param {number} length
     * @param {number} proc_id
     * @param {string} client_id
     * @return {!FSLockRegion}
     */
    public DescribeLock(type : number, start : number, length : number, proc_id : number, client_id: string) : FSLockRegion {
        debug_assert!(type === P9_LOCK_TYPE_RDLCK ||
            type === P9_LOCK_TYPE_WRLCK ||
            type === P9_LOCK_TYPE_UNLCK,
            "Filesystem: Invalid lock type: " + type);
        debug_assert!(start >= 0, "Filesystem: Invalid negative lock starting offset: " + start);
        debug_assert!(length > 0, "Filesystem: Invalid non-positive lock length: " + length);

        const lock = new FSLockRegion();
        lock.r#type = type;
        lock.start = start;
        lock.length = length;
        lock.proc_id = proc_id;
        lock.client_id = client_id;

        return lock;
    }

    /**
     * @param {number} id
     * @param {FSLockRegion} request
     * @return {FSLockRegion} The first conflicting lock found, or null if requested lock is possible.
     */
    public GetLock(id : number, request : FSLockRegion) : FSLockRegion | null {
        const inode = self.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return self.follow_fs(inode).GetLock(foreign_id, request);
        }

        for(const region of inode.locks)
        {
            if(request.conflicts_with(region))
            {
                return region.clone();
            }
        }
        return null;
    }

    /**
     * @param {number} id
     * @param {FSLockRegion} request
     * @param {number} flags
     * @return {number} One of P9_LOCK_SUCCESS / P9_LOCK_BLOCKED / P9_LOCK_ERROR / P9_LOCK_GRACE.
     */
    public Lock(id : number, request : FSLockRegion, flags : number) : number {
        const inode = self.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return self.follow_fs(inode).Lock(foreign_id, request, flags);
        }

        request = request.clone();

        // (1) Check whether lock is possible before any modification.
        if(request.r#type !== P9_LOCK_TYPE_UNLCK && self.GetLock(id, request))
        {
            return P9_LOCK_BLOCKED;
        }

        // (2) Subtract requested region from locks of the same owner.
        for(let i = 0; i < inode.locks.length; i++)
        {
            const region = inode.locks[i];

            debug_assert!(region.length > 0,
                "Filesystem: Found non-positive lock region length: " + region.length);
            debug_assert!(region.r#type === P9_LOCK_TYPE_RDLCK || region.r#type === P9_LOCK_TYPE_WRLCK,
                "Filesystem: Found invalid lock type: " + region.r#type);
            debug_assert!(!inode.locks[i-1] || inode.locks[i-1].start <= region.start,
                "Filesystem: Locks should be sorted by starting offset");

            // Skip to requested region.
            if(region.start + region.length <= request.start) continue;

            // Check whether we've skipped past the requested region.
            if(request.start + request.length <= region.start) break;

            // Skip over locks of different owners.
            if(region.proc_id !== request.proc_id || region.client_id !== request.client_id)
            {
                debug_assert!(!region.conflicts_with(request),
                    "Filesytem: Found conflicting lock region, despite already checked for conflicts");
                continue;
            }

            // Pretend region would be split into parts 1 and 2.
            const start1 = region.start;
            const start2 = request.start + request.length;
            const length1 = request.start - start1;
            const length2 = region.start + region.length - start2;

            if(length1 > 0 && length2 > 0 && region.r#type === request.r#type)
            {
                // Requested region is already locked with the required type.
                // Return early - no need to modify anything.
                return P9_LOCK_SUCCESS;
            }

            if(length1 > 0)
            {
                // Shrink from right / first half of the split.
                region.length = length1;
            }

            if(length1 <= 0 && length2 > 0)
            {
                // Shrink from left.
                region.start = start2;
                region.length = length2;
            }
            else if(length2 > 0)
            {
                // Add second half of the split.

                // Fast-forward to correct location.
                while(i < inode.locks.length && inode.locks[i].start < start2) i++;

                inode.locks.splice(i, 0,
                    self.DescribeLock(region.r#type, start2, length2, region.proc_id, region.client_id));
            }
            else if(length1 <= 0)
            {
                // Requested region completely covers this region. Delete.
                inode.locks.splice(i, 1);
                i--;
            }
        }

        // (3) Insert requested lock region as a whole.
        // No point in adding the requested lock region as fragmented bits in the above loop
        // and having to merge them all back into one.
        if(request.r#type !== P9_LOCK_TYPE_UNLCK)
        {
            let new_region = request;
            let has_merged = false;
            let i = 0;

            // Fast-forward to requested position, and try merging with previous region.
            for(; i < inode.locks.length; i++)
            {
                if(new_region.may_merge_after(inode.locks[i]))
                {
                    inode.locks[i].length += request.length;
                    new_region = inode.locks[i];
                    has_merged = true;
                }
                if(request.start <= inode.locks[i].start) break;
            }

            if(!has_merged)
            {
                inode.locks.splice(i, 0, new_region);
                i++;
            }

            // Try merging with the subsequent alike region.
            for(; i < inode.locks.length; i++)
            {
                if(!inode.locks[i].is_alike(new_region)) continue;

                if(inode.locks[i].may_merge_after(new_region))
                {
                    new_region.length += inode.locks[i].length;
                    inode.locks.splice(i, 1);
                }

                // No more mergable regions after self.
                break;
            }
        }

        return P9_LOCK_SUCCESS;
    }

    public read_dir(path : string) : Nullable<Array<string>> {
        const p = self.SearchPath(path);

        if(p.id === -1)
        {
            return undefined;
        }

        const dir = self.GetInode(p.id);

        return Array.from(dir.direntries.keys()).filter(path => path !== "." && path !== "..");
    }
    */

    pub fn read_file(&mut self, file : &String) -> Option<&[u8]> {
        let p = self.search_path(file);

        if p.id.is_none()
        {
            return None;
        }

        let inode = self.get_inode(p.id.unwrap());

        return self.read(p.id.unwrap(), 0, inode.size);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct QID {
    // r# is needed because type is a keyword
    r#type: u8,
    version: u32,
    path: u64
}

/** @constructor */
#[derive(Serialize, Deserialize, Debug)]
struct INode {
    direntries : HashMap<String, usize>,
    status : i32,
    size : usize,
    uid : i32,
    gid : i32,
    fid : usize,
    ctime : u64,
    atime : u64,
    mtime : u64,
    major : i32,
    minor : i32,
    symlink : String,
    mode : i32,
    qid : QID,
    caps : Option<UInt8Array>,
    nlinks : i32,
    sha256sum : String,

    locks : Vec<FSLockRegion>,

    mount_id : Option<usize>,
    foreign_id : Option<usize>
}

impl INode {
    pub fn new(quidnumber : u64) -> INode {
        INode {
            direntries : HashMap::new(), // maps filename to inode id
            status : 0,
            size : 0x0,
            uid : 0x0,
            gid : 0x0,
            fid : 0,
            ctime : 0,
            atime : 0,
            mtime : 0,
            major : 0x0,
            minor : 0x0,
            symlink : "".to_owned(),
            mode : 0x01ED,
            qid : QID {
                r#type: 0,
                version: 0,
                path: quidnumber
            },

            caps : None,
            nlinks : 0,
            sha256sum : "".to_owned(),
        
            locks : Vec::new(), // lock regions applied to the file, sorted by starting offset.
        
            // For forwarders:
            mount_id : None, // which fs in self.mounts does this inode forward to?
            foreign_id : None // which foreign inode id does it represent?
        
            //this.qid_type = 0;
            //this.qid_version = 0;
            //this.qid_path = qidnumber;
        }
    }
}


/*
    get_state() : Array<string | i32 | Array<FSLockRegion> | null | Array<i32>> {
        const state = [];
        state[0] = self.mode;
    
        if((this.mode & S_IFMT) === S_IFDIR)
        {
            state[1] = [...this.direntries];
        }
        else if((this.mode & S_IFMT) === S_IFREG)
        {
            state[1] = self.sha256sum;
        }
        else if((this.mode & S_IFMT) === S_IFLNK)
        {
            state[1] = self.symlink;
        }
        else if((this.mode & S_IFMT) === S_IFSOCK)
        {
            state[1] = [this.minor, self.major];
        }
        else
        {
            state[1] = null;
        }
    
        state[2] = self.locks;
        state[3] = self.status;
        state[4] = self.size;
        state[5] = self.uid;
        state[6] = self.gid;
        state[7] = self.fid;
        state[8] = self.ctime;
        state[9] = self.atime;
        state[10] = self.mtime;
        state[11] = self.qid.version;
        state[12] = self.qid.path;
        state[13] = self.nlinks;
    
        //state[23] = self.mount_id;
        //state[24] = self.foreign_id;
        //state[25] = self.caps; // currently not writable
        return state;
    }

    set_state(state : Array<string | i32 | Array<FSLockRegion> | Array<i32> | Array<Array<string | i32>> | null>) : void {
        self.mode = state[0] as i32;

        if((this.mode & S_IFMT) === S_IFDIR)
        {
            self.direntries = new Map();
            for(const [name, entry] of (state[1] as Array<Array<string | i32>>))
            {
                self.direntries.set(name, entry);
            }
        }
        else if((this.mode & S_IFMT) === S_IFREG)
        {
            self.sha256sum = state[1] as string;
        }
        else if((this.mode & S_IFMT) === S_IFLNK)
        {
            self.symlink = state[1] as string;
        }
        else if((this.mode & S_IFMT) === S_IFSOCK)
        {
            [this.minor, self.major] = state[1] as Array<i32>;
        }
        else
        {
            // Nothing
        }
    
        self.locks = [];
        for(const lock_state of (state[2] as Array<FSLockRegion>))
        {
            const lock = new FSLockRegion();
            lock.set_state(lock_state);
            self.locks.push(lock);
        }
        self.status = state[3] as i32;
        self.size = state[4] as i32;
        self.uid = state[5] as i32;
        self.gid = state[6] as i32;
        self.fid = state[7] as i32;
        self.ctime = state[8] as i32;
        self.atime = state[9] as i32;
        self.mtime = state[10] as i32;
        self.qid.r#type = (this.mode & S_IFMT) >> 8;
        self.qid.version = state[11] as i32;
        self.qid.path = state[12] as i32;
        self.nlinks = state[13] as i32;
    
        //this.mount_id = state[23];
        //this.foreign_id = state[24];
        //this.caps = state[20];    
    }
}
*/


#[derive(Serialize, Deserialize, Debug)]
struct FSMountInfo
{
    fs : FS,
    // Maps foreign inode id back to local inode id.
    backtrack : HashMap<usize, usize>
}


impl FSMountInfo {
    pub fn new(filesystem: FS) -> FSMountInfo {
        FSMountInfo {
            fs : filesystem,
            backtrack : HashMap::new()
        }
    }
}

/*

    get_state() : Array<any> {
        const state = [];

        state[0] = self.fs;

        state[1] = [...this.backtrack];
    
        return state;    
    }

    set_state(state : Array<any>) : void {
        self.fs = state[0] as FS;
        // todo: figure out this
        self.backtrack = new Map(state[1] as object);
    }
}
*/

/**
 * @constructor
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
struct FSLockRegion {
    r#type : i32,
    start : i32,
    length : Option<i32>,
    proc_id : i32,
    client_id : String
}

impl FSLockRegion {
    pub fn new() -> FSLockRegion {
        FSLockRegion {
            r#type : P9_LOCK_TYPE_UNLCK,
            start : 0,
            //length : Infinity,
            length: None,
            proc_id : -1,
            client_id : "".to_owned()
        }
    }
}
/*
    public get_state() : Array<any> {
        const state = [];

        state[0] = self.r#type;
        state[1] = self.start;
        // Infinity is not JSON.stringify-able
        state[2] = self.length === Infinity ? 0 : self.length;
        state[3] = self.proc_id;
        state[4] = self.client_id;

        return state;
    }

    public set_state(state : Array<any>) : void {
        self.r#type = state[0] as i32;
        self.start = state[1] as i32;
        self.length = (state[2] === 0 ? Infinity : state[2]) as i32;
        self.proc_id = state[3] as i32;
        self.client_id = state[4] as string;
    }

    public clone() : FSLockRegion {
        const new_region = new FSLockRegion();
        new_region.set_state(this.get_state());
        return new_region;
    }

    public conflicts_with(region: FSLockRegion) : boolean {
        if(this.proc_id === region.proc_id && self.client_id === region.client_id) return false;
        if(this.r#type === P9_LOCK_TYPE_UNLCK || region.r#type === P9_LOCK_TYPE_UNLCK) return false;
        if(this.r#type !== P9_LOCK_TYPE_WRLCK && region.r#type !== P9_LOCK_TYPE_WRLCK) return false;
        if(this.start + self.length <= region.start) return false;
        if(region.start + region.length <= self.start) return false;
        return true;    
    }

    public is_alike(region : FSLockRegion) : boolean {
        return region.proc_id === self.proc_id &&
            region.client_id === self.client_id &&
            region.r#type === self.r#type;
    }

    public may_merge_after(region : FSLockRegion) : boolean {
        return self.is_alike(region) && region.start + region.length === self.start;
    }
}
*/