#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::mem::MaybeUninit;

use rusl::error::Errno;
pub use rusl::platform::Mode;
use rusl::platform::{Dirent, OpenFlags, Stat, NULL_BYTE};
use rusl::string::strlen::{buf_strlen, strlen};
use rusl::string::unix_str::UnixStr;
use rusl::unistd::UnlinkFlags;

use crate::error::Error;
use crate::error::Result;
use crate::io::{Read, Write};
use crate::unix::fd::{AsRawFd, BorrowedFd, OwnedFd, RawFd};

#[cfg(test)]
mod test;

pub struct File(OwnedFd);

impl File {
    /// Opens a file with default options
    /// # Errors
    /// Operating system errors ond finding and reading files
    #[inline]
    pub fn open(path: &UnixStr) -> Result<Self> {
        Self::open_with_options(path, OpenOptions::new().read(true))
    }

    #[inline]
    fn open_with_options(path: &UnixStr, opts: &OpenOptions) -> Result<Self> {
        let flags =
            OpenFlags::O_CLOEXEC | opts.get_access_mode()? | opts.get_creation_mode()? | opts.flags;
        let fd = rusl::unistd::open_mode(path, flags, opts.mode)?;
        Ok(File(OwnedFd(fd)))
    }

    fn open_at(dir_fd: RawFd, path: &UnixStr) -> Result<Self> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let flags =
            OpenFlags::O_CLOEXEC | opts.get_access_mode()? | opts.get_creation_mode()? | opts.flags;
        let fd = rusl::unistd::open_at_mode(dir_fd, path, flags, opts.mode)?;
        Ok(File(OwnedFd(fd)))
    }

    /// Create a File from a raw `fd`
    /// # Safety
    /// The fd is valid and is not duplicated.
    /// Duplication is bad since the `fd` will be closed when this `File` is dropped
    #[must_use]
    pub const unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self(OwnedFd::from_raw(fd))
    }

    /// Set this `File` to be non-blocking.
    /// This will result in for example read expecting a certain number of bytes
    /// to fail with `EAGAIN` if that data isn't available.
    /// # Errors
    /// Errors making the underlying syscalls
    #[inline]
    pub fn set_nonblocking(&self) -> Result<()> {
        self.0.set_nonblocking()
    }

    /// Get file metadata for this open `File`
    /// # Errors
    /// Os errors making the stat-syscall
    #[inline]
    pub fn metadata(&self) -> Result<Metadata> {
        let stat = rusl::unistd::stat_fd(self.as_raw_fd())?;
        Ok(Metadata(stat))
    }

    /// Copies `src` to `dest`, can be used to move files.
    /// Will overwrite anything currently at `dest`.
    /// Returns a handle to the new file.
    /// # Errors
    /// Os errors relating to file access
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    pub fn copy(&self, dest: &UnixStr) -> Result<Self> {
        let this_metadata = self.metadata()?;
        let dest = OpenOptions::new()
            .create(true)
            .write(true)
            .mode(this_metadata.mode())
            .open(dest)?;
        let mut offset = 0;
        // We don't have to care about sign loss on the st_size, it's always positive.
        let mut remaining = this_metadata.0.st_size as u64 - offset;
        while remaining > 0 {
            let w = rusl::unistd::copy_file_range(
                self.as_raw_fd(),
                offset,
                dest.as_raw_fd(),
                offset,
                remaining as usize,
            )?;
            if w == 0 {
                return Ok(dest);
            }
            offset += w as u64;
            remaining = this_metadata.0.st_size as u64 - offset;
        }
        Ok(dest)
    }
}

impl File {
    #[inline]
    pub(crate) fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl AsRawFd for File {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl Read for File {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(rusl::unistd::read(self.0 .0, buf)?)
    }
}

/// Reads a file into a newly allocated vec
/// # Errors
/// Os errors relating to file access and reading
#[cfg(feature = "alloc")]
pub fn read(path: &UnixStr) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Reads a file into a newly allocated vec
/// # Errors
/// Os errors relating to file access and reading as well as utf8 conversion errors
#[cfg(feature = "alloc")]
pub fn read_to_string(path: &UnixStr) -> Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Attempts to write the entire contents of `buf` into the path specified at `path`.
/// If no file exists at `path` one will be created.
/// If a file exists at `path` it will be overwritten.
/// Use `File` and open with `append` to append to a file.
/// # Errors
/// Os errors relating to file creation or writing, such as permissions errors.
pub fn write(path: &UnixStr, buf: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(buf)?;
    Ok(())
}
#[derive(Debug, Clone)]
pub struct Metadata(Stat);

impl Metadata {
    #[inline]
    #[must_use]
    pub fn is_dir(&self) -> bool {
        Mode::from(self.0.st_mode) & Mode::S_IFMT == Mode::S_IFDIR
    }

    #[inline]
    #[must_use]
    pub fn is_file(&self) -> bool {
        Mode::from(self.0.st_mode) & Mode::S_IFMT == Mode::S_IFREG
    }

    #[inline]
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        Mode::from(self.0.st_mode) & Mode::S_IFMT == Mode::S_IFLNK
    }

    #[inline]
    #[must_use]
    pub fn mode(&self) -> Mode {
        Mode::from(self.0.st_mode)
    }

    #[inline]
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    #[allow(clippy::cast_sign_loss)]
    pub fn len(&self) -> u64 {
        self.0.st_size as u64
    }
}

/// Reads metadata at path
/// # Errors
/// Os errors relating to file access
#[inline]
pub fn metadata(path: &UnixStr) -> Result<Metadata> {
    let res = rusl::unistd::stat(path)?;
    Ok(Metadata(res))
}

/// Renames `src` to `dest`, can be used to move files or directories.
/// Will overwrite anything currently at `dest`.
/// # Errors
/// Os errors relating to file access
#[inline]
pub fn rename(src: &UnixStr, dest: &UnixStr) -> Result<()> {
    rusl::unistd::rename(src, dest)?;
    Ok(())
}

/// Copies the file at `src`, to the `dest`, overwriting anything at `dest`.
/// Returns a handle to the new file.
/// # Errors
/// See [`File::copy`]
#[inline]
pub fn copy_file(src: &UnixStr, dest: &UnixStr) -> Result<File> {
    let src_file = File::open(src)?;
    src_file.copy(dest)
}

/// Checks if anything exists at the provided path.
/// Will false-negative if the path is empty.
/// # Errors
/// Os errors relating to file access
pub fn exists(path: &UnixStr) -> Result<bool> {
    match rusl::unistd::stat(path) {
        Ok(_) => Ok(true),
        Err(e) => {
            if matches!(e.code, Some(Errno::ENOENT)) {
                return Ok(false);
            }
            Err(Error::from(e))
        }
    }
}

/// Tries to remove a file from the filesystem at the specified path
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn remove_file(path: &UnixStr) -> Result<()> {
    rusl::unistd::unlink(path)?;
    Ok(())
}

/// Tries to create a directory at the specified path
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn create_dir(path: &UnixStr) -> Result<()> {
    create_dir_mode(path, Mode::from(0o755))
}

/// Create a directory with the given mode
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn create_dir_mode(path: &UnixStr, mode: Mode) -> Result<()> {
    rusl::unistd::mkdir(path, mode)?;
    Ok(())
}

/// Tries to create a directory at the specified path
/// # Errors
/// OS errors relating to file access/permissions
/// If working without an allocator, the maximum path length is 512 bytes
#[inline]
pub fn create_dir_all(path: &UnixStr) -> Result<()> {
    // To make it simple we'll just stack alloc an uninit 512 array for the path.
    // Kind of a travesty, but needs to be like this to work without an allocator
    const NO_ALLOC_MAX_LEN: usize = 512;
    const EMPTY: [MaybeUninit<u8>; NO_ALLOC_MAX_LEN] = [MaybeUninit::uninit(); NO_ALLOC_MAX_LEN];
    unsafe {
        // Could check if we haven't got any slashes at all and just run the pointer straight through
        // without possibly having to add an extra indirection buffer here.
        let ptr = path.as_ptr();
        let len = strlen(ptr);
        if len == 0 {
            return Err(Error::no_code(
                "Can't create a directory with an empty name",
            ));
        }
        #[cfg(feature = "alloc")]
        if len > NO_ALLOC_MAX_LEN {
            let mut owned: Vec<u8> = Vec::with_capacity(len);
            ptr.copy_to(owned.as_mut_ptr(), len);
            write_all_sub_paths(owned.as_mut_slice(), ptr)?;
            return Ok(());
        }
        #[cfg(not(feature = "alloc"))]
        if len > NO_ALLOC_MAX_LEN {
            return Err(Error::no_code(
                "Supplied path larger than 512 without an allocator present",
            ));
        }
        let mut copied = EMPTY;
        ptr.copy_to(copied.as_mut_ptr().cast(), len);
        // Make into slice, we know the actual slice-length is len + 1
        let initialized_section = copied[..len].as_mut_ptr().cast();
        let buf: &mut [u8] = core::slice::from_raw_parts_mut(initialized_section, len);
        write_all_sub_paths(buf, ptr)?;
    }
    Ok(())
}

#[inline]
unsafe fn write_all_sub_paths(
    buf: &mut [u8],
    raw: *const u8,
) -> core::result::Result<(), rusl::Error> {
    let len = buf.len();
    let mut it = 1;
    loop {
        // Iterate down
        let ind = len - it;
        if ind == 0 {
            break;
        }

        let byte = buf[ind];
        if byte == b'/' {
            // Swap slash for null termination to make a valid path
            buf[ind] = NULL_BYTE;

            return match rusl::unistd::mkdir(
                UnixStr::from_bytes_unchecked(&buf[..=ind]),
                Mode::from(0o755),
            ) {
                // Successfully wrote, traverse down
                Ok(()) => {
                    // Replace the null byte to make a valid path concatenation
                    buf[ind] = b'/';
                    for i in ind + 1..len {
                        // Found next
                        if buf[i] == b'/' {
                            // Swap slash for null termination to make a valid path
                            buf[i] = NULL_BYTE;
                            rusl::unistd::mkdir(
                                UnixStr::from_bytes_unchecked(&buf[..=i]),
                                Mode::from(0o755),
                            )?;
                            // Swap back to continue down
                            buf[i] = b'/';
                        }
                    }
                    // if we end on a slash we don't have to write the last part
                    if unsafe { raw.add(len - 1).read() } == b'/' {
                        return Ok(());
                    }
                    // We know the actual length is len + 1 and null terminated, try write full
                    rusl::unistd::mkdir(
                        UnixStr::from_bytes_unchecked(core::slice::from_raw_parts(raw, len + 1)),
                        Mode::from(0o755),
                    )?;
                    Ok(())
                }
                Err(e) => {
                    if let Some(code) = e.code {
                        if code == Errno::ENOENT {
                            it += 1;
                            // Put slash back, only way we end up here is if we tried to write
                            // previously replacing the slash with a null-byte
                            buf[ind] = b'/';
                            continue;
                        } else if code == Errno::EEXIST {
                            return Ok(());
                        }
                    }
                    Err(e)
                }
            };
        }
        it += 1;
    }
    Ok(())
}

pub struct Directory(OwnedFd);

impl Directory {
    /// Opens a directory for reading
    /// # Errors
    /// OS errors relating to file access/permissions
    #[inline]
    pub fn open(path: &UnixStr) -> Result<Directory> {
        let fd = rusl::unistd::open(path, OpenFlags::O_CLOEXEC | OpenFlags::O_RDONLY)?;
        Ok(Directory(OwnedFd(fd)))
    }

    #[inline]
    fn open_at(dir_fd: RawFd, path: &UnixStr) -> Result<Directory> {
        let fd = rusl::unistd::open_at(dir_fd, path, OpenFlags::O_CLOEXEC | OpenFlags::O_RDONLY)?;
        Ok(Directory(OwnedFd(fd)))
    }

    /// Will try to read a directory into a 512 byte buffer.
    /// The iterator will try to keep requesting entries until the directory is EOF or produces an error.
    /// Best case number of syscalls to drain the interator is 2.
    /// Worst case, assuming each entity has the linux-max name of 256 bytes is
    /// `n + 1` when `n` is the number of files.
    #[must_use]
    pub fn read<'a>(&self) -> ReadDir<'a> {
        let buf = [0u8; 512];
        ReadDir {
            fd: BorrowedFd::new(self.0 .0),
            filled_buf: buf,
            offset: 0,
            read_size: 0,
            eod: false,
        }
    }

    /// Removes all children of this directory.
    /// Potentially destructive.
    /// # Errors
    /// Os errors relating to permissions
    pub fn remove_all(&self) -> Result<()> {
        for sub_dir in self.read() {
            let sub_dir = sub_dir?;
            if FileType::Directory == sub_dir.file_type() {
                if sub_dir.is_relative_reference() {
                    continue;
                }
                let fname = sub_dir.file_unix_name()?;
                let next = Self::open_at(self.0 .0, fname)?;
                next.remove_all()?;
                rusl::unistd::unlink_at(self.0 .0, fname, UnlinkFlags::at_removedir())?;
            } else {
                rusl::unistd::unlink_at(
                    self.0 .0,
                    sub_dir.file_unix_name()?,
                    UnlinkFlags::empty(),
                )?;
            }
        }
        Ok(())
    }
}

pub struct ReadDir<'a> {
    fd: BorrowedFd<'a>,
    // Maximum
    filled_buf: [u8; 512],
    offset: usize,
    read_size: usize,
    eod: bool,
}

impl<'a> Iterator for ReadDir<'a> {
    type Item = Result<DirEntry<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.read_size == self.offset {
            if self.eod {
                return None;
            }
            match rusl::unistd::get_dents(self.fd.fd, &mut self.filled_buf) {
                Ok(read) => {
                    if read == 0 {
                        self.eod = true;
                        return None;
                    }
                    self.read_size = read;
                    // Luckily we don't read any partials, so no shift-back the buffer
                    self.offset = 0;
                }
                Err(e) => {
                    self.eod = true;
                    return Some(Err(e.into()));
                }
            }
        }
        unsafe {
            Dirent::try_from_bytes(&self.filled_buf[self.offset..])
                .map(|d| {
                    self.offset += d.d_reclen as usize;
                    d
                })
                .map(|de| {
                    Ok(DirEntry {
                        inner: de,
                        fd: self.fd,
                    })
                })
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FileType {
    Fifo,
    CharDevice,
    Directory,
    BlockDevice,
    RegularFile,
    Symlink,
    Socket,
    Unknown,
}

pub struct DirEntry<'a> {
    inner: Dirent,
    fd: BorrowedFd<'a>,
}

impl<'a> DirEntry<'a> {
    /// If the dir entry is `.` or `..`
    #[must_use]
    #[inline]
    pub fn is_relative_reference(&self) -> bool {
        &self.inner.d_name[..2] == b".\0" || &self.inner.d_name[..3] == b"..\0"
    }

    #[must_use]
    pub fn file_type(&self) -> FileType {
        match self.inner.d_type {
            rusl::platform::DirType::DT_FIFO => FileType::Fifo,
            rusl::platform::DirType::DT_CHR => FileType::CharDevice,
            rusl::platform::DirType::DT_DIR => FileType::Directory,
            rusl::platform::DirType::DT_BLK => FileType::BlockDevice,
            rusl::platform::DirType::DT_REG => FileType::RegularFile,
            rusl::platform::DirType::DT_LNK => FileType::Symlink,
            rusl::platform::DirType::DT_SOCK => FileType::Socket,
            _ => FileType::Unknown,
        }
    }

    /// Gets the utf8 filename of the entity
    /// # Errors
    /// The file name is not null terminated, or that null terminated name is not utf8
    pub fn file_name(&self) -> Result<&str> {
        let len = buf_strlen(&self.inner.d_name)?;
        // Safety:
        // We just checked the len
        let as_str = unsafe { core::str::from_utf8(self.inner.d_name.get_unchecked(..len)) }
            .map_err(|_| Error::no_code("File name not utf8"))?;
        Ok(as_str)
    }

    /// Gets the unixstr file name of the entity
    /// # Errors
    /// The file name is not null terminated
    pub fn file_unix_name(&self) -> Result<&UnixStr> {
        let len = buf_strlen(&self.inner.d_name)?;
        unsafe {
            // Safety: The non-null terminated len is one less than the null terminated len
            // ie. we just did a range check.
            let tgt = self.inner.d_name.get_unchecked(..=len);
            // Safety: `&UnixStr` and `&[u8]` have the same layout
            Ok(&*(core::ptr::from_ref::<[u8]>(tgt) as *const UnixStr))
        }
    }

    /// Opens this entity's file in read only mode
    /// # Errors
    /// This entity is not a file, check `file_type(&self)` first to be sure
    /// Os errors relating to permissions
    #[inline]
    pub fn open_file(&self) -> Result<File> {
        let ft = self.file_type();
        if ft == FileType::RegularFile || ft == FileType::CharDevice {
            Ok(File::open_at(self.fd.fd, self.file_unix_name()?)?)
        } else {
            Err(Error::no_code("Tried to open non-file as file"))
        }
    }

    /// Opens this entity's directory
    /// # Errors
    /// The entity is nota directory, check `file_type(&self)` first to be sure
    /// Os errors relating to permissions
    #[inline]
    pub fn open_dir(&self) -> Result<Directory> {
        if self.file_type() == FileType::Directory {
            Ok(Directory::open_at(self.fd.fd, self.file_unix_name()?)?)
        } else {
            Err(Error::no_code("Tried to open non-file as file"))
        }
    }
}

/// Tries to remove a directory from the filesystem at the specified path
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn remove_dir(path: &UnixStr) -> Result<()> {
    rusl::unistd::unlink_flags(path, UnlinkFlags::at_removedir())?;
    Ok(())
}

/// Tries to recursively remove a directory and its contents from the filesystem at the specified path.
/// Potentially very destructive
/// # Errors
/// Os errors relating to file access/permissions
pub fn remove_dir_all(path: &UnixStr) -> Result<()> {
    let dir = Directory::open(path)?;
    dir.remove_all()?;
    remove_dir(path)
}

impl Write for File {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(rusl::unistd::write(self.0 .0, buf)?)
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    flags: OpenFlags,
    mode: Mode,
}

impl Default for OpenOptions {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl OpenOptions {
    #[must_use]
    pub fn new() -> OpenOptions {
        OpenOptions {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            flags: OpenFlags::empty(),
            mode: Mode::from(0o0_000_666),
        }
    }

    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    pub fn custom_flags(&mut self, flags: OpenFlags) -> &mut Self {
        self.flags = flags;
        self
    }
    pub fn mode(&mut self, mode: Mode) -> &mut Self {
        self.mode = mode;
        self
    }
    /// Opens a file with `self` as provided options
    /// # Errors
    /// See `File::open_with_options`
    #[inline]
    pub fn open(&self, path: &UnixStr) -> Result<File> {
        File::open_with_options(path, self)
    }

    fn get_access_mode(&self) -> Result<OpenFlags> {
        match (self.read, self.write, self.append) {
            (true, false, false) => Ok(OpenFlags::O_RDONLY),
            (false, true, false) => Ok(OpenFlags::O_WRONLY),
            (true, true, false) => Ok(OpenFlags::O_RDWR),
            (false, _, true) => Ok(OpenFlags::O_WRONLY | OpenFlags::O_APPEND),
            (true, _, true) => Ok(OpenFlags::O_RDWR | OpenFlags::O_APPEND),
            (false, false, false) => Err(Error::no_code(
                "Bad OpenOptions, no access mode (read, write, append)",
            )),
        }
    }

    fn get_creation_mode(&self) -> crate::error::Result<OpenFlags> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(Error::no_code("Bad OpenOptions, used truncate, create, or create_new without access mode write or append"));
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(Error::no_code(
                        "Bad OpenOptions, used truncate without create_new with access mode append",
                    ));
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => OpenFlags::empty(),
            (true, false, false) => OpenFlags::O_CREAT,
            (false, true, false) => OpenFlags::O_TRUNC,
            (true, true, false) => OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
            (_, _, true) => OpenFlags::O_CREAT | OpenFlags::O_EXCL,
        })
    }
}
