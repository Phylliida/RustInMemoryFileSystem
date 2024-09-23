#![allow(unused_variables)]
#![allow(dead_code)]

use serde::{Serialize, Deserialize};

type Number = i64;

use std::collections::HashMap;
use std::vec::Vec;
use async_trait::async_trait;
use std::sync::Arc;

use std::time::{SystemTime, UNIX_EPOCH};


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
    last_qidnumber: i32
}

type EventFn = fn() -> ();

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    id: usize,
    #[serde(skip)]
    on_event: Option<EventFn>
}

#[derive(Serialize, Deserialize, Debug)]
struct Uint8Array {

}

#[derive(Serialize, Deserialize, Debug)]
struct INodeData {
    data: HashMap<i32, Uint8Array>    
    //[key: number]: Uint8Array;
}

#[async_trait]
trait FileStorageInterface {
    /**
    * Read a portion of a file.
    * @param {string} sha256sum
    * @param {number} offset
    * @param {number} count
    * @return {!Promise<Uint8Array>} null if file does not exist.
    */
    async fn read(&mut self, sha256sum: String, offset: i32, count: i32) -> Uint8Array;
    /**
    * Add a read-only file to the filestorage.
    * @param {string} sha256sum
    * @param {!Uint8Array} data
    * @return {!Promise}
    */
    async fn cache(&mut self, sha256sum: String, data: Uint8Array) -> ();

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


impl INodeData {
    pub fn new() -> INodeData {
        INodeData {
            data: HashMap::new()
        }
    }
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
    inodedata : INodeData,
    total_size : Number,
    used_size : Number,
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

            inodedata : INodeData::new(),

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

        result.create_directory("".to_owned(), None);

        result
    }
    /*

    public get_state() : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | Uint8Array | FSMountInfo> {
        var state : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | Uint8Array | FSMountInfo> = [];
    
        state[0] = self.inodes;
        state[1] = self.qidcounter.last_qidnumber;
        state[2] = [] as Array<Array<number | Uint8Array>>;
        for(const [id, data] of Object.entries(this.inodedata))
        {
            if((this.inodes[id].mode & S_IFDIR) === 0)
            {
                state[2].push([id, data] as Array<number | Uint8Array>);
            }
        }
        state[3] = self.total_size;
        state[4] = self.used_size;
        state = state.concat(this.mounts);
    
        return state;
    }
    

    public set_state(state : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | FSMountInfo>) {
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
        else if FS::is_forwarder(&inode)
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
    pub fn link_under_dir(&mut self, parentid : usize, idx : usize, name : String) {
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
            debug_assert!(!self.inodes[parentid].direntries.contains_key(&name),
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
    pub fn unlink_from_dir(&mut self, parentid : usize, name : String) {
        
        debug_assert!(!FS::is_forwarder(&self.inodes[parentid]), "Filesystem: Can't unlink from forwarders");
        debug_assert!(self.is_directory(parentid), "Filesystem: Can't unlink from non-directories");

        let idx = self.search(parentid, &name).unwrap();
        let is_directory = self.is_directory(idx);
        let parent_inode = &mut self.inodes[parentid];

        
        let exists = parent_inode.direntries.remove(&name);
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
    
    pub fn push_inode(&mut self, mut inode : INode, parentid : Option<usize>, name : String) {
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
    /*
    /**
         * Clones given inode to new idx, effectively diverting the inode to new idx value.
         * Hence, original idx value is now free to use without losing the original information.
         * @private
         * @param {number} parentid Parent of target to divert.
         * @param {string} filename Name of target to divert.
         * @return {number} New idx of diversion.
         */
    public divert(parentid : number, filename : string) : number {
        const old_idx = self.Search(parentid, filename);
        const old_inode = self.inodes[old_idx];
        const new_inode = new Inode(-1);

        debug_assert!(old_inode, "Filesystem divert: name (" + filename + ") not found");
        debug_assert!(this.IsDirectory(old_idx) || old_inode.nlinks <= 1,
            "Filesystem: can't divert hardlinked file '" + filename + "' with nlinks=" +
            old_inode.nlinks);

        // Shallow copy is alright.
        Object.assign(new_inode, old_inode);

        const idx = self.inodes.length;
        self.inodes.push(new_inode);
        new_inode.fid = idx;

        // Relink references
        if(this.is_forwarder(old_inode))
        {
            self.mounts[old_inode.mount_id].backtrack.set(old_inode.foreign_id, idx);
        }
        if(this.should_be_linked(old_inode))
        {
            self.unlink_from_dir(parentid, filename);
            self.link_under_dir(parentid, idx, filename);
        }

        // Update children
        if(this.IsDirectory(old_idx) && !this.is_forwarder(old_inode))
        {
            for(const [name, child_id] of new_inode.direntries)
            {
                if(name === "." || name === "..") continue;
                if(this.IsDirectory(child_id))
                {
                    self.inodes[child_id].direntries.set("..", idx);
                }
            }
        }

        // Relocate local data if any.
        self.inodedata[idx] = self.inodedata[old_idx];
        delete self.inodedata[old_idx];

        // Retire old reference information.
        old_inode.direntries = new Map();
        old_inode.nlinks = 0;

        return idx;
    }



    /**
     * Copy all non-redundant info.
     * References left untouched: local idx value and links
     * @private
     * @param {!Inode} src_inode
     * @param {!Inode} dest_inode
     */
    public copy_inode(src_inode : INode, dest_inode : INode) : void {
        Object.assign(dest_inode, src_inode, {
            fid: dest_inode.fid,
            direntries: dest_inode.direntries,
            nlinks: dest_inode.nlinks,
        });
    }
    */

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
    pub fn create_directory(&mut self, name: String, parentid: Option<usize>) -> usize {
        
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
        x.qid.r#type = S_IFDIR >> 8;
        self.push_inode(x, parentid, name);
        self.notify_listeners(self.inodes.len()-1, "newdir".to_owned());
        return self.inodes.len()-1;
    }
    /*

    public CreateFile(filename : string, parentid : number) : number {
        const parent_inode = self.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = self.follow_fs(parent_inode).CreateFile(filename, foreign_parentid);
            return self.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = self.CreateInode();
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = S_IFREG >> 8;
        x.mode = (this.inodes[parentid].mode & 0x1B6) | S_IFREG;
        self.PushInode(x, parentid, filename);
        self.NotifyListeners(this.inodes.length-1, "newfile");
        return self.inodes.length-1;
    }


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
        var data = new Uint8Array(str.length);
        x.size = str.length;
        for(var j = 0; j < str.length; j++) {
            data[j] = str.charCodeAt(j);
        }
        await self.set_data(id, data);
        return id;
    }

    /**
     * @param {Uint8Array} buffer
     */
    public async CreateBinaryFile(filename : string, parentid : number, buffer : Uint8Array) : number {
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
        var data = new Uint8Array(buffer.length);
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

    /**
     * @return {!Promise<number>} 0 if success, or -errno if failured.
     */
    public async Rename(olddirid: number, oldname: string, newdirid: number, newname: string) : Promise<number> {
        // message.Debug("Rename " + oldname + " to " + newname);
        if((olddirid === newdirid) && (oldname === newname)) {
            return 0;
        }
        var oldid = self.Search(olddirid, oldname);
        if(oldid === -1)
        {
            return -ENOENT;
        }

        // For event notification near end of method.
        var oldpath = self.GetFullPath(olddirid) + "/" + oldname;

        var newid = self.Search(newdirid, newname);
        if(newid !== -1) {
            const ret = self.Unlink(newdirid, newname);
            if(ret < 0) return ret;
        }

        var idx = oldid; // idx contains the id which we want to rename
        var inode = self.inodes[idx];
        const olddir = self.inodes[olddirid];
        const newdir = self.inodes[newdirid];

        if(!this.is_forwarder(olddir) && !this.is_forwarder(newdir))
        {
            // Move inode within current filesystem.

            self.unlink_from_dir(olddirid, oldname);
            self.link_under_dir(newdirid, idx, newname);

            inode.qid.version++;
        }
        else if(this.is_forwarder(olddir) && olddir.mount_id === newdir.mount_id)
        {
            // Move inode within the same child filesystem.

            const ret = await
                self.follow_fs(olddir).Rename(olddir.foreign_id, oldname, newdir.foreign_id, newname);

            if(ret < 0) return ret;
        }
        else if(this.is_a_root(idx))
        {
            // The actual inode is a root of some descendant filesystem.
            // Moving mountpoint across fs not supported - needs to update all corresponding forwarders.
            dbg_log("XXX: Attempted to move mountpoint (" + oldname + ") - skipped", LOG_9P);
            return -EPERM;
        }
        else if(!this.IsDirectory(idx) && self.GetInode(idx).nlinks > 1)
        {
            // Move hardlinked inode vertically in mount tree.
            dbg_log("XXX: Attempted to move hardlinked file (" + oldname + ") " +
                    "across filesystems - skipped", LOG_9P);
            return -EPERM;
        }
        else
        {
            // Jump between filesystems.

            // Can't work with both old and new inode information without first diverting the old
            // information into a new idx value.
            const diverted_old_idx = self.divert(olddirid, oldname);
            const old_real_inode = self.GetInode(idx);

            const data = await self.Read(diverted_old_idx, 0, old_real_inode.size);

            if(this.is_forwarder(newdir))
            {
                // Create new inode.
                const foreign_fs = self.follow_fs(newdir);
                const foreign_id = self.IsDirectory(diverted_old_idx) ?
                    foreign_fs.CreateDirectory(newname, newdir.foreign_id) :
                    foreign_fs.CreateFile(newname, newdir.foreign_id);

                const new_real_inode = foreign_fs.GetInode(foreign_id);
                self.copy_inode(old_real_inode, new_real_inode);

                // Point to this new location.
                self.set_forwarder(idx, newdir.mount_id, foreign_id);
            }
            else
            {
                // Replace current forwarder with real inode.
                self.delete_forwarder(inode);
                self.copy_inode(old_real_inode, inode);

                // Link into new location in this filesystem.
                self.link_under_dir(newdirid, idx, newname);
            }

            // Rewrite data to newly created destination.
            await self.ChangeSize(idx, old_real_inode.size);
            if(data && data.length)
            {
                await self.Write(idx, 0, data.length, data);
            }

            // Move children to newly created destination.
            if(this.IsDirectory(idx))
            {
                for(const child_filename of self.GetChildren(diverted_old_idx))
                {
                    const ret = await self.Rename(diverted_old_idx, child_filename, idx, child_filename);
                    if(ret < 0) return ret;
                }
            }

            // Perform destructive changes only after migration succeeded.
            await self.DeleteData(diverted_old_idx);
            const ret = self.Unlink(olddirid, oldname);
            if(ret < 0) return ret;
        }

        self.NotifyListeners(idx, "rename", {oldpath: oldpath});

        return 0;
    }

    public async Write(id : number, offset : number, count : number, buffer : Uint8Array) : Promise<void>{
        self.NotifyListeners(id, "write");
        var inode = self.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            await self.follow_fs(inode).Write(foreign_id, offset, count, buffer);
            return;
        }

        var data = await self.get_buffer(id);

        if(!data || data.length < (offset+count)) {
            await self.ChangeSize(id, Math.floor(((offset+count)*3)/2));
            inode.size = offset + count;
            data = await self.get_buffer(id);
        } else
        if(inode.size < (offset+count)) {
            inode.size = offset + count;
        }
        if(buffer)
        {
            data.set(buffer.subarray(0, count), offset);
        }
        await self.set_data(id, data);
    }

    public async Read(inodeid : number, offset : number, count : number) : Promise<Uint8Array> {
        const inode = self.inodes[inodeid];
        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return await self.follow_fs(inode).Read(foreign_id, offset, count);
        }

        return await self.get_data(inodeid, offset, count);
    }
    
    */
    pub fn search(&mut self, parentid : usize, name : &String) -> Option<usize> {
        let parent_inode = &self.inodes[parentid];

        if FS::is_forwarder(&parent_inode)
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
    /*

    public CountUsedInodes() : number {
        let count = self.inodes.length;
        for(const { fs, backtrack } of self.mounts)
        {
            count += fs.CountUsedInodes();

            // Forwarder inodes don't count.
            count -=  backtrack.size;
        }
        return count;
    }

    public CountFreeInodes() : number {
        let count = 1024 * 1024;
        for(const { fs } of self.mounts)
        {
            count += fs.CountFreeInodes();
        }
        return count;
    }

    public GetTotalSize() : number {
        let size = self.used_size;
        for(const { fs } of self.mounts)
        {
            size += fs.GetTotalSize();
        }
        return size;
        //var size = 0;
        //for(var i=0; i<this.inodes.length; i++) {
        //    var d = self.inodes[i].data;
        //    size += d ? d.length : 0;
        //}
        //return size;
    }

    public GetSpace() : number {
        let size = self.total_size;
        for(const { fs } of self.mounts)
        {
            size += fs.GetSpace();
        }
        return self.total_size;
    }

    /**
     * XXX: Not ideal.
     * @param {number} idx
     * @return {string}
     */
    public GetDirectoryName(idx : number) : string {
        const parent_inode = self.inodes[this.GetParent(idx)];

        if(this.is_forwarder(parent_inode))
        {
            return self.follow_fs(parent_inode).GetDirectoryName(this.inodes[idx].foreign_id);
        }

        // Root directory.
        if(!parent_inode) return "";

        for(const [name, childid] of parent_inode.direntries)
        {
            if(childid === idx) return name;
        }

        debug_assert!(false, "Filesystem: Found directory inode whose parent doesn't link to it");
        return "";
    }

    public GetFullPath(idx : number) : string {
        debug_assert!(this.IsDirectory(idx), "Filesystem: Cannot get full path of non-directory inode");

        var path = "";

        while(idx !== 0) {
            path = "/" + self.GetDirectoryName(idx) + path;
            idx = self.GetParent(idx);
        }
        return path.substring(1);
    }

    /**
     * @param {number} parentid
     * @param {number} targetid
     * @param {string} name
     * @return {number} 0 if success, or -errno if failured.
     */
    public Link(parentid : number, targetid : number, name : string) : number {
        if(this.IsDirectory(targetid))
        {
            return -EPERM;
        }

        const parent_inode = self.inodes[parentid];
        const inode = self.inodes[targetid];

        if(this.is_forwarder(parent_inode))
        {
            if(!this.is_forwarder(inode) || inode.mount_id !== parent_inode.mount_id)
            {
                dbg_log("XXX: Attempted to hardlink a file into a child filesystem - skipped", LOG_9P);
                return -EPERM;
            }
            return self.follow_fs(parent_inode).Link(parent_inode.foreign_id, inode.foreign_id, name);
        }

        if(this.is_forwarder(inode))
        {
            dbg_log("XXX: Attempted to hardlink file across filesystems - skipped", LOG_9P);
            return -EPERM;
        }

        self.link_under_dir(parentid, targetid, name);
        return 0;
    }

    public Unlink(parentid : number, name : string) : number {
        if(name === "." || name === "..")
        {
            // Also guarantees that root cannot be deleted.
            return -EPERM;
        }
        const idx = self.Search(parentid, name);
        const inode = self.inodes[idx];
        const parent_inode = self.inodes[parentid];
        //message.Debug("Unlink " + inode.name);

        // forward if necessary
        if(this.is_forwarder(parent_inode))
        {
            debug_assert!(this.is_forwarder(inode), "Children of forwarders should be forwarders");

            const foreign_parentid = parent_inode.foreign_id;
            return self.follow_fs(parent_inode).Unlink(foreign_parentid, name);

            // Keep the forwarder dangling - file is still accessible.
        }

        if(this.IsDirectory(idx) && !this.IsEmpty(idx))
        {
            return -ENOTEMPTY;
        }

        self.unlink_from_dir(parentid, name);

        if(inode.nlinks === 0)
        {
            // don't delete the content. The file is still accessible
            inode.status = STATUS_UNLINKED;
            self.NotifyListeners(idx, "delete");
        }
        return 0;
    }

    public async DeleteData(idx : number) : Promise<void> {
        const inode = self.inodes[idx];
        if(this.is_forwarder(inode))
        {
            await self.follow_fs(inode).DeleteData(inode.foreign_id);
            return;
        }
        inode.size = 0;
        delete self.inodedata[idx];
    }

    /**
     * @private
     * @param {number} idx
     * @return {!Promise<Uint8Array>} The buffer that contains the file contents, which may be larger
     *      than the data itself. To ensure that any modifications done to this buffer is reflected
     *      to the file, call set_data with the modified buffer.
     */
    public async get_buffer(idx : number) : Promise<Uint8Array> {
        const inode = self.inodes[idx];
        debug_assert!(inode, `Filesystem get_buffer: idx ${idx} does not point to an inode`);

        if(this.inodedata[idx])
        {
            return self.inodedata[idx];
        }
        else if(inode.status === STATUS_ON_STORAGE)
        {
            debug_assert!(inode.sha256sum, "Filesystem get_data: found inode on server without sha256sum");
            return await self.storage.read(inode.sha256sum, 0, inode.size);
        }
        else
        {
            return null;
        }
    }

    /**
     * @private
     * @param {number} idx
     * @param {number} offset
     * @param {number} count
     * @return {!Promise<Uint8Array>}
     */
    public async get_data(idx : number, offset : number, count : number) : Promise<Uint8Array> {
        const inode = self.inodes[idx];
        debug_assert!(inode, `Filesystem get_data: idx ${idx} does not point to an inode`);

        if(this.inodedata[idx])
        {
            return self.inodedata[idx].subarray(offset, offset + count);
        }
        else if(inode.status === STATUS_ON_STORAGE)
        {
            debug_assert!(inode.sha256sum, "Filesystem get_data: found inode on server without sha256sum");
            return await self.storage.read(inode.sha256sum, offset, count);
        }
        else
        {
            return null;
        }
    }

    /**
     * @private
     * @param {number} idx
     * @param {Uint8Array} buffer
     */
    public async set_data(idx : number, buffer : UInt8Array) : Promise<void> {
        // Current scheme: Save all modified buffers into local inodedata.
        self.inodedata[idx] = buffer;
        if(this.inodes[idx].status === STATUS_ON_STORAGE)
        {
            self.inodes[idx].status = STATUS_OK;
            self.storage.uncache(this.inodes[idx].sha256sum);
        }
    }

    /**
     * @param {number} idx
     * @return {!Inode}
     */
    public GetInode(idx : number) : INode {
        debug_assert!(!isNaN(idx), "Filesystem GetInode: NaN idx");
        debug_assert!(idx >= 0 && idx < self.inodes.length, "Filesystem GetInode: out of range idx:" + idx);

        const inode = self.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return self.follow_fs(inode).GetInode(inode.foreign_id);
        }

        return inode;
    }

    public async ChangeSize(idx : number, newsize : number) : Promise<void> {
        var inode = self.GetInode(idx);
        var temp = await self.get_data(idx, 0, inode.size);
        //message.Debug("change size to: " + newsize);
        if(newsize === inode.size) return;
        var data = new Uint8Array(newsize);
        inode.size = newsize;
        if(temp)
        {
            var size = Math.min(temp.length, inode.size);
            data.set(temp.subarray(0, size), 0);
        }
        await self.set_data(idx, data);
    }

    public SearchPath(path : string) : object {
        //path = path.replace(/\/\//g, "/");
        path = path.replace("//", "/");
        var walk = path.split("/");
        if(walk.length > 0 && walk[walk.length - 1].length === 0) walk.pop();
        if(walk.length > 0 && walk[0].length === 0) walk.shift();
        const n = walk.length;

        var parentid = -1;
        var id = 0;
        let forward_path = null;
        for(var i=0; i<n; i++) {
            parentid = id;
            id = self.Search(parentid, walk[i]);
            if(!forward_path && self.is_forwarder(this.inodes[parentid]))
            {
                forward_path = "/" + walk.slice(i).join("/");
            }
            if(id === -1) {
                if(i < n-1) return {id: -1, parentid: -1, name: walk[i], forward_path }; // one name of the path cannot be found
                return {id: -1, parentid: parentid, name: walk[i], forward_path}; // the last element in the path does not exist, but the parent
            }
        }
        return {id: id, parentid: parentid, name: walk[i], forward_path};
    }
    // -----------------------------------------------------

    /**
     * @param {number} dirid
     * @param {Array<{parentid: number, name: string}>} list
     */
    public GetRecursiveList(dirid : number, list : Array<{parentid: number, name: string}>) : void {
        if(this.is_forwarder(this.inodes[dirid]))
        {
            const foreign_fs = self.follow_fs(this.inodes[dirid]);
            const foreign_dirid = self.inodes[dirid].foreign_id;
            const mount_id = self.inodes[dirid].mount_id;

            const foreign_start = list.length;
            foreign_fs.GetRecursiveList(foreign_dirid, list);
            for(let i = foreign_start; i < list.length; i++)
            {
                list[i].parentid = self.get_forwarder(mount_id, list[i].parentid);
            }
            return;
        }
        for(const [name, id] of self.inodes[dirid].direntries)
        {
            if(name !== "." && name !== "..")
            {
                list.push({ parentid: dirid, name });
                if(this.IsDirectory(id))
                {
                    self.GetRecursiveList(id, list);
                }
            }
        }
    }

    public RecursiveDelete(path : string) : void {
        var toDelete = [];
        var ids = self.SearchPath(path);
        if(ids.id === -1) return;

        self.GetRecursiveList(ids.id, toDelete);

        for(var i=toDelete.length-1; i>=0; i--)
        {
            const ret = self.Unlink(toDelete[i].parentid, toDelete[i].name);
            debug_assert!(ret === 0, "Filesystem RecursiveDelete failed at parent=" + toDelete[i].parentid +
                ", name='" + toDelete[i].name + "' with error code: " + (-ret));
        }
    }

    public DeleteNode(path : string) : void {
        var ids = self.SearchPath(path);
        if(ids.id === -1) return;

        if((this.inodes[ids.id].mode&S_IFMT) === S_IFREG){
            const ret = self.Unlink(ids.parentid, ids.name);
            debug_assert!(ret === 0, "Filesystem DeleteNode failed with error code: " + (-ret));
        }
        else if((this.inodes[ids.id].mode&S_IFMT) === S_IFDIR){
            self.RecursiveDelete(path);
            const ret = self.Unlink(ids.parentid, ids.name);
            debug_assert!(ret === 0, "Filesystem DeleteNode failed with error code: " + (-ret));
        }
    }
    */
    pub fn notify_listeners(&self, id: usize, action: String) {

    }

    /** @param {*=} info */
    pub fn notify_listeners_with_data(&self, id: usize, action: String, info: Option<NotifyInfo>) {
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

    /*
    public Check() : void {
        for(var i=1; i<this.inodes.length; i++)
        {
            if(this.inodes[i].status === STATUS_INVALID) continue;

            var inode = self.GetInode(i);
            if(inode.nlinks < 0) {
                message.Debug("Error in filesystem: negative nlinks=" + inode.nlinks + " at id =" + i);
            }

            if(this.IsDirectory(i))
            {
                const inode = self.GetInode(i);
                if(this.IsDirectory(i) && self.GetParent(i) < 0) {
                    message.Debug("Error in filesystem: negative parent id " + i);
                }
                for(const [name, id] of inode.direntries)
                {
                    if(name.length === 0) {
                        message.Debug("Error in filesystem: inode with no name and id " + id);
                    }

                    for(const c of name) {
                        if(c < 32) {
                            message.Debug("Error in filesystem: Unallowed char in filename");
                        }
                    }
                }
            }
        }
    }


    public FillDirectory(dirid: number) : void {
        const inode = self.inodes[dirid];
        if(this.is_forwarder(inode))
        {
            // XXX: The ".." of a mountpoint should point back to an inode in this fs.
            // Otherwise, ".." gets the wrong qid and mode.
            self.follow_fs(inode).FillDirectory(inode.foreign_id);
            return;
        }

        let size = 0;
        for(const name of inode.direntries.keys())
        {
            size += 13 + 8 + 1 + 2 + texten.encode(name).length;
        }
        const data = self.inodedata[dirid] = new Uint8Array(size);
        inode.size = size;

        let offset = 0x0;
        for([name, id] of inode.direntries)
        {
            const child = self.GetInode(id);
            offset += marshall.Marshall(
                ["Q", "d", "b", "s"],
                [child.qid,
                offset+13+8+1+2+texten.encode(name).length,
                child.mode >> 12,
                name],
                data, offset);
        }
    }

    public RoundToDirentry(dirid: number, offset_target: number) : number {
        const data = self.inodedata[dirid];
        debug_assert!(data, `FS directory data for dirid=${dirid} should be generated`);
        debug_assert!(data.length, "FS directory should have at least an entry");

        if(offset_target >= data.length)
        {
            return data.length;
        }

        let offset = 0;
        while(true)
        {
            const next_offset = marshall.Unmarshall(["Q", "d"], data, { offset })[1];
            if(next_offset > offset_target) break;
            offset = next_offset;
        }

        return offset;
    }
    */
    /**
     * @param {number} idx
     * @return {boolean}
     */
    pub fn is_directory(&self, idx : usize) -> bool {
        let inode = &self.inodes[idx];
        if FS::is_forwarder(inode)
        {
            let foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_immutable(&inode).is_directory(foreign_id);
        }
        return (inode.mode & S_IFMT) == S_IFDIR;
    }
    /*
    /**
     * @param {number} idx
     * @return {boolean}
     */
    public IsEmpty(idx : number) : boolean {
        const inode = self.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return self.follow_fs(inode).IsDirectory(inode.foreign_id);
        }
        for(const name of inode.direntries.keys())
        {
            if(name !== "." && name !== "..") return false;
        }
        return true;
    }

    /**
     * @param {number} idx
     * @return {!Array<string>} List of children names
     */
    public GetChildren(idx : number) : Array<string> {
        debug_assert!(this.IsDirectory(idx), "Filesystem: cannot get children of non-directory inode");
        const inode = self.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return self.follow_fs(inode).GetChildren(inode.foreign_id);
        }
        const children = [];
        for(const name of inode.direntries.keys())
        {
            if(name !== "." && name !== "..")
            {
                children.push(name);
            }
        }
        return children;
    }

    /**
     * @param {number} idx
     * @return {number} Local idx of parent
     */
    public GetParent(idx : number) : number {
        debug_assert!(this.IsDirectory(idx), "Filesystem: cannot get parent of non-directory inode");

        const inode = self.inodes[idx];

        if(this.should_be_linked(inode))
        {
            return inode.direntries.get("..");
        }
        else
        {
            const foreign_dirid = self.follow_fs(inode).GetParent(inode.foreign_id);
            debug_assert!(foreign_dirid !== -1, "Filesystem: should not have invalid parent ids");
            return self.get_forwarder(inode.mount_id, foreign_dirid);
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
    public PrepareCAPs(id : number) : number {
        var inode = self.GetInode(id);
        if(inode.caps) return inode.caps.length;
        inode.caps = new Uint8Array(20);
        // format is little endian
        // note: getxattr returns -EINVAL if using revision 1 format.
        // note: getxattr presents revision 3 as revision 2 when revision 3 is not needed.
        // magic_etc (revision=0x02: 20 bytes)
        inode.caps[0]  = 0x00;
        inode.caps[1]  = 0x00;
        inode.caps[2]  = 0x00;
        inode.caps[3]  = 0x02;

        // lower
        // permitted (first 32 capabilities)
        inode.caps[4]  = 0xFF;
        inode.caps[5]  = 0xFF;
        inode.caps[6]  = 0xFF;
        inode.caps[7]  = 0xFF;
        // inheritable (first 32 capabilities)
        inode.caps[8]  = 0xFF;
        inode.caps[9]  = 0xFF;
        inode.caps[10] = 0xFF;
        inode.caps[11] = 0xFF;

        // higher
        // permitted (last 6 capabilities)
        inode.caps[12] = 0x3F;
        inode.caps[13] = 0x00;
        inode.caps[14] = 0x00;
        inode.caps[15] = 0x00;
        // inheritable (last 6 capabilities)
        inode.caps[16] = 0x3F;
        inode.caps[17] = 0x00;
        inode.caps[18] = 0x00;
        inode.caps[19] = 0x00;

        return inode.caps.length;
    }

    // -----------------------------------------------------

    */
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

        if FS::is_forwarder(&inode) 
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
    /*
    /**
     * Whether the inode it points to is a root of some filesystem.
     * @private
     * @param {number} idx
     * @return {boolean}
     */
    public is_a_root(idx : number) : boolean {
        return self.GetInode(idx).fid === 0;
    }

    /**
     * Ensures forwarder exists, and returns such forwarder, for the described foreign inode.
     * @private
     * @param {number} mount_id
     * @param {number} foreign_id
     * @return {number} Local idx of a forwarder to described inode.
     */
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
    /*

    /**
     * @private
     * @param {Inode} inode
     */
    public delete_forwarder(inode : INode) : void {
        debug_assert!(this.is_forwarder(inode), "Filesystem delete_forwarder: expected forwarder");

        inode.status = STATUS_INVALID;
        self.mounts[inode.mount_id].backtrack.delete(inode.foreign_id);
    }

    */

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

        debug_assert!(mount.is_none(), "Filesystem follow_fs: mount_id:{} should point to valid mounted FS", mount_id);
        
        return &mount.unwrap().fs;
    }

    pub fn follow_fs_by_id_mut(&mut self, mount_id : usize) -> &mut FS {
        let mount = self.mounts.get_mut(mount_id);

        debug_assert!(mount.is_none(), "Filesystem follow_fs: mount_id:{} should point to valid mounted FS", mount_id);
        
        return &mut mount.unwrap().fs;
    }

    pub fn follow_fs_immutable(&self, inode : &INode) -> &FS {
        debug_assert!(FS::is_forwarder(inode),
            "Filesystem follow_fs: inode should be a forwarding inode");

        let msg : &str = "Filesystem follow_fs: inode<id={inode.fid}> should point to valid mounted FS";

        debug_assert!(inode.mount_id.is_some(), "{}", msg);
        
        let mount = self.mounts.get(inode.mount_id.unwrap());

        debug_assert!(mount.is_none(), "{}", msg);
        
        return &mount.unwrap().fs;
    }

    /*

    /**
     * Mount another filesystem to given path.
     * @param {string} path
     * @param {FS} fs
     * @return {number} inode id of mount point if successful, or -errno if mounting failed.
     */
    public Mount(path : string, fs : FS) : number {
        debug_assert!(fs.qidcounter === self.qidcounter,
            "Cannot mount filesystem whose qid numbers aren't synchronised with current filesystem.");

        const path_infos = self.SearchPath(path);

        if(path_infos.parentid === -1)
        {
            dbg_log("Mount failed: parent for path not found: " + path, LOG_9P);
            return -ENOENT;
        }
        if(path_infos.id !== -1)
        {
            dbg_log("Mount failed: file already exists at path: " + path, LOG_9P);
            return -EEXIST;
        }
        if(path_infos.forward_path)
        {
            const parent = self.inodes[path_infos.parentid];
            const ret = self.follow_fs(parent).Mount(path_infos.forward_path, fs);
            if(ret < 0) return ret;
            return self.get_forwarder(parent.mount_id, ret);
        }

        const mount_id = self.mounts.length;
        self.mounts.push(new FSMountInfo(fs));

        const idx = self.create_forwarder(mount_id, 0);
        self.link_under_dir(path_infos.parentid, idx, path_infos.name);

        return idx;
    }
    
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

    public read_file(file : string) : Uint8Array {
        const p = self.SearchPath(file);

        if(p.id === -1)
        {
            return Promise.resolve(null);
        }

        const inode = self.GetInode(p.id);

        return self.Read(p.id, 0, inode.size);
    }
    */
}

#[derive(Serialize, Deserialize, Debug)]
struct QID {
    // r# is needed because type is a keyword
    r#type: i32,
    version: i32,
    path: i32
}

/** @constructor */
#[derive(Serialize, Deserialize, Debug)]
struct INode {
    direntries : HashMap<String, usize>,
    status : i32,
    size : i32,
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
    caps : Option<Uint8Array>,
    nlinks : i32,
    sha256sum : String,

    locks : Vec<FSLockRegion>,

    mount_id : Option<usize>,
    foreign_id : Option<usize>
}

impl INode {
    pub fn new(quidnumber : i32) -> INode {
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
#[derive(Serialize, Deserialize, Debug)]
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