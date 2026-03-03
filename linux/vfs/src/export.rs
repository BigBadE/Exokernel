//! C-compatible VFS exports
//!
//! This module provides all the C-callable functions that Linux
//! filesystem drivers need to interact with the VFS.
//!
//! Functions are written in safe Rust using references, and the `linux_ffi!`
//! macro in the `ffi` submodule generates the unsafe FFI wrappers automatically.

use alloc::alloc::{alloc_zeroed, Layout};
use core::ffi::{c_char, c_int, c_long, c_uint, c_ulong, c_void};
use core::ptr;

use linux_core::{linux_export, Errno, KernelResult, KernelResultExt, OptionExt};
use crate::types::*;
use crate::inode::*;
use crate::dentry::*;
use crate::file::*;
use crate::superblock::*;
use crate::operations::*;
use crate::dcache;
use crate::mount;
use crate::fs_type;

// ============================================================================
// Inode allocation and management
// ============================================================================

/// Allocate a new inode
#[linux_export]
unsafe fn new_inode(sb: *mut super_block) -> *mut inode {
    if sb.is_null() {
        return ptr::null_mut();
    }

    // Check if superblock has custom alloc_inode
    if let Some(alloc_fn) = (*sb).s_op.as_ref().and_then(|op| (*op).alloc_inode) {
        return alloc_fn(sb);
    }

    // Default allocation
    let layout = match Layout::from_size_align(core::mem::size_of::<inode>(), 8) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };

    let ptr = alloc_zeroed(layout) as *mut inode;
    if ptr.is_null() {
        return ptr::null_mut();
    }

    (*ptr).i_sb = sb;
    (*ptr).i_nlink = 1;
    (*ptr).i_mapping = &raw mut (*ptr).i_data;
    (*ptr).i_data.host = ptr;
    (*ptr).i_state = I_NEW;
    (*ptr).i_writecount = atomic_t::new(0);
    (*ptr).i_io_list.init();
    (*ptr).i_wb_list.init();
    (*ptr).i_lru.init();
    (*ptr).i_sb_list.init();
    (*ptr).i_devices.init();

    ptr
}

/// Allocate inode from superblock (for slab allocator)
#[linux_export]
unsafe fn alloc_inode_sb(sb: *mut super_block, _cache: *mut c_void, _gfp: gfp_t) -> *mut inode {
    // For now, just allocate normally
    new_inode(sb)
}

/// Initialize inode once (for slab)
#[linux_export]
unsafe fn inode_init_once(inode: *mut inode) {
    if inode.is_null() {
        return;
    }
    (*inode).i_io_list.init();
    (*inode).i_wb_list.init();
    (*inode).i_lru.init();
    (*inode).i_sb_list.init();
    (*inode).i_devices.init();
}

/// Put (release) an inode
#[linux_export]
fn iput(_inode: &mut inode) {
    // Simplified: would decrement refcount and possibly free
}

/// Get inode reference
#[linux_export]
fn igrab(inode: &mut inode) -> &mut inode {
    // Would increment refcount
    inode
}

/// Clear inode
#[linux_export]
fn clear_inode(inode: &mut inode) {
    inode.i_state = I_FREEING | I_CLEAR;
}

/// Mark inode dirty with flags (internal)
#[linux_export]
fn __mark_inode_dirty(inode: &mut inode, flags: c_int) {
    inode.i_state |= flags as c_ulong & I_DIRTY;
}

/// Mark inode dirty
#[linux_export]
unsafe fn mark_inode_dirty(inode: *mut inode) {
    if !inode.is_null() {
        (*inode).i_state |= I_DIRTY;
    }
}

/// Mark inode dirty for sync
#[linux_export]
unsafe fn mark_inode_dirty_sync(inode: *mut inode) {
    if !inode.is_null() {
        (*inode).i_state |= I_DIRTY_SYNC;
    }
}

/// Insert inode into hash
#[linux_export]
fn insert_inode_hash(_inode: *mut inode) {
    // Would add to inode hash table
}

/// Lookup inode by number
#[linux_export]
fn ilookup(_sb: *mut super_block, _ino: c_ulong) -> *mut inode {
    // Simplified: would search inode hash
    ptr::null_mut()
}

/// Get unique inode number
#[linux_export]
unsafe fn iunique(_sb: *mut super_block, max_reserved: c_ulong) -> c_ulong {
    static mut COUNTER: c_ulong = 0;
    COUNTER += 1;
    if COUNTER <= max_reserved {
        COUNTER = max_reserved + 1;
    }
    COUNTER
}

// ============================================================================
// Link count operations
// ============================================================================

/// Set link count
#[linux_export]
fn set_nlink(inode: &mut inode, nlink: c_uint) {
    inode.i_nlink = nlink;
}

/// Increment link count
#[linux_export]
fn inc_nlink(inode: &mut inode) {
    inode.i_nlink += 1;
}

/// Decrement link count
#[linux_export]
fn drop_nlink(inode: &mut inode) {
    if inode.i_nlink > 0 {
        inode.i_nlink -= 1;
    }
}

/// Clear link count
#[linux_export]
fn clear_nlink(inode: &mut inode) {
    inode.i_nlink = 0;
}

// ============================================================================
// Inode size operations
// ============================================================================

/// Read inode size
#[linux_export]
fn i_size_read(inode: &inode) -> loff_t {
    inode.i_size
}

/// Write inode size
#[linux_export]
fn i_size_write(inode: &mut inode, size: loff_t) {
    inode.i_size = size;
}

// ============================================================================
// Inode locking
// ============================================================================

/// Lock inode mutex
#[linux_export]
fn inode_lock(_inode: &mut inode) {
    // Simplified: would actually lock
}

/// Unlock inode mutex
#[linux_export]
fn inode_unlock(_inode: &mut inode) {
    // Simplified: would actually unlock
}

/// Lock inode for shared access
#[linux_export]
fn inode_lock_shared(_inode: &mut inode) {
    // Simplified
}

/// Unlock inode shared
#[linux_export]
fn inode_unlock_shared(_inode: &mut inode) {
    // Simplified
}

/// Wait for direct IO to complete
#[linux_export]
fn inode_dio_wait(_inode: &mut inode) {
    // Simplified
}

/// Check if inode needs sync
#[linux_export]
fn inode_needs_sync(inode: &inode) -> c_int {
    ((inode.i_state & I_DIRTY_SYNC) != 0) as c_int
}

/// Check inode version equality
#[linux_export]
fn inode_eq_iversion(inode: &inode, version: u64) -> bool {
    inode.i_version == version
}

// ============================================================================
// Dentry operations
// ============================================================================

/// Make root dentry
#[linux_export]
unsafe fn d_make_root(root_inode: *mut inode) -> *mut dentry {
    dcache::d_make_root_fn(root_inode)
}

/// Instantiate dentry
#[linux_export]
unsafe fn d_instantiate(dentry: *mut dentry, inode: *mut inode) {
    dcache::d_instantiate_fn(dentry, inode);
}

/// Splice alias
#[linux_export]
unsafe fn d_splice_alias(inode: *mut inode, dentry: *mut dentry) -> *mut dentry {
    dcache::d_splice_alias_fn(inode, dentry)
}

/// Find alias
#[linux_export]
unsafe fn d_find_alias(inode: *mut inode) -> *mut dentry {
    dcache::d_find_alias_fn(inode)
}

/// Move dentry
#[linux_export]
unsafe fn d_move(dentry: *mut dentry, target: *mut dentry) {
    dcache::d_move_fn(dentry, target);
}

/// Obtain alias
#[linux_export]
unsafe fn d_obtain_alias(inode: *mut inode) -> *mut dentry {
    dcache::d_obtain_alias_fn(inode)
}

/// Get inode from dentry
#[linux_export]
unsafe fn d_inode(dentry: *const dentry) -> *mut inode {
    d_inode_fn(dentry)
}

/// Check if dentry is really positive
#[linux_export]
unsafe fn d_really_is_positive(dentry: *const dentry) -> bool {
    dcache::d_really_is_positive_fn(dentry)
}

/// Put dentry
#[linux_export]
unsafe fn dput(dentry: *mut dentry) {
    dcache::dput_fn(dentry);
}

// ============================================================================
// File operations
// ============================================================================

/// Get inode from file
#[linux_export]
unsafe fn file_inode(f: *const file) -> *mut inode {
    file_inode_fn(f)
}

/// Get mount ID map from file
#[linux_export]
fn file_mnt_idmap(_f: *const file) -> *mut mnt_idmap {
    // Simplified: return null (no ID mapping)
    ptr::null_mut()
}

// ============================================================================
// Filesystem registration
// ============================================================================

/// Register filesystem
#[linux_export]
unsafe fn register_filesystem(fs: *mut file_system_type) -> c_int {
    fs_type::register_filesystem_fn(fs)
}

/// Unregister filesystem
#[linux_export]
unsafe fn unregister_filesystem(fs: *mut file_system_type) -> c_int {
    fs_type::unregister_filesystem_fn(fs)
}

/// Get filesystem type
#[linux_export]
unsafe fn get_fs_type(name: *const c_char) -> *mut file_system_type {
    fs_type::get_fs_type_fn(name)
}

// ============================================================================
// Mount operations
// ============================================================================

/// Mount block device
#[linux_export]
unsafe fn mount_bdev(
    fs_type: *mut file_system_type,
    flags: c_int,
    dev_name: *const c_char,
    data: *mut c_void,
    fill_super: Option<unsafe extern "C" fn(*mut super_block, *mut c_void, c_int) -> c_int>,
) -> *mut dentry {
    mount::mount_bdev_fn(fs_type, flags, dev_name, data, fill_super)
}

/// Kill block super
#[linux_export]
unsafe fn kill_block_super(sb: *mut super_block) {
    mount::kill_block_super_fn(sb);
}

/// Sync filesystem
#[linux_export]
unsafe fn sync_filesystem(sb: *mut super_block) -> c_int {
    if sb.is_null() {
        return -22;
    }
    if let Some(sync_fn) = (*sb).s_op.as_ref().and_then(|op| (*op).sync_fs) {
        return sync_fn(sb, 1);
    }
    0
}

// ============================================================================
// Attribute operations
// ============================================================================

/// Generic getattr
#[linux_export]
fn generic_fillattr(
    _idmap: &mut mnt_idmap,
    _request_mask: u32,
    inode: &inode,
    stat: &mut kstat,
) {
    stat.ino = inode.i_ino;
    stat.mode = inode.i_mode;
    stat.nlink = inode.i_nlink;
    stat.uid = inode.i_uid;
    stat.gid = inode.i_gid;
    stat.size = inode.i_size;
    stat.atime = inode.i_atime;
    stat.mtime = inode.i_mtime;
    stat.ctime = inode.i_ctime;
    stat.blocks = inode.i_blocks;
    stat.blksize = 1 << inode.i_blkbits;
}

/// Prepare for setattr
#[linux_export]
fn setattr_prepare(_idmap: &mut mnt_idmap, _dentry: &mut dentry, _attr: &mut iattr) -> KernelResult<()> {
    // Simplified: allow all changes
    Ok(())
}

/// Copy attributes from iattr to inode
#[linux_export]
fn setattr_copy(_idmap: &mut mnt_idmap, inode: &mut inode, attr: &iattr) {
    let valid = attr.ia_valid;

    if (valid & ATTR_MODE) != 0 {
        inode.i_mode = attr.ia_mode;
    }
    if (valid & ATTR_UID) != 0 {
        inode.i_uid = attr.ia_uid;
    }
    if (valid & ATTR_GID) != 0 {
        inode.i_gid = attr.ia_gid;
    }
    if (valid & ATTR_SIZE) != 0 {
        inode.i_size = attr.ia_size;
    }
    if (valid & ATTR_ATIME) != 0 {
        inode.i_atime = attr.ia_atime;
    }
    if (valid & ATTR_MTIME) != 0 {
        inode.i_mtime = attr.ia_mtime;
    }
    if (valid & ATTR_CTIME) != 0 {
        inode.i_ctime = attr.ia_ctime;
    }
}

// Attribute flags
pub const ATTR_MODE: c_uint = 1;
pub const ATTR_UID: c_uint = 2;
pub const ATTR_GID: c_uint = 4;
pub const ATTR_SIZE: c_uint = 8;
pub const ATTR_ATIME: c_uint = 16;
pub const ATTR_MTIME: c_uint = 32;
pub const ATTR_CTIME: c_uint = 64;
pub const ATTR_ATIME_SET: c_uint = 128;
pub const ATTR_MTIME_SET: c_uint = 256;
pub const ATTR_FORCE: c_uint = 512;
pub const ATTR_KILL_SUID: c_uint = 2048;
pub const ATTR_KILL_SGID: c_uint = 4096;
pub const ATTR_FILE: c_uint = 8192;
pub const ATTR_KILL_PRIV: c_uint = 16384;
pub const ATTR_OPEN: c_uint = 32768;
pub const ATTR_TIMES_SET: c_uint = 65536;
pub const ATTR_TOUCH: c_uint = 131072;

// ============================================================================
// User/group operations
// ============================================================================

/// Get current UID
#[linux_export]
fn current_uid() -> kuid_t {
    0 // root for now
}

/// Get current GID
#[linux_export]
fn current_gid() -> kgid_t {
    0 // root for now
}

/// Get current fsuid
#[linux_export]
fn current_fsuid() -> kuid_t {
    0
}

/// Get current umask
#[linux_export]
fn current_umask() -> umode_t {
    0o022
}

/// Get current user namespace
#[linux_export]
fn current_user_ns() -> *mut user_namespace {
    ptr::null_mut()
}

/// Init user namespace
#[unsafe(no_mangle)]
pub static mut init_user_ns: user_namespace = user_namespace { _opaque: [] };

/// Convert uid to vfsuid
#[linux_export]
fn i_uid_into_vfsuid(_idmap: &mut mnt_idmap, inode: &inode) -> kuid_t {
    inode.i_uid
}

/// Convert gid to vfsgid
#[linux_export]
fn i_gid_into_vfsgid(_idmap: &mut mnt_idmap, inode: &inode) -> kgid_t {
    inode.i_gid
}

/// Check vfsuid equality
#[linux_export]
fn vfsuid_eq_kuid(vfsuid: kuid_t, kuid: kuid_t) -> bool {
    vfsuid == kuid
}

/// Check if vfsgid is in group
#[linux_export]
fn vfsgid_in_group_p(_vfsgid: kgid_t) -> bool {
    // Simplified: root is in all groups
    true
}

// ============================================================================
// Directory operations
// ============================================================================

/// Generic read dir (returns -EISDIR)
#[linux_export]
fn generic_read_dir(
    _filp: &mut file,
    _buf: *mut c_char, // raw pointer as it's a buffer
    _size: usize,
    _ppos: &mut loff_t,
) -> isize {
    -21 // -EISDIR
}

/// Emit . and .. entries
#[linux_export]
fn dir_emit_dots(_file: &mut file, _ctx: &mut dir_context) -> bool {
    // Simplified: would emit . and ..
    true
}

// ============================================================================
// NFS export operations
// ============================================================================

/// Generic fh_to_dentry (returns null for now)
#[linux_export]
fn generic_fh_to_dentry(
    _sb: *mut super_block,
    _fid: *mut c_void,
    _fh_len: c_int,
    _fh_type: c_int,
) -> *mut dentry {
    ptr::null_mut()
}

/// Generic fh_to_parent (returns null for now)
#[linux_export]
fn generic_fh_to_parent(
    _sb: *mut super_block,
    _fid: *mut c_void,
    _fh_len: c_int,
    _fh_type: c_int,
) -> *mut dentry {
    ptr::null_mut()
}

// ============================================================================
// Misc VFS operations
// ============================================================================

/// Write inode now
#[linux_export]
fn write_inode_now(_inode: &mut inode, _sync: c_int) -> KernelResult<()> {
    Ok(())
}

/// Sync inode metadata
#[linux_export]
fn sync_inode_metadata(_inode: &mut inode, _wait: c_int) -> KernelResult<()> {
    Ok(())
}

/// Generic file fsync
#[linux_export]
fn __generic_file_fsync(
    _file: &mut file,
    _start: loff_t,
    _end: loff_t,
    _datasync: c_int,
) -> KernelResult<()> {
    Ok(())
}

/// Generic cont expand simple
#[linux_export]
fn generic_cont_expand_simple(inode: &mut inode, size: loff_t) -> KernelResult<()> {
    if size > inode.i_size {
        inode.i_size = size;
    }
    Ok(())
}

/// Invalidate inodes
#[linux_export]
fn invalidate_inodes(_sb: &mut super_block, _kill_dirty: c_int) -> KernelResult<()> {
    Ok(())
}

// ============================================================================
// Checking macros as functions
// ============================================================================

/// Check IS_DEADDIR
#[linux_export]
fn IS_DEADDIR(inode: &inode) -> bool {
    (inode.i_flags & S_DEAD) != 0
}

/// Check IS_SYNC
#[linux_export]
fn IS_SYNC(inode: &inode) -> bool {
    (inode.i_flags & S_SYNC) != 0
}

/// Check IS_DIRSYNC
#[linux_export]
fn IS_DIRSYNC(inode: &inode) -> bool {
    (inode.i_flags & S_DIRSYNC) != 0 || (inode.i_flags & S_SYNC) != 0
}

// ============================================================================
// Filemap operations (page cache)
// ============================================================================

/// Filemap fdatawrite
#[linux_export]
fn filemap_fdatawrite(_mapping: &mut address_space) -> KernelResult<()> {
    Ok(())
}

/// Filemap fdatawrite range
#[linux_export]
fn filemap_fdatawrite_range(
    _mapping: &mut address_space,
    _start: loff_t,
    _end: loff_t,
) -> KernelResult<()> {
    Ok(())
}

/// Filemap fdatawait range
#[linux_export]
fn filemap_fdatawait_range(
    _mapping: &mut address_space,
    _start: loff_t,
    _end: loff_t,
) -> KernelResult<()> {
    Ok(())
}

/// Filemap splice read
#[linux_export]
fn filemap_splice_read(
    _file: *mut file,
    _ppos: *mut loff_t,
    _pipe: *mut c_void,
    _len: usize,
    _flags: c_uint,
) -> isize {
    -38 // -ENOSYS
}

/// Iter file splice write
#[linux_export]
fn iter_file_splice_write(
    _pipe: *mut c_void,
    _out: *mut file,
    _ppos: *mut loff_t,
    _len: usize,
    _flags: c_uint,
) -> isize {
    -38 // -ENOSYS
}

// ============================================================================
// Mpage operations
// ============================================================================

/// Mpage read folio
#[linux_export]
fn mpage_read_folio(_folio: *mut c_void, _get_block: *mut c_void) -> c_int {
    -38 // -ENOSYS
}

/// Mpage readahead
#[linux_export]
fn mpage_readahead(_rac: *mut c_void, _get_block: *mut c_void) {
    // No-op
}

/// Mpage writepages
#[linux_export]
fn mpage_writepages(
    _mapping: *mut address_space,
    _wbc: *mut writeback_control,
    _get_block: *mut c_void,
) -> c_int {
    0
}

// ============================================================================
// Block device direct IO
// ============================================================================

/// Blockdev direct IO
#[linux_export]
fn blockdev_direct_IO(
    _iocb: *mut kiocb,
    _iter: *mut iov_iter,
    _get_block: *mut c_void,
) -> isize {
    -38 // -ENOSYS
}

// ============================================================================
// Name operations
// ============================================================================

/// Allocate name buffer
#[linux_export]
unsafe fn __getname() -> *mut c_char {
    use alloc::alloc::{alloc, Layout};
    const PATH_MAX: usize = 4096;
    let layout = match Layout::from_size_align(PATH_MAX, 1) {
        Ok(l) => l,
        Err(_) => return ptr::null_mut(),
    };
    alloc(layout) as *mut c_char
}

/// Free name buffer
#[linux_export]
fn __putname(_name: *mut c_char) {
    // Would free but we don't track allocations yet
}

// ============================================================================
// I/O iterator operations
// ============================================================================

/// Get count from iov_iter
#[linux_export]
fn iov_iter_count(iter: &iov_iter) -> usize {
    iter.count
}

/// Check if read or write
#[linux_export]
fn iov_iter_rw(iter: &iov_iter) -> c_int {
    if iter.data_source { 1 } else { 0 } // WRITE : READ
}

// ============================================================================
// Mount write operations
// ============================================================================

/// Want write on mount
#[linux_export]
fn mnt_want_write_file(_file: &mut file) -> KernelResult<()> {
    Ok(())
}

/// Drop write on mount
#[linux_export]
fn mnt_drop_write_file(_file: &mut file) {
    // No-op
}

// ============================================================================
// Compat ioctl
// ============================================================================

/// Compat pointer ioctl
#[linux_export]
fn compat_ptr_ioctl(_file: &mut file, _cmd: c_uint, _arg: c_ulong) -> c_long {
    -25 // -ENOTTY
}
