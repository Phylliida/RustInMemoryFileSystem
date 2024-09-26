
//use serde::{Serialize, Deserialize};
use crate::marshall::*;
use std::collections::HashMap;
use std::vec::Vec;
use std::str::from_utf8;
use std::cmp::min;
use crate::wasi::clock_time_get;

use crate::v9p::*;

// -------------------------------------------------
// ----------------- FILESYSTEM---------------------
// -------------------------------------------------
// Implementation of a unix filesystem in memory.
// Ported to rust, from v86 js emulator

pub const CLOCK_REALTIME : u32 = 0;

pub const S_IRWXUGO: i32 = 0x1FF;
pub const S_IFMT: i32 = 0xF000;
pub const S_IFSOCK: i32 = 0xC000;
pub const S_IFLNK: i32 = 0xA000;
pub const S_IFREG: i32 = 0x8000;
pub const S_IFBLK: i32 = 0x6000;
pub const S_IFDIR: i32 = 0x4000;
pub const S_IFCHR: i32 = 0x2000;

//var S_IFIFO  0010000
//var S_ISUID  0004000
//var S_ISGID  0002000
//var S_ISVTX  0001000

pub const O_RDONLY: i32 = 0x0000; // open for reading only
pub const O_WRONLY: i32 = 0x0001; // open for writing only
pub const O_RDWR: i32 = 0x0002; // open for reading and writing
pub const O_ACCMODE: i32 = 0x0003; // mask for above modes

pub const STATUS_INVALID: i32 = -0x1;
pub const STATUS_OK: i32 = 0x0;
pub const STATUS_ON_STORAGE: i32 = 0x2;
pub const STATUS_UNLINKED: i32 = 0x4;
pub const STATUS_FORWARDING: i32 = 0x5;


const JSONFS_VERSION: i32 = 3;


const JSONFS_IDX_NAME: i32 = 0;
const JSONFS_IDX_SIZE: i32 = 1;
const JSONFS_IDX_MTIME: i32 = 2;
const JSONFS_IDX_MODE: i32 = 3;
const JSONFS_IDX_UID: i32 = 4;
const JSONFS_IDX_GID: i32 = 5;
const JSONFS_IDX_TARGET: i32 = 6;
const JSONFS_IDX_SHA256: i32 = 6;

//#[derive(Serialize, Deserialize, Debug)]
pub struct QIDCounter {
    pub last_qidnumber: u64
}

type EventFn = fn() -> ();

//#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: usize,
    //#[serde(skip)]
    pub on_event: Option<EventFn>
}

//#[derive(Serialize, Deserialize, Debug)]
pub struct UInt8Array {
    pub data: Box<[u8]>
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



pub struct NotifyInfo {
    pub oldpath: String
}


pub struct SearchResult {
    pub id: Option<usize>,
    pub parentid: Option<usize>,
    pub name: String,
    pub forward_path: Option<String>
}
pub struct RecursiveListResult {
    pub parentid: usize,
    pub name: String
}




//#[derive(Serialize, Deserialize, Debug)]
pub struct FS{
    pub inodes : Vec<INode>,
    pub root_id : usize,
    //#[serde(skip)]
    pub events : Vec<Event>,
    pub qidcounter : QIDCounter,
    pub inodedata : HashMap<usize, UInt8Array>,
    pub total_size : u64,
    pub used_size : u64,
    pub mounts : Vec<FSMountInfo>
}

impl FS {
    pub fn new(qidcounter : Option<QIDCounter>) -> FS {
        let mut result = FS {
            inodes : Vec::new(),
            events : Vec::new(),
            root_id: 0,

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

        result.root_id = result.create_directory("", None);

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
    pub fn link_under_dir(&mut self, parentid : usize, idx : usize, name : &str) {
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
            parent_inode.direntries.insert(name.to_owned().clone(), idx);
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
    pub fn unlink_from_dir(&mut self, parentid : usize, name : &str) {
        
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
    
    pub fn push_inode(&mut self, mut inode : INode, parentid : Option<usize>, name : &str) {
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
    pub fn divert(&mut self, parentid : usize, filename : &str) -> usize {
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

    pub fn seconds_since_epoch() -> u64 {
        let mut nanoseconds_time : u64 = 0;
        let res = unsafe { clock_time_get(CLOCK_REALTIME, 1_000_000_000, (&mut nanoseconds_time) as *mut u64) };
        return nanoseconds_time / 1_000_000_000;
        
        /*
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let millis = since_the_epoch.as_millis();
        // Divide by 1000 and round
        return (millis as f64 / 1000.0).round() as u64;
        */
    }
    

    pub fn create_inode(&mut self) -> INode {
        //console.log("CreateInode", Error().stack);
        self.qidcounter.last_qidnumber += 1;
        let mut inode = INode::new(self.qidcounter.last_qidnumber);
        let now = FS::seconds_since_epoch();
        inode.mtime = now;
        inode.atime = now;
        inode.ctime = now;
        return inode;
    }

    // Note: parentid = -1 for initial root directory.
    pub fn create_directory(&mut self, name: &str, parentid: Option<usize>) -> usize {
        
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
        self.notify_listeners(self.inodes.len()-1, "newdir");
        return self.inodes.len()-1;
    }
    


    pub fn create_file(&mut self, filename : &str, parentid : usize) -> usize {
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
        self.notify_listeners(self.inodes.len()-1, "newfile");
        return self.inodes.len()-1;
    }

    pub fn create_node(&mut self, filename: &str, parentid: usize, major: i32, minor: i32) -> usize {
        let parent_inode = &self.inodes[parentid];
        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let parent_inode_foreign_id = parent_inode.foreign_id.unwrap();
            let foreign_id =
                self.follow_fs_by_id_mut(parent_inode_mount_id)
                    .create_node(filename, parent_inode_foreign_id
                        , major, minor);
            return self.create_forwarder(parent_inode_mount_id, foreign_id);
        }
        let mut x = self.create_inode();
        x.major = major;
        x.minor = minor;
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = (S_IFSOCK >> 8) as u8;
        x.mode = self.inodes[parentid].mode & 0x1B6;
        self.push_inode(x, Some(parentid), filename);
        return self.inodes.len()-1;
    }

    pub fn create_symlink(&mut self, filename : &str, parentid: usize, symlink : &str) -> usize {
        let parent_inode = &self.inodes[parentid];
        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let parent_inode_foreign_id = parent_inode.foreign_id.unwrap();
            let foreign_id =
                self.follow_fs_by_id_mut(parent_inode_mount_id)
                    .create_symlink(filename, parent_inode_foreign_id, symlink);
            return self.create_forwarder(parent_inode_mount_id, foreign_id);
        }
        let mut x = self.create_inode();
        x.uid = self.inodes[parentid].uid;
        x.gid = self.inodes[parentid].gid;
        x.qid.r#type = (S_IFLNK >> 8) as u8;
        x.symlink = symlink.to_owned().clone();
        x.mode = S_IFLNK;
        self.push_inode(x, Some(parentid), filename);
        return self.inodes.len()-1;
    }

    pub fn create_text_file(&mut self, filename : &str, parentid : usize, data : &str) -> usize {
        let parent_inode = &self.inodes[parentid];
        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let parent_inode_foreign_id = parent_inode.foreign_id.unwrap();
            let foreign_id = 
                self.follow_fs_by_id_mut(parent_inode_mount_id)
                    .create_text_file(filename, parent_inode_foreign_id, data);
            return self.create_forwarder(parent_inode_mount_id, foreign_id);
        }
        let id = self.create_file(filename, parentid);
        let x = &mut self.inodes[id];
        let str_array = string_to_array(data);
        x.size = str_array.data.len();
        self.set_data(id, str_array);
        return id;
    }

    /**
     * @param {UInt8Array} buffer
     */
    pub fn create_binary_file(&mut self, filename : &str, parentid : usize, buffer : &[u8]) -> usize {
        let parent_inode = &self.inodes[parentid];
        if FS::is_forwarder(parent_inode)
        {
            let parent_inode_mount_id = parent_inode.mount_id.unwrap();
            let parent_inode_foreign_id = parent_inode.foreign_id.unwrap();
            let foreign_id =
                self.follow_fs_by_id_mut(parent_inode_mount_id)
                    .create_binary_file(filename, parent_inode_foreign_id, buffer);
            return self.create_forwarder(parent_inode_mount_id, foreign_id);
        }
        let id = self.create_file(filename, parentid);
        let x = &mut self.inodes[id];
        let data = bytes_to_array(buffer);
        x.size = data.data.len();
        self.set_data(id, data);
        return id;
    }


    pub fn open_inode(&mut self, id : usize, mode : usize) -> bool {
        let inode = &self.inodes[id];
        if FS::is_forwarder(inode)
        {
            let inode_mount_id = inode.mount_id.unwrap();
            let inode_foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_by_id_mut(inode_mount_id)
                .open_inode(inode_foreign_id, mode);
        }
        if (inode.mode&S_IFMT) == S_IFDIR {
            self.fill_directory(id);
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

    pub fn close_inode(&mut self, id : usize) {
        //message.Debug("close: " + self.GetFullPath(id));
        let inode = &self.inodes[id];
        if FS::is_forwarder(inode)
        {
            let inode_mount_id = inode.mount_id.unwrap();
            let inode_foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_by_id_mut(inode_mount_id)
                .close_inode(inode_foreign_id);
        }
        //if(inode.status == STATUS_ON_STORAGE)
        //{
        //    self.storage.uncache(inode.sha256sum);
        //}
        if inode.status == STATUS_UNLINKED {
            //message.Debug("Filesystem: Delete unlinked file");
            self.inodes[id].status = STATUS_INVALID;
            self.delete_data(id);
        }
    }
    /**
     * @return {!Promise<number>} 0 if success, or errno if failured.
     */
    pub fn rename(&mut self, olddirid: usize, oldname: &str, newdirid: usize, newname: &str) -> i32 {
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
            print_debug(format!("XXX: Attempted to move mountpoint ({}) - skipped", oldname).as_str());
            return EPERM;
        }
        else if !self.is_directory(idx) && self.get_inode(idx).nlinks > 1
        {
            // Move hardlinked inode vertically in mount tree.
            print_debug(format!("XXX: Attempted to move hardlinked file ({}) across filesystems - skipped", oldname).as_str());
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
            "rename",
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
        self.notify_listeners(id, "write");
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
    
    pub fn search(&mut self, parentid : usize, name : &str) -> Option<usize> {
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
        return path[1..].join("");
    }
    
    /**
     * @param {number} parentid
     * @param {number} targetid
     * @param {string} name
     * @return {number} 0 if success, or errno if failured.
     */
    pub fn link(&mut self, parentid : usize, targetid : usize, name : &str) -> i32 {
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
                print_debug("XXX: Attempted to hardlink a file into a child filesystem - skipped");
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
            print_debug("XXX: Attempted to hardlink file across filesystems - skipped");
            return EPERM;
        }

        self.link_under_dir(parentid, targetid, name);
        return SUCCESS;
    }

    pub fn unlink(&mut self, parentid : usize, name : &str) -> i32 {
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
            self.notify_listeners(idx, "delete");
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

    pub fn is_index_valid(&self, idx : usize) -> bool {
        if idx >= self.inodes.len() {
            false
        } else {
            self.inodes[idx].status != STATUS_INVALID
        }
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

    pub fn get_size(&mut self, idx : usize) -> usize {
        return self.inodes[idx].size;
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

    pub fn search_path(&mut self, path : &str) -> SearchResult {
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
                id = self.search(parentid.unwrap(), walk[i]);
                
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
    

    pub fn recursive_delete(&mut self, path : &str) {
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

    pub fn delete_node(&mut self, path : &str) {
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
    
    pub fn notify_listeners(&self, id: usize, action: &str) {

    }

    /** @param {*=} info */
    pub fn notify_listeners_with_info(&self, id: usize, action: &str, info: NotifyInfo) {
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
                print_debug(format!("Error in filesystem: negative nlinks={} at id ={}", inode.nlinks, i).as_str());
            }

            if self.is_directory(i)
            {
                if self.is_directory(i) && self.get_parent(i).is_none() {
                    print_debug(format!("Error in filesystem: negative parent id {}", i).as_str());
                }
                
                let inode_const = self.get_inode(i);
                for (name, id) in inode_const.direntries.iter()
                {
                    if name.len() == 0 {
                        print_debug(format!("Error in filesystem: inode with no name and id {}", id).as_str());
                    }

                    for c in name.chars() {
                        if (c as u32) < 32 {
                            print_debug("Error in filesystem: Unallowed char in filename");
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
            return inode.direntries.get("..").map(|&x| x);
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
    pub fn mount(&mut self, path : &str, fs : FS) -> (Option<usize>, i32)  {
        debug_assert!(fs.qidcounter.last_qidnumber == self.qidcounter.last_qidnumber,
            "Cannot mount filesystem whose qid numbers aren't synchronised with current filesystem.");

        let path_infos = self.search_path(path);

        if path_infos.parentid.is_none()
        {
            print_debug(format!("Mount failed: parent for path not found: {}", path).as_str());
            return (None, ENOENT);
        }
        if path_infos.id.is_none()
        {
            print_debug(format!("Mount failed: file already exists at path: {}", path).as_str());
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
    /**
     * @param {number} type
     * @param {number} start
     * @param {number} length
     * @param {number} proc_id
     * @param {string} client_id
     * @return {!FSLockRegion}
     */
    pub fn describe_lock(r#type : i32, start : usize, length : usize, proc_id : i32, client_id: &str) -> FSLockRegion {
        debug_assert!(r#type == P9_LOCK_TYPE_RDLCK ||
            r#type == P9_LOCK_TYPE_WRLCK ||
            r#type == P9_LOCK_TYPE_UNLCK,
            "Filesystem: Invalid lock type: {}", r#type);

        return FSLockRegion {
            r#type: r#type,
            start: start,
            length: Some(length),
            proc_id: proc_id,
            client_id: client_id.to_owned().clone()
        };
    }
    /**
     * @param {number} id
     * @param {FSLockRegion} request
     * @return {FSLockRegion} The first conflicting lock found, or null if requested lock is possible.
     */
    pub fn get_lock(&self, id : usize, request : &FSLockRegion) -> Option<FSLockRegion> {
        let inode = &self.inodes[id];

        if FS::is_forwarder(inode)
        {
            return self.follow_fs_immutable(inode)
                .get_lock(inode.foreign_id.unwrap(), request);
        }

        for region in &inode.locks
        {
            if request.conflicts_with(&region)
            {
                return Some(region.clone());
            }
        }
        return None;
    }

    /**
     * @param {number} id
     * @param {FSLockRegion} request
     * @param {number} flags
     * @return {number} One of P9_LOCK_SUCCESS / P9_LOCK_BLOCKED / P9_LOCK_ERROR / P9_LOCK_GRACE.
     */
    pub fn lock(&mut self, id : usize, request : &FSLockRegion, flags : i32) -> i32 {
        let inode = &self.inodes[id];

        if FS::is_forwarder(inode)
        {
            let mount_id = inode.mount_id.unwrap();
            let foreign_id = inode.foreign_id.unwrap();
            return self.follow_fs_by_id_mut(mount_id)
                .lock(foreign_id, request, flags);
        }

        let request_copy_mut = &mut request.clone();
        let request_copy = &*request_copy_mut;

        // (1) Check whether lock is possible before any modification.
        if request_copy.r#type != P9_LOCK_TYPE_UNLCK && self.get_lock(id, request_copy).is_some()
        {
            return P9_LOCK_BLOCKED;
        }
        {
            let inode_mut = &mut self.inodes[id];
            let mut i = 0;
            // (2) Subtract requested region from locks of the same owner.
            loop
            {

                if i >= inode_mut.locks.len() {
                    break;
                }
                let region = &inode_mut.locks[i];

                debug_assert!(region.length.is_some(),
                    "Filesystem: Lock region length should be non None");
                debug_assert!(region.r#type == P9_LOCK_TYPE_RDLCK || region.r#type == P9_LOCK_TYPE_WRLCK,
                    "Filesystem: Found invalid lock type: {}", region.r#type);
                debug_assert!(!i-1 >= inode_mut.locks.len() || inode_mut.locks[i-1].start <= region.start,
                    "Filesystem: Locks should be sorted by starting offset");

                // Skip to requested region.
                if region.start + region.length.unwrap() <= request_copy.start {
                    i += 1;
                    continue;
                } 

                // Check whether we've skipped past the requested region.
                if request_copy.start + request_copy.length.unwrap() <= region.start {
                    break;
                }

                // Skip over locks of different owners.
                if region.proc_id != request_copy.proc_id || region.client_id != request_copy.client_id
                {
                    debug_assert!(!region.conflicts_with(request_copy),
                        "Filesytem: Found conflicting lock region, despite already checked for conflicts");
                    i += 1;
                    continue;
                }

                // Pretend region would be split into parts 1 and 2.
                let start1 = region.start;
                let start2 = request_copy.start + request_copy.length.unwrap();
                let length1 = request_copy.start - start1;
                let length2 = region.start + region.length.unwrap() - start2;

                if length1 > 0 && length2 > 0 && region.r#type == request_copy.r#type
                {
                    // Requested region is already locked with the required type.
                    // Return early - no need to modify anything.
                    return P9_LOCK_SUCCESS;
                }

                if length1 > 0
                {
                    let region_mut = &mut inode_mut.locks[i];
                    // Shrink from right / first half of the split.
                    region_mut.length = Some(length1);
                }

                if length1 <= 0 && length2 > 0
                {
                    let region_mut = &mut inode_mut.locks[i];
                    // Shrink from left.
                    region_mut.start = start2;
                    region_mut.length = Some(length2);
                }
                else if length2 > 0
                {
                    // Add second half of the split.

                    // Fast-forward to correct location.
                    while i < inode_mut.locks.len() && inode_mut.locks[i].start < start2 {
                        i += 1;
                    }

                    let updated_region = &inode_mut.locks[i];

                    inode_mut.locks.insert(i,
                        FS::describe_lock(updated_region.r#type, start2, length2, updated_region.proc_id, &updated_region.client_id));
                }
                else if length1 <= 0
                {
                    // Requested region completely covers this region. Delete.
                    inode_mut.locks.remove(i);
                    i -= 1;
                }
                i += 1;
            }
        }
        // (3) Insert requested lock region as a whole.
        // No point in adding the requested lock region as fragmented bits in the above loop
        // and having to merge them all back into one.
        if request_copy.r#type != P9_LOCK_TYPE_UNLCK
        {
            let mut has_merged = false;
            let mut i = 0;
            let inode_mut = &mut self.inodes[id];

            let mut new_region_i : Option<usize> = None;

            // Fast-forward to request_copyed position, and try merging with previous region.
            loop
            {
                let new_region = 
                    if let Some(ind) = new_region_i {
                        &inode_mut.locks[ind]
                    } else {
                        request_copy
                    };
                
                let i_lock_start = inode_mut.locks[i].start;

                if i >= inode_mut.locks.len() {
                    break;
                }
                if new_region.may_merge_after(&inode_mut.locks[i])
                {
                    let new_length = inode_mut.locks[i].length.unwrap() + request_copy.length.unwrap();
                    inode_mut.locks[i].length = Some(new_length);
                    new_region_i = Some(i);
                    //new_region = &inode_mut.locks[i];
                    has_merged = true;
                }
                if request_copy.start <= i_lock_start {
                    break;
                } 
                i += 1;
            }
            {
                let new_region1 = 
                    if let Some(ind) = new_region_i {
                        &inode_mut.locks[ind]
                    } else {
                        request_copy
                    };

                if !has_merged
                {
                    inode_mut.locks.insert(i, new_region1.clone());
                    i += 1;
                }
                let new_region = 
                    if let Some(ind) = new_region_i {
                        &inode_mut.locks[ind]
                    } else {
                        request_copy
                    };
                
                // Try merging with the subsequent alike region.
                loop 
                {
                    if i >= inode_mut.locks.len() {
                        break;
                    }

                    if !inode_mut.locks[i].is_alike(&new_region){
                        i += 1;
                        continue;
                    }

                    if inode_mut.locks[i].may_merge_after(&new_region)
                    {
                        let new_region_length_size = new_region.length.unwrap() + inode_mut.locks[i].length.unwrap();
                        inode_mut.locks.remove(i);
                        let new_region_mut = 
                            if let Some(ind) = new_region_i {
                                &mut inode_mut.locks[ind]
                            } else {
                                request_copy_mut
                            };
    
                        new_region_mut.length = Some(new_region_length_size);
                    }

                    // No more mergable regions after self.
                    break;
                }
            }
        }

        return P9_LOCK_SUCCESS;
    }
    pub fn read_dir(&mut self, path : &str) -> Option<Vec<String>> {
        let p = self.search_path(path);

        if p.id.is_none()
        {
            return None;
        }

        let dir = self.get_inode(p.id.unwrap());

        let mut result : Vec<String> = Vec::new();

        for key in dir.direntries.keys() {
            if key != "." && key != ".." {
                result.push(key.clone());
            }
        }
        return Some(result);
    }

    pub fn read_text_file(&mut self, file : &str) -> Option<String> {
        return if let Some(data) = self.read_file(file) {
            from_utf8(data).ok().map(String::from)
        }
        else {
            None
        }
    }

    pub fn read_file(&mut self, file : &str) -> Option<&[u8]> {
        let p = self.search_path(file);
        if p.id.is_none()
        {
            return None;
        }

        let inode = self.get_inode(p.id.unwrap());

        return self.read(p.id.unwrap(), 0, inode.size);
    }
}

//#[derive(Serialize, Deserialize, Debug)]
pub struct QID {
    // r# is needed because type is a keyword
    pub r#type: u8,
    pub version: u32,
    pub path: u64
}

/** @constructor */
//#[derive(Serialize, Deserialize, Debug)]
pub struct INode {
    pub direntries : HashMap<String, usize>,
    pub status : i32,
    pub size : usize,
    pub uid : i32,
    pub gid : i32,
    pub fid : usize,
    pub ctime : u64,
    pub atime : u64,
    pub mtime : u64,
    pub major : i32,
    pub minor : i32,
    pub symlink : String,
    pub mode : i32,
    pub qid : QID,
    pub caps : Option<UInt8Array>,
    pub nlinks : i32,
    pub sha256sum : String,

    pub locks : Vec<FSLockRegion>,

    pub mount_id : Option<usize>,
    pub foreign_id : Option<usize>
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


//#[derive(Serialize, Deserialize, Debug)]
pub struct FSMountInfo
{
    pub fs : FS,
    // Maps foreign inode id back to local inode id.
    pub backtrack : HashMap<usize, usize>
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
//#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Clone)]
pub struct FSLockRegion {
    pub r#type : i32,
    pub start : usize,
    pub length : Option<usize>,
    pub proc_id : i32,
    pub client_id : String
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
    
    pub fn clone(&self) -> FSLockRegion {
        return FSLockRegion {
            r#type : self.r#type,
            start : self.start,
            length : self.length.clone(),
            proc_id : self.proc_id,
            client_id : self.client_id.clone()
        };
    }

    pub fn conflicts_with(&self, region: &FSLockRegion) -> bool {
        if self.proc_id == region.proc_id && self.client_id == region.client_id { return false; }
        if self.r#type == P9_LOCK_TYPE_UNLCK || region.r#type == P9_LOCK_TYPE_UNLCK { return false; }
        if self.r#type != P9_LOCK_TYPE_WRLCK && region.r#type != P9_LOCK_TYPE_WRLCK { return false; }
        // ADDED is_some test here
        if self.length.is_some() && self.start + self.length.unwrap() <= region.start { return false; }
        if region.length.is_some() && region.start + region.length.unwrap() <= self.start { return false; }
        return true;    
    }

    pub fn is_alike(&self, region : &FSLockRegion) -> bool {
        return region.proc_id == self.proc_id &&
            region.client_id == self.client_id &&
            region.r#type == self.r#type;
    }

    pub fn may_merge_after(&self, region : &FSLockRegion) -> bool {
        // Added: is_some
        return self.is_alike(region) && region.length.is_some() && region.start + region.length.unwrap() == self.start;
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

}
*/