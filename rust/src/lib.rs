#![allow(unused_variables)]
#![allow(dead_code)]

use serde::{Serialize, Deserialize};

type Number = i64;

use std::collections::HashMap;
use std::vec::Vec;
use async_trait::async_trait;
use std::sync::Arc;


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
    id: i32,
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
        let result = FS {
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

            //RegisterMessage("LoadFilesystem", this.LoadFilesystem.bind(this) );
            //RegisterMessage("MergeFile", this.MergeFile.bind(this) );
            //RegisterMessage("tar",
            //    function(data) {
            //        SendToMaster("tar", this.tar.Pack(data));
            //    }.bind(this)
            //);
            //RegisterMessage("sync",
            //    function(data) {
            //        SendToMaster("sync", this.tar.Pack(data));
            //    }.bind(this)
            //);
        };

        //result.CreateDirectory("", -1);

        result
    }
    /*

    public get_state() : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | Uint8Array | FSMountInfo> {
        var state : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | Uint8Array | FSMountInfo> = [];
    
        state[0] = this.inodes;
        state[1] = this.qidcounter.last_qidnumber;
        state[2] = [] as Array<Array<number | Uint8Array>>;
        for(const [id, data] of Object.entries(this.inodedata))
        {
            if((this.inodes[id].mode & S_IFDIR) === 0)
            {
                state[2].push([id, data] as Array<number | Uint8Array>);
            }
        }
        state[3] = this.total_size;
        state[4] = this.used_size;
        state = state.concat(this.mounts);
    
        return state;
    }

    public set_state(state : Array<number | Array<INode> | Array<Array<number | Uint8Array>> | FSMountInfo>) {
        this.inodes = state[0].map(state => { const inode = new Inode(0); inode.set_state(state); return inode; });
        this.qidcounter.last_qidnumber = state[1] as number;
        this.inodedata = {};
        for(let [key, value] of state[2])
        {
            if(value.buffer.byteLength !== value.byteLength)
            {
                // make a copy if we didn't get one
                value = value.slice();
            }
    
            this.inodedata[key] = value;
        }
        this.total_size = state[3] as number;
        this.used_size = state[4] as number;
        this.mounts = state.slice(5) as Array<FSMountInfo>;
    }


    // -----------------------------------------------------

    public AddEvent(id : number, OnEvent : Function) : void {
        var inode = this.inodes[id];
        if(inode.status === STATUS_OK || inode.status === STATUS_ON_STORAGE) {
            OnEvent();
        }
        else if(this.is_forwarder(inode))
        {
            this.follow_fs(inode).AddEvent(inode.foreign_id, OnEvent);
        }
        else
        {
            this.events.push({id: id, OnEvent: OnEvent});
        }
    }

    public HandleEvent(id : number) : void {
        const inode = this.inodes[id];
        if(this.is_forwarder(inode))
        {
            this.follow_fs(inode).HandleEvent(inode.foreign_id);
        }
        //message.Debug("number of events: " + this.events.length);
        var newevents = [];
        for(var i=0; i<this.events.length; i++) {
            if(this.events[i].id === id) {
                this.events[i].OnEvent();
            } else {
                newevents.push(this.events[i]);
            }
        }
        this.events = newevents;
    }

    public load_from_json(fs : Map<string, any>) : void {
        dbg_assert(fs, "Invalid fs passed to load_from_json");

        if(fs.get("version") !== JSONFS_VERSION)
        {
            throw "The filesystem JSON format has changed. " +
                "Please update your fs2json (https://github.com/copy/fs2json) and recreate the filesystem JSON.";
        }

        var fsroot = fs.get("fsroot");
        this.used_size = fs.get("size");

        for(var i = 0; i < fsroot.length; i++) {
            this.LoadRecursive(fsroot[i], 0);
        }

        //if(DEBUG)
        //{
        //    this.Check();
        //}
    }

    public LoadRecursive(data : Map<string, any>, parentid : number) : void {
        var inode = this.CreateInode();

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
            this.PushInode(inode, parentid, name);
            this.LoadDir(this.inodes.length - 1, data.get(JSONFS_IDX_TARGET));
        }
        else if(ifmt === S_IFREG)
        {
            inode.status = STATUS_ON_STORAGE;
            inode.sha256sum = data.get(JSONFS_IDX_SHA256);
            dbg_assert(inode.sha256sum);
            this.PushInode(inode, parentid, name);
        }
        else if(ifmt === S_IFLNK)
        {
            inode.symlink = data.get(JSONFS_IDX_TARGET);
            this.PushInode(inode, parentid, name);
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
            this.LoadRecursive(children[i], parentid);
        }
    }


    // -----------------------------------------------------

    /**
     * @private
     * @param {Inode} inode
     * @return {boolean}
     */
    public should_be_linked(inode : INode) : boolean {
        // Note: Non-root forwarder inode could still have a non-forwarder parent, so don't use
        // parent inode to check.
        return !this.is_forwarder(inode) || inode.foreign_id === 0;
    }

    /**
     * @private
     * @param {number} parentid
     * @param {number} idx
     * @param {string} name
     */
    public link_under_dir(parentid : number, idx : number, name : string) : void {
        const inode = this.inodes[idx];
        const parent_inode = this.inodes[parentid];

        dbg_assert(!this.is_forwarder(parent_inode),
            "Filesystem: Shouldn't link under fowarder parents");
        dbg_assert(this.IsDirectory(parentid),
            "Filesystem: Can't link under non-directories");
        dbg_assert(this.should_be_linked(inode),
            "Filesystem: Can't link across filesystems apart from their root");
        dbg_assert(inode.nlinks >= 0,
            "Filesystem: Found negative nlinks value of " + inode.nlinks);
        dbg_assert(!parent_inode.direntries.has(name),
            "Filesystem: Name '" + name + "' is already taken");

        parent_inode.direntries.set(name, idx);
        inode.nlinks++;

        if(this.IsDirectory(idx))
        {
            dbg_assert(!inode.direntries.has(".."),
                "Filesystem: Cannot link a directory twice");

            if(!inode.direntries.has(".")) inode.nlinks++;
            inode.direntries.set(".", idx);

            inode.direntries.set("..", parentid);
            parent_inode.nlinks++;
        }
    }

    /**
     * @private
     * @param {number} parentid
     * @param {string} name
     */
    public unlink_from_dir(parentid : number, name : string) : void {
        const idx = this.Search(parentid, name);
        const inode = this.inodes[idx];
        const parent_inode = this.inodes[parentid];

        dbg_assert(!this.is_forwarder(parent_inode), "Filesystem: Can't unlink from forwarders");
        dbg_assert(this.IsDirectory(parentid), "Filesystem: Can't unlink from non-directories");

        const exists = parent_inode.direntries.delete(name);
        if(!exists)
        {
            dbg_assert(false, "Filesystem: Can't unlink non-existent file: " + name);
            return;
        }

        inode.nlinks--;

        if(this.IsDirectory(idx))
        {
            dbg_assert(inode.direntries.get("..") === parentid,
                "Filesystem: Found directory with bad parent id");

            inode.direntries.delete("..");
            parent_inode.nlinks--;
        }

        dbg_assert(inode.nlinks >= 0,
            "Filesystem: Found negative nlinks value of " + inode.nlinks);
    }

    public PushInode(inode : number, parentid : number, name : string) : void {
        if(parentid !== -1) {
            this.inodes.push(inode);
            inode.fid = this.inodes.length - 1;
            this.link_under_dir(parentid, inode.fid, name);
            return;
        } else {
            if(this.inodes.length === 0) { // if root directory
                this.inodes.push(inode);
                inode.direntries.set(".", 0);
                inode.direntries.set("..", 0);
                inode.nlinks = 2;
                return;
            }
        }

        message.Debug("Error in Filesystem: Pushed inode with name = "+ name + " has no parent");
        message.Abort();
    }

    /**
         * Clones given inode to new idx, effectively diverting the inode to new idx value.
         * Hence, original idx value is now free to use without losing the original information.
         * @private
         * @param {number} parentid Parent of target to divert.
         * @param {string} filename Name of target to divert.
         * @return {number} New idx of diversion.
         */
    public divert(parentid : number, filename : string) : number {
        const old_idx = this.Search(parentid, filename);
        const old_inode = this.inodes[old_idx];
        const new_inode = new Inode(-1);

        dbg_assert(old_inode, "Filesystem divert: name (" + filename + ") not found");
        dbg_assert(this.IsDirectory(old_idx) || old_inode.nlinks <= 1,
            "Filesystem: can't divert hardlinked file '" + filename + "' with nlinks=" +
            old_inode.nlinks);

        // Shallow copy is alright.
        Object.assign(new_inode, old_inode);

        const idx = this.inodes.length;
        this.inodes.push(new_inode);
        new_inode.fid = idx;

        // Relink references
        if(this.is_forwarder(old_inode))
        {
            this.mounts[old_inode.mount_id].backtrack.set(old_inode.foreign_id, idx);
        }
        if(this.should_be_linked(old_inode))
        {
            this.unlink_from_dir(parentid, filename);
            this.link_under_dir(parentid, idx, filename);
        }

        // Update children
        if(this.IsDirectory(old_idx) && !this.is_forwarder(old_inode))
        {
            for(const [name, child_id] of new_inode.direntries)
            {
                if(name === "." || name === "..") continue;
                if(this.IsDirectory(child_id))
                {
                    this.inodes[child_id].direntries.set("..", idx);
                }
            }
        }

        // Relocate local data if any.
        this.inodedata[idx] = this.inodedata[old_idx];
        delete this.inodedata[old_idx];

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

    public CreateInode() : INode {
        //console.log("CreateInode", Error().stack);
        const now = Math.round(Date.now() / 1000);
        const inode = new INode(++this.qidcounter.last_qidnumber);
        inode.atime = inode.ctime = inode.mtime = now;
        return inode;
    }


    // Note: parentid = -1 for initial root directory.
    public CreateDirectory(name: string, parentid: number) : number {
        const parent_inode = this.inodes[parentid];
        if(parentid >= 0 && this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = this.follow_fs(parent_inode).CreateDirectory(name, foreign_parentid);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = this.CreateInode();
        x.mode = 0x01FF | S_IFDIR;
        if(parentid >= 0) {
            x.uid = this.inodes[parentid].uid;
            x.gid = this.inodes[parentid].gid;
            x.mode = (this.inodes[parentid].mode & 0x1FF) | S_IFDIR;
        }
        x.qid.type = S_IFDIR >> 8;
        this.PushInode(x, parentid, name);
        this.NotifyListeners(this.inodes.length-1, "newdir");
        return this.inodes.length-1;
    }

    public CreateFile(filename : string, parentid : number) : number {
        const parent_inode = this.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = this.follow_fs(parent_inode).CreateFile(filename, foreign_parentid);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = this.CreateInode();
        x.uid = this.inodes[parentid].uid;
        x.gid = this.inodes[parentid].gid;
        x.qid.type = S_IFREG >> 8;
        x.mode = (this.inodes[parentid].mode & 0x1B6) | S_IFREG;
        this.PushInode(x, parentid, filename);
        this.NotifyListeners(this.inodes.length-1, "newfile");
        return this.inodes.length-1;
    }


    public CreateNode(filename: string, parentid: number, major: number, minor: number) : number {
        const parent_inode = this.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id =
                this.follow_fs(parent_inode).CreateNode(filename, foreign_parentid, major, minor);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = this.CreateInode();
        x.major = major;
        x.minor = minor;
        x.uid = this.inodes[parentid].uid;
        x.gid = this.inodes[parentid].gid;
        x.qid.type = S_IFSOCK >> 8;
        x.mode = (this.inodes[parentid].mode & 0x1B6);
        this.PushInode(x, parentid, filename);
        return this.inodes.length-1;
    }

    public CreateSymlink(filename : string, parentid: number, symlink : string) : number {
        const parent_inode = this.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id =
                this.follow_fs(parent_inode).CreateSymlink(filename, foreign_parentid, symlink);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var x = this.CreateInode();
        x.uid = this.inodes[parentid].uid;
        x.gid = this.inodes[parentid].gid;
        x.qid.type = S_IFLNK >> 8;
        x.symlink = symlink;
        x.mode = S_IFLNK;
        this.PushInode(x, parentid, filename);
        return this.inodes.length-1;
    }

    public CreateTextFile(filename : string, parentid : number, str : string) : number {
        const parent_inode = this.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = await
                this.follow_fs(parent_inode).CreateTextFile(filename, foreign_parentid, str);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var id = this.CreateFile(filename, parentid);
        var x = this.inodes[id];
        var data = new Uint8Array(str.length);
        x.size = str.length;
        for(var j = 0; j < str.length; j++) {
            data[j] = str.charCodeAt(j);
        }
        await this.set_data(id, data);
        return id;
    }

    /**
     * @param {Uint8Array} buffer
     */
    public async CreateBinaryFile(filename : string, parentid : number, buffer : Uint8Array) : number {
        const parent_inode = this.inodes[parentid];
        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = await
                this.follow_fs(parent_inode).CreateBinaryFile(filename, foreign_parentid, buffer);
            return this.create_forwarder(parent_inode.mount_id, foreign_id);
        }
        var id = this.CreateFile(filename, parentid);
        var x = this.inodes[id];
        var data = new Uint8Array(buffer.length);
        data.set(buffer);
        await this.set_data(id, data);
        x.size = buffer.length;
        return id;
    }


    public OpenInode(id : number, mode : number) : boolean {
        var inode = this.inodes[id];
        if(this.is_forwarder(inode))
        {
            return this.follow_fs(inode).OpenInode(inode.foreign_id, mode);
        }
        if((inode.mode&S_IFMT) === S_IFDIR) {
            this.FillDirectory(id);
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
        //message.Debug("open:" + this.GetFullPath(id) +  " type: " + inode.mode + " status:" + inode.status);
        return true;
    }

    public async CloseInode(id : number) : Promise {
        //message.Debug("close: " + this.GetFullPath(id));
        var inode = this.inodes[id];
        if(this.is_forwarder(inode))
        {
            return await this.follow_fs(inode).CloseInode(inode.foreign_id);
        }
        if(inode.status === STATUS_ON_STORAGE)
        {
            this.storage.uncache(inode.sha256sum);
        }
        if(inode.status === STATUS_UNLINKED) {
            //message.Debug("Filesystem: Delete unlinked file");
            inode.status = STATUS_INVALID;
            await this.DeleteData(id);
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
        var oldid = this.Search(olddirid, oldname);
        if(oldid === -1)
        {
            return -ENOENT;
        }

        // For event notification near end of method.
        var oldpath = this.GetFullPath(olddirid) + "/" + oldname;

        var newid = this.Search(newdirid, newname);
        if(newid !== -1) {
            const ret = this.Unlink(newdirid, newname);
            if(ret < 0) return ret;
        }

        var idx = oldid; // idx contains the id which we want to rename
        var inode = this.inodes[idx];
        const olddir = this.inodes[olddirid];
        const newdir = this.inodes[newdirid];

        if(!this.is_forwarder(olddir) && !this.is_forwarder(newdir))
        {
            // Move inode within current filesystem.

            this.unlink_from_dir(olddirid, oldname);
            this.link_under_dir(newdirid, idx, newname);

            inode.qid.version++;
        }
        else if(this.is_forwarder(olddir) && olddir.mount_id === newdir.mount_id)
        {
            // Move inode within the same child filesystem.

            const ret = await
                this.follow_fs(olddir).Rename(olddir.foreign_id, oldname, newdir.foreign_id, newname);

            if(ret < 0) return ret;
        }
        else if(this.is_a_root(idx))
        {
            // The actual inode is a root of some descendant filesystem.
            // Moving mountpoint across fs not supported - needs to update all corresponding forwarders.
            dbg_log("XXX: Attempted to move mountpoint (" + oldname + ") - skipped", LOG_9P);
            return -EPERM;
        }
        else if(!this.IsDirectory(idx) && this.GetInode(idx).nlinks > 1)
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
            const diverted_old_idx = this.divert(olddirid, oldname);
            const old_real_inode = this.GetInode(idx);

            const data = await this.Read(diverted_old_idx, 0, old_real_inode.size);

            if(this.is_forwarder(newdir))
            {
                // Create new inode.
                const foreign_fs = this.follow_fs(newdir);
                const foreign_id = this.IsDirectory(diverted_old_idx) ?
                    foreign_fs.CreateDirectory(newname, newdir.foreign_id) :
                    foreign_fs.CreateFile(newname, newdir.foreign_id);

                const new_real_inode = foreign_fs.GetInode(foreign_id);
                this.copy_inode(old_real_inode, new_real_inode);

                // Point to this new location.
                this.set_forwarder(idx, newdir.mount_id, foreign_id);
            }
            else
            {
                // Replace current forwarder with real inode.
                this.delete_forwarder(inode);
                this.copy_inode(old_real_inode, inode);

                // Link into new location in this filesystem.
                this.link_under_dir(newdirid, idx, newname);
            }

            // Rewrite data to newly created destination.
            await this.ChangeSize(idx, old_real_inode.size);
            if(data && data.length)
            {
                await this.Write(idx, 0, data.length, data);
            }

            // Move children to newly created destination.
            if(this.IsDirectory(idx))
            {
                for(const child_filename of this.GetChildren(diverted_old_idx))
                {
                    const ret = await this.Rename(diverted_old_idx, child_filename, idx, child_filename);
                    if(ret < 0) return ret;
                }
            }

            // Perform destructive changes only after migration succeeded.
            await this.DeleteData(diverted_old_idx);
            const ret = this.Unlink(olddirid, oldname);
            if(ret < 0) return ret;
        }

        this.NotifyListeners(idx, "rename", {oldpath: oldpath});

        return 0;
    }

    public async Write(id : number, offset : number, count : number, buffer : Uint8Array) : Promise<void>{
        this.NotifyListeners(id, "write");
        var inode = this.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            await this.follow_fs(inode).Write(foreign_id, offset, count, buffer);
            return;
        }

        var data = await this.get_buffer(id);

        if(!data || data.length < (offset+count)) {
            await this.ChangeSize(id, Math.floor(((offset+count)*3)/2));
            inode.size = offset + count;
            data = await this.get_buffer(id);
        } else
        if(inode.size < (offset+count)) {
            inode.size = offset + count;
        }
        if(buffer)
        {
            data.set(buffer.subarray(0, count), offset);
        }
        await this.set_data(id, data);
    }

    public async Read(inodeid : number, offset : number, count : number) : Promise<Uint8Array> {
        const inode = this.inodes[inodeid];
        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return await this.follow_fs(inode).Read(foreign_id, offset, count);
        }

        return await this.get_data(inodeid, offset, count);
    }

    public Search(parentid : number, name : string) : number {
        const parent_inode = this.inodes[parentid];

        if(this.is_forwarder(parent_inode))
        {
            const foreign_parentid = parent_inode.foreign_id;
            const foreign_id = this.follow_fs(parent_inode).Search(foreign_parentid, name);
            if(foreign_id === -1) return -1;
            return this.get_forwarder(parent_inode.mount_id, foreign_id);
        }

        const childid = parent_inode.direntries.get(name);
        return childid === undefined ? -1 : childid;
    }

    public CountUsedInodes() : number {
        let count = this.inodes.length;
        for(const { fs, backtrack } of this.mounts)
        {
            count += fs.CountUsedInodes();

            // Forwarder inodes don't count.
            count -=  backtrack.size;
        }
        return count;
    }

    public CountFreeInodes() : number {
        let count = 1024 * 1024;
        for(const { fs } of this.mounts)
        {
            count += fs.CountFreeInodes();
        }
        return count;
    }

    public GetTotalSize() : number {
        let size = this.used_size;
        for(const { fs } of this.mounts)
        {
            size += fs.GetTotalSize();
        }
        return size;
        //var size = 0;
        //for(var i=0; i<this.inodes.length; i++) {
        //    var d = this.inodes[i].data;
        //    size += d ? d.length : 0;
        //}
        //return size;
    }

    public GetSpace() : number {
        let size = this.total_size;
        for(const { fs } of this.mounts)
        {
            size += fs.GetSpace();
        }
        return this.total_size;
    }

    /**
     * XXX: Not ideal.
     * @param {number} idx
     * @return {string}
     */
    public GetDirectoryName(idx : number) : string {
        const parent_inode = this.inodes[this.GetParent(idx)];

        if(this.is_forwarder(parent_inode))
        {
            return this.follow_fs(parent_inode).GetDirectoryName(this.inodes[idx].foreign_id);
        }

        // Root directory.
        if(!parent_inode) return "";

        for(const [name, childid] of parent_inode.direntries)
        {
            if(childid === idx) return name;
        }

        dbg_assert(false, "Filesystem: Found directory inode whose parent doesn't link to it");
        return "";
    }

    public GetFullPath(idx : number) : string {
        dbg_assert(this.IsDirectory(idx), "Filesystem: Cannot get full path of non-directory inode");

        var path = "";

        while(idx !== 0) {
            path = "/" + this.GetDirectoryName(idx) + path;
            idx = this.GetParent(idx);
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

        const parent_inode = this.inodes[parentid];
        const inode = this.inodes[targetid];

        if(this.is_forwarder(parent_inode))
        {
            if(!this.is_forwarder(inode) || inode.mount_id !== parent_inode.mount_id)
            {
                dbg_log("XXX: Attempted to hardlink a file into a child filesystem - skipped", LOG_9P);
                return -EPERM;
            }
            return this.follow_fs(parent_inode).Link(parent_inode.foreign_id, inode.foreign_id, name);
        }

        if(this.is_forwarder(inode))
        {
            dbg_log("XXX: Attempted to hardlink file across filesystems - skipped", LOG_9P);
            return -EPERM;
        }

        this.link_under_dir(parentid, targetid, name);
        return 0;
    }

    public Unlink(parentid : number, name : string) : number {
        if(name === "." || name === "..")
        {
            // Also guarantees that root cannot be deleted.
            return -EPERM;
        }
        const idx = this.Search(parentid, name);
        const inode = this.inodes[idx];
        const parent_inode = this.inodes[parentid];
        //message.Debug("Unlink " + inode.name);

        // forward if necessary
        if(this.is_forwarder(parent_inode))
        {
            dbg_assert(this.is_forwarder(inode), "Children of forwarders should be forwarders");

            const foreign_parentid = parent_inode.foreign_id;
            return this.follow_fs(parent_inode).Unlink(foreign_parentid, name);

            // Keep the forwarder dangling - file is still accessible.
        }

        if(this.IsDirectory(idx) && !this.IsEmpty(idx))
        {
            return -ENOTEMPTY;
        }

        this.unlink_from_dir(parentid, name);

        if(inode.nlinks === 0)
        {
            // don't delete the content. The file is still accessible
            inode.status = STATUS_UNLINKED;
            this.NotifyListeners(idx, "delete");
        }
        return 0;
    }

    public async DeleteData(idx : number) : Promise<void> {
        const inode = this.inodes[idx];
        if(this.is_forwarder(inode))
        {
            await this.follow_fs(inode).DeleteData(inode.foreign_id);
            return;
        }
        inode.size = 0;
        delete this.inodedata[idx];
    }

    /**
     * @private
     * @param {number} idx
     * @return {!Promise<Uint8Array>} The buffer that contains the file contents, which may be larger
     *      than the data itself. To ensure that any modifications done to this buffer is reflected
     *      to the file, call set_data with the modified buffer.
     */
    public async get_buffer(idx : number) : Promise<Uint8Array> {
        const inode = this.inodes[idx];
        dbg_assert(inode, `Filesystem get_buffer: idx ${idx} does not point to an inode`);

        if(this.inodedata[idx])
        {
            return this.inodedata[idx];
        }
        else if(inode.status === STATUS_ON_STORAGE)
        {
            dbg_assert(inode.sha256sum, "Filesystem get_data: found inode on server without sha256sum");
            return await this.storage.read(inode.sha256sum, 0, inode.size);
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
        const inode = this.inodes[idx];
        dbg_assert(inode, `Filesystem get_data: idx ${idx} does not point to an inode`);

        if(this.inodedata[idx])
        {
            return this.inodedata[idx].subarray(offset, offset + count);
        }
        else if(inode.status === STATUS_ON_STORAGE)
        {
            dbg_assert(inode.sha256sum, "Filesystem get_data: found inode on server without sha256sum");
            return await this.storage.read(inode.sha256sum, offset, count);
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
        this.inodedata[idx] = buffer;
        if(this.inodes[idx].status === STATUS_ON_STORAGE)
        {
            this.inodes[idx].status = STATUS_OK;
            this.storage.uncache(this.inodes[idx].sha256sum);
        }
    }

    /**
     * @param {number} idx
     * @return {!Inode}
     */
    public GetInode(idx : number) : INode {
        dbg_assert(!isNaN(idx), "Filesystem GetInode: NaN idx");
        dbg_assert(idx >= 0 && idx < this.inodes.length, "Filesystem GetInode: out of range idx:" + idx);

        const inode = this.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return this.follow_fs(inode).GetInode(inode.foreign_id);
        }

        return inode;
    }

    public async ChangeSize(idx : number, newsize : number) : Promise<void> {
        var inode = this.GetInode(idx);
        var temp = await this.get_data(idx, 0, inode.size);
        //message.Debug("change size to: " + newsize);
        if(newsize === inode.size) return;
        var data = new Uint8Array(newsize);
        inode.size = newsize;
        if(temp)
        {
            var size = Math.min(temp.length, inode.size);
            data.set(temp.subarray(0, size), 0);
        }
        await this.set_data(idx, data);
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
            id = this.Search(parentid, walk[i]);
            if(!forward_path && this.is_forwarder(this.inodes[parentid]))
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
            const foreign_fs = this.follow_fs(this.inodes[dirid]);
            const foreign_dirid = this.inodes[dirid].foreign_id;
            const mount_id = this.inodes[dirid].mount_id;

            const foreign_start = list.length;
            foreign_fs.GetRecursiveList(foreign_dirid, list);
            for(let i = foreign_start; i < list.length; i++)
            {
                list[i].parentid = this.get_forwarder(mount_id, list[i].parentid);
            }
            return;
        }
        for(const [name, id] of this.inodes[dirid].direntries)
        {
            if(name !== "." && name !== "..")
            {
                list.push({ parentid: dirid, name });
                if(this.IsDirectory(id))
                {
                    this.GetRecursiveList(id, list);
                }
            }
        }
    }

    public RecursiveDelete(path : string) : void {
        var toDelete = [];
        var ids = this.SearchPath(path);
        if(ids.id === -1) return;

        this.GetRecursiveList(ids.id, toDelete);

        for(var i=toDelete.length-1; i>=0; i--)
        {
            const ret = this.Unlink(toDelete[i].parentid, toDelete[i].name);
            dbg_assert(ret === 0, "Filesystem RecursiveDelete failed at parent=" + toDelete[i].parentid +
                ", name='" + toDelete[i].name + "' with error code: " + (-ret));
        }
    }

    public DeleteNode(path : string) : void {
        var ids = this.SearchPath(path);
        if(ids.id === -1) return;

        if((this.inodes[ids.id].mode&S_IFMT) === S_IFREG){
            const ret = this.Unlink(ids.parentid, ids.name);
            dbg_assert(ret === 0, "Filesystem DeleteNode failed with error code: " + (-ret));
        }
        else if((this.inodes[ids.id].mode&S_IFMT) === S_IFDIR){
            this.RecursiveDelete(path);
            const ret = this.Unlink(ids.parentid, ids.name);
            dbg_assert(ret === 0, "Filesystem DeleteNode failed with error code: " + (-ret));
        }
    }

    /** @param {*=} info */
    public NotifyListeners(id: number, action: Function, info: object) : void {
        //if(info==undefined)
        //    info = {};

        //var path = this.GetFullPath(id);
        //if (this.watchFiles[path] === true && action=='write') {
        //  message.Send("WatchFileEvent", path);
        //}
        //for (var directory of this.watchDirectories) {
        //    if (this.watchDirectories.hasOwnProperty(directory)) {
        //        var indexOf = path.indexOf(directory)
        //        if(indexOf === 0 || indexOf === 1)
        //            message.Send("WatchDirectoryEvent", {path: path, event: action, info: info});
        //    }
        //}
    }


    public Check() : void {
        for(var i=1; i<this.inodes.length; i++)
        {
            if(this.inodes[i].status === STATUS_INVALID) continue;

            var inode = this.GetInode(i);
            if(inode.nlinks < 0) {
                message.Debug("Error in filesystem: negative nlinks=" + inode.nlinks + " at id =" + i);
            }

            if(this.IsDirectory(i))
            {
                const inode = this.GetInode(i);
                if(this.IsDirectory(i) && this.GetParent(i) < 0) {
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
        const inode = this.inodes[dirid];
        if(this.is_forwarder(inode))
        {
            // XXX: The ".." of a mountpoint should point back to an inode in this fs.
            // Otherwise, ".." gets the wrong qid and mode.
            this.follow_fs(inode).FillDirectory(inode.foreign_id);
            return;
        }

        let size = 0;
        for(const name of inode.direntries.keys())
        {
            size += 13 + 8 + 1 + 2 + texten.encode(name).length;
        }
        const data = this.inodedata[dirid] = new Uint8Array(size);
        inode.size = size;

        let offset = 0x0;
        for([name, id] of inode.direntries)
        {
            const child = this.GetInode(id);
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
        const data = this.inodedata[dirid];
        dbg_assert(data, `FS directory data for dirid=${dirid} should be generated`);
        dbg_assert(data.length, "FS directory should have at least an entry");

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

    /**
     * @param {number} idx
     * @return {boolean}
     */
    public IsDirectory(idx : number) : boolean {
        const inode = this.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return this.follow_fs(inode).IsDirectory(inode.foreign_id);
        }
        return (inode.mode & S_IFMT) === S_IFDIR;
    }

    /**
     * @param {number} idx
     * @return {boolean}
     */
    public IsEmpty(idx : number) : boolean {
        const inode = this.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return this.follow_fs(inode).IsDirectory(inode.foreign_id);
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
        dbg_assert(this.IsDirectory(idx), "Filesystem: cannot get children of non-directory inode");
        const inode = this.inodes[idx];
        if(this.is_forwarder(inode))
        {
            return this.follow_fs(inode).GetChildren(inode.foreign_id);
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
        dbg_assert(this.IsDirectory(idx), "Filesystem: cannot get parent of non-directory inode");

        const inode = this.inodes[idx];

        if(this.should_be_linked(inode))
        {
            return inode.direntries.get("..");
        }
        else
        {
            const foreign_dirid = this.follow_fs(inode).GetParent(inode.foreign_id);
            dbg_assert(foreign_dirid !== -1, "Filesystem: should not have invalid parent ids");
            return this.get_forwarder(inode.mount_id, foreign_dirid);
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
        var inode = this.GetInode(id);
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


        /**
     * @private
     * @param {number} idx Local idx of inode.
     * @param {number} mount_id Mount number of the destination fs.
     * @param {number} foreign_id Foreign idx of destination inode.
     */
    public set_forwarder(idx : number, mount_id : number, foreign_id : number) : void {
        const inode = this.inodes[idx];

        dbg_assert(inode.nlinks === 0,
            "Filesystem: attempted to convert an inode into forwarder before unlinking the inode");

        if(this.is_forwarder(inode))
        {
            this.mounts[inode.mount_id].backtrack.delete(inode.foreign_id);
        }

        inode.status = STATUS_FORWARDING;
        inode.mount_id = mount_id;
        inode.foreign_id = foreign_id;

        this.mounts[mount_id].backtrack.set(foreign_id, idx);
    }

    /**
     * @private
     * @param {number} mount_id Mount number of the destination fs.
     * @param {number} foreign_id Foreign idx of destination inode.
     * @return {number} Local idx of newly created forwarder.
     */
    public create_forwarder(mount_id : number, foreign_id : number) : number {
        const inode = this.CreateInode();

        const idx = this.inodes.length;
        this.inodes.push(inode);
        inode.fid = idx;

        this.set_forwarder(idx, mount_id, foreign_id);
        return idx;
    }

    /**
     * @private
     * @param {Inode} inode
     * @return {boolean}
     */
    public is_forwarder(inode : INode) : boolean {
        return inode.status === STATUS_FORWARDING;
    }

    /**
     * Whether the inode it points to is a root of some filesystem.
     * @private
     * @param {number} idx
     * @return {boolean}
     */
    public is_a_root(idx : number) : boolean {
        return this.GetInode(idx).fid === 0;
    }

    /**
     * Ensures forwarder exists, and returns such forwarder, for the described foreign inode.
     * @private
     * @param {number} mount_id
     * @param {number} foreign_id
     * @return {number} Local idx of a forwarder to described inode.
     */
    public get_forwarder(mount_id : number, foreign_id : number) : number {
        const mount = this.mounts[mount_id];

        dbg_assert(foreign_id >= 0, "Filesystem get_forwarder: invalid foreign_id: " + foreign_id);
        dbg_assert(mount, "Filesystem get_forwarder: invalid mount number: " + mount_id);

        const result = mount.backtrack.get(foreign_id);

        if(result === undefined)
        {
            // Create if not already exists.
            return this.create_forwarder(mount_id, foreign_id);
        }

        return result;
    }

    /**
     * @private
     * @param {Inode} inode
     */
    public delete_forwarder(inode : INode) : void {
        dbg_assert(this.is_forwarder(inode), "Filesystem delete_forwarder: expected forwarder");

        inode.status = STATUS_INVALID;
        this.mounts[inode.mount_id].backtrack.delete(inode.foreign_id);
    }

    /**
     * @private
     * @param {Inode} inode
     * @return {FS}
     */
    public follow_fs(inode : INode) : FS {
        const mount = this.mounts[inode.mount_id];

        dbg_assert(this.is_forwarder(inode),
            "Filesystem follow_fs: inode should be a forwarding inode");
        dbg_assert(mount, "Filesystem follow_fs: inode<id=" + inode.fid +
            "> should point to valid mounted FS");

        return mount.fs;
    }

    /**
     * Mount another filesystem to given path.
     * @param {string} path
     * @param {FS} fs
     * @return {number} inode id of mount point if successful, or -errno if mounting failed.
     */
    public Mount(path : string, fs : FS) : number {
        dbg_assert(fs.qidcounter === this.qidcounter,
            "Cannot mount filesystem whose qid numbers aren't synchronised with current filesystem.");

        const path_infos = this.SearchPath(path);

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
            const parent = this.inodes[path_infos.parentid];
            const ret = this.follow_fs(parent).Mount(path_infos.forward_path, fs);
            if(ret < 0) return ret;
            return this.get_forwarder(parent.mount_id, ret);
        }

        const mount_id = this.mounts.length;
        this.mounts.push(new FSMountInfo(fs));

        const idx = this.create_forwarder(mount_id, 0);
        this.link_under_dir(path_infos.parentid, idx, path_infos.name);

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
        dbg_assert(type === P9_LOCK_TYPE_RDLCK ||
            type === P9_LOCK_TYPE_WRLCK ||
            type === P9_LOCK_TYPE_UNLCK,
            "Filesystem: Invalid lock type: " + type);
        dbg_assert(start >= 0, "Filesystem: Invalid negative lock starting offset: " + start);
        dbg_assert(length > 0, "Filesystem: Invalid non-positive lock length: " + length);

        const lock = new FSLockRegion();
        lock.type = type;
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
        const inode = this.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return this.follow_fs(inode).GetLock(foreign_id, request);
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
        const inode = this.inodes[id];

        if(this.is_forwarder(inode))
        {
            const foreign_id = inode.foreign_id;
            return this.follow_fs(inode).Lock(foreign_id, request, flags);
        }

        request = request.clone();

        // (1) Check whether lock is possible before any modification.
        if(request.type !== P9_LOCK_TYPE_UNLCK && this.GetLock(id, request))
        {
            return P9_LOCK_BLOCKED;
        }

        // (2) Subtract requested region from locks of the same owner.
        for(let i = 0; i < inode.locks.length; i++)
        {
            const region = inode.locks[i];

            dbg_assert(region.length > 0,
                "Filesystem: Found non-positive lock region length: " + region.length);
            dbg_assert(region.type === P9_LOCK_TYPE_RDLCK || region.type === P9_LOCK_TYPE_WRLCK,
                "Filesystem: Found invalid lock type: " + region.type);
            dbg_assert(!inode.locks[i-1] || inode.locks[i-1].start <= region.start,
                "Filesystem: Locks should be sorted by starting offset");

            // Skip to requested region.
            if(region.start + region.length <= request.start) continue;

            // Check whether we've skipped past the requested region.
            if(request.start + request.length <= region.start) break;

            // Skip over locks of different owners.
            if(region.proc_id !== request.proc_id || region.client_id !== request.client_id)
            {
                dbg_assert(!region.conflicts_with(request),
                    "Filesytem: Found conflicting lock region, despite already checked for conflicts");
                continue;
            }

            // Pretend region would be split into parts 1 and 2.
            const start1 = region.start;
            const start2 = request.start + request.length;
            const length1 = request.start - start1;
            const length2 = region.start + region.length - start2;

            if(length1 > 0 && length2 > 0 && region.type === request.type)
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
                    this.DescribeLock(region.type, start2, length2, region.proc_id, region.client_id));
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
        if(request.type !== P9_LOCK_TYPE_UNLCK)
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

                // No more mergable regions after this.
                break;
            }
        }

        return P9_LOCK_SUCCESS;
    }

    public read_dir(path : string) : Nullable<Array<string>> {
        const p = this.SearchPath(path);

        if(p.id === -1)
        {
            return undefined;
        }

        const dir = this.GetInode(p.id);

        return Array.from(dir.direntries.keys()).filter(path => path !== "." && path !== "..");
    }

    public read_file(file : string) : Uint8Array {
        const p = this.SearchPath(file);

        if(p.id === -1)
        {
            return Promise.resolve(null);
        }

        const inode = this.GetInode(p.id);

        return this.Read(p.id, 0, inode.size);
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
    direntries : HashMap<String, i32>,
    status : i32,
    size : i32,
    uid : i32,
    gid : i32,
    fid : i32,
    ctime : i32,
    atime : i32,
    mtime : i32,
    major : i32,
    minor : i32,
    symlink : String,
    mode : i32,
    qid : QID,
    caps : Option<Uint8Array>,
    nlinks : i32,
    sha256sum : String,

    locks : Vec<FSLockRegion>,

    mount_id : i32,
    foreign_id : i32
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
            mount_id : -1, // which fs in this.mounts does this inode forward to?
            foreign_id : -1 // which foreign inode id does it represent?
        
            //this.qid_type = 0;
            //this.qid_version = 0;
            //this.qid_path = qidnumber;
        }
    }
}


/*
    get_state() : Array<string | i32 | Array<FSLockRegion> | null | Array<i32>> {
        const state = [];
        state[0] = this.mode;
    
        if((this.mode & S_IFMT) === S_IFDIR)
        {
            state[1] = [...this.direntries];
        }
        else if((this.mode & S_IFMT) === S_IFREG)
        {
            state[1] = this.sha256sum;
        }
        else if((this.mode & S_IFMT) === S_IFLNK)
        {
            state[1] = this.symlink;
        }
        else if((this.mode & S_IFMT) === S_IFSOCK)
        {
            state[1] = [this.minor, this.major];
        }
        else
        {
            state[1] = null;
        }
    
        state[2] = this.locks;
        state[3] = this.status;
        state[4] = this.size;
        state[5] = this.uid;
        state[6] = this.gid;
        state[7] = this.fid;
        state[8] = this.ctime;
        state[9] = this.atime;
        state[10] = this.mtime;
        state[11] = this.qid.version;
        state[12] = this.qid.path;
        state[13] = this.nlinks;
    
        //state[23] = this.mount_id;
        //state[24] = this.foreign_id;
        //state[25] = this.caps; // currently not writable
        return state;
    }

    set_state(state : Array<string | i32 | Array<FSLockRegion> | Array<i32> | Array<Array<string | i32>> | null>) : void {
        this.mode = state[0] as i32;

        if((this.mode & S_IFMT) === S_IFDIR)
        {
            this.direntries = new Map();
            for(const [name, entry] of (state[1] as Array<Array<string | i32>>))
            {
                this.direntries.set(name, entry);
            }
        }
        else if((this.mode & S_IFMT) === S_IFREG)
        {
            this.sha256sum = state[1] as string;
        }
        else if((this.mode & S_IFMT) === S_IFLNK)
        {
            this.symlink = state[1] as string;
        }
        else if((this.mode & S_IFMT) === S_IFSOCK)
        {
            [this.minor, this.major] = state[1] as Array<i32>;
        }
        else
        {
            // Nothing
        }
    
        this.locks = [];
        for(const lock_state of (state[2] as Array<FSLockRegion>))
        {
            const lock = new FSLockRegion();
            lock.set_state(lock_state);
            this.locks.push(lock);
        }
        this.status = state[3] as i32;
        this.size = state[4] as i32;
        this.uid = state[5] as i32;
        this.gid = state[6] as i32;
        this.fid = state[7] as i32;
        this.ctime = state[8] as i32;
        this.atime = state[9] as i32;
        this.mtime = state[10] as i32;
        this.qid.type = (this.mode & S_IFMT) >> 8;
        this.qid.version = state[11] as i32;
        this.qid.path = state[12] as i32;
        this.nlinks = state[13] as i32;
    
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
    backtrack : HashMap<i32, i32>
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

        state[0] = this.fs;

        state[1] = [...this.backtrack];
    
        return state;    
    }

    set_state(state : Array<any>) : void {
        this.fs = state[0] as FS;
        // todo: figure out this
        this.backtrack = new Map(state[1] as object);
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

        state[0] = this.type;
        state[1] = this.start;
        // Infinity is not JSON.stringify-able
        state[2] = this.length === Infinity ? 0 : this.length;
        state[3] = this.proc_id;
        state[4] = this.client_id;

        return state;
    }

    public set_state(state : Array<any>) : void {
        this.type = state[0] as i32;
        this.start = state[1] as i32;
        this.length = (state[2] === 0 ? Infinity : state[2]) as i32;
        this.proc_id = state[3] as i32;
        this.client_id = state[4] as string;
    }

    public clone() : FSLockRegion {
        const new_region = new FSLockRegion();
        new_region.set_state(this.get_state());
        return new_region;
    }

    public conflicts_with(region: FSLockRegion) : boolean {
        if(this.proc_id === region.proc_id && this.client_id === region.client_id) return false;
        if(this.type === P9_LOCK_TYPE_UNLCK || region.type === P9_LOCK_TYPE_UNLCK) return false;
        if(this.type !== P9_LOCK_TYPE_WRLCK && region.type !== P9_LOCK_TYPE_WRLCK) return false;
        if(this.start + this.length <= region.start) return false;
        if(region.start + region.length <= this.start) return false;
        return true;    
    }

    public is_alike(region : FSLockRegion) : boolean {
        return region.proc_id === this.proc_id &&
            region.client_id === this.client_id &&
            region.type === this.type;
    }

    public may_merge_after(region : FSLockRegion) : boolean {
        return this.is_alike(region) && region.start + region.length === this.start;
    }
}
*/