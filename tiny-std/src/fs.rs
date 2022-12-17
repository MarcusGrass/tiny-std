#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::mem::MaybeUninit;

use rusl::error::Errno;
use rusl::platform::{Dirent, Mode, OpenFlags, Stat, AT_REMOVEDIR, NULL_BYTE};
use rusl::string::strlen::{buf_strlen, strlen};
use rusl::string::unix_str::{AsUnixStr, UnixStr};

use crate::error::Error;
use crate::error::Result;
use crate::io::{Read, Write};
use crate::unix::fd::{BorrowedFd, OwnedFd, RawFd};

#[cfg(test)]
mod test;

pub struct File(OwnedFd);

impl File {
    /// Opens a file with default options
    /// # Errors
    /// Operating system errors ond finding and reading files
    #[inline]
    pub fn open(path: impl AsUnixStr) -> Result<Self> {
        Self::open_with_options(path, OpenOptions::new().read(true))
    }

    #[inline]
    fn open_with_options(path: impl AsUnixStr, opts: &OpenOptions) -> Result<Self> {
        let flags =
            OpenFlags::O_CLOEXEC | opts.get_access_mode()? | opts.get_creation_mode()? | opts.flags;
        let fd = rusl::unistd::open_mode(path, flags, opts.mode)?;
        Ok(File(OwnedFd(fd)))
    }

    fn open_at(dir_fd: RawFd, path: impl AsUnixStr) -> Result<Self> {
        let mut opts = OpenOptions::new();
        opts.read(true);
        let flags =
            OpenFlags::O_CLOEXEC | opts.get_access_mode()? | opts.get_creation_mode()? | opts.flags;
        let fd = rusl::unistd::open_at_mode(dir_fd, path, flags, opts.mode)?;
        Ok(File(OwnedFd(fd)))
    }
}

impl File {
    #[inline]
    pub(crate) fn into_inner(self) -> OwnedFd {
        self.0
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
pub fn read<P: AsUnixStr>(path: P) -> Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Reads a file into a newly allocated vec
/// # Errors
/// Os errors relating to file access and reading as well as utf8 conversion errors
#[cfg(feature = "alloc")]
pub fn read_to_string<P: AsUnixStr>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
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
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.0.st_size as u64
    }
}

/// Reads metadata at path
/// # Errors
/// Os errors relating to file access
#[inline]
pub fn metadata<P: AsUnixStr>(path: P) -> Result<Metadata> {
    let res = rusl::unistd::stat(path)?;
    Ok(Metadata(res))
}

/// Tries to remove a file from the filesystem at the specified path
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn remove_file<P: AsUnixStr>(path: P) -> Result<()> {
    rusl::unistd::unlink(path)?;
    Ok(())
}

/// Tries to create a directory at the specified path
/// # Errors
/// OS errors relating to file access/permissions
#[inline]
pub fn create_dir<P: AsUnixStr>(path: P) -> Result<()> {
    rusl::unistd::mkdir(path, Mode::from(0o755))?;
    Ok(())
}

/// Tries to create a directory at the specified path
/// # Errors
/// OS errors relating to file access/permissions
/// If working without an allocator, the maximum path length is 512 bytes
#[inline]
pub fn create_dir_all<P: AsUnixStr>(path: P) -> Result<()> {
    // To make it simple we'll just stack alloc an uninit 512 array for the path.
    // Kind of a travesty, but needs to be like this to work without an allocator
    const NO_ALLOC_MAX_LEN: usize = 512;
    const EMPTY: [MaybeUninit<u8>; NO_ALLOC_MAX_LEN] = [MaybeUninit::uninit(); NO_ALLOC_MAX_LEN];
    path.exec_with_self_as_ptr(|ptr| unsafe {
        // Could check if we haven't got any slashes at all and just run the pointer straight through
        // without possibly having to add an extra indirection buffer here.
        let len = strlen(ptr);
        #[cfg(feature = "alloc")]
        if len > NO_ALLOC_MAX_LEN {
            let mut owned: Vec<u8> = Vec::with_capacity(len);
            ptr.copy_to(owned.as_mut_ptr(), len);
            return write_all_sub_paths(owned.as_mut_slice(), ptr);
        }
        #[cfg(not(feature = "alloc"))]
        if len > NO_ALLOC_MAX_LEN {
            return Err(rusl::Error::no_code(
                "Supplied path larger than 512 without an allocator present",
            ));
        }
        let mut copied = EMPTY;
        ptr.copy_to(copied.as_mut_ptr().cast(), len);
        // Make into slice, we know the actual slice-length is len + 1
        let initialized_section = copied[..len].as_mut_ptr().cast();
        let buf: &mut [u8] = core::slice::from_raw_parts_mut(initialized_section, len);
        write_all_sub_paths(buf, ptr)
    })?;
    Ok(())
}

#[inline]
fn write_all_sub_paths(buf: &mut [u8], raw: *const u8) -> core::result::Result<(), rusl::Error> {
    let len = buf.len();
    let mut it = 1;
    loop {
        // Iterate down
        let ind = len - it;
        if ind == 0 {
            break;
        }
        // Todo, actually make sure we restore
        let byte = buf[ind];
        if byte == b'/' {
            // Swap slash for null termination to make a valid path
            buf[ind] = NULL_BYTE;
            return match rusl::unistd::mkdir(&buf[..=ind], Mode::from(0o755)) {
                // Successfully wrote, traverse down
                Ok(_) => {
                    // Replace the null byte to make a valid path concatenation
                    buf[ind] = b'/';
                    for i in ind + 1..len {
                        // Found next
                        if buf[i] == b'/' {
                            // Swap slash for null termination to make a valid path
                            buf[i] = NULL_BYTE;
                            rusl::unistd::mkdir(&buf[..=i], Mode::from(0o755))?;
                            // Swap back to continue down
                            buf[i] = b'/';
                        }
                    }
                    // We know the actual length is len + 1 and null terminated, try write full
                    rusl::unistd::mkdir(
                        unsafe { core::slice::from_raw_parts(raw, len + 1) },
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
    pub fn open<P: AsUnixStr>(path: P) -> Result<Directory> {
        let fd = rusl::unistd::open(path, OpenFlags::O_CLOEXEC | OpenFlags::O_RDONLY)?;
        Ok(Directory(OwnedFd(fd)))
    }

    #[inline]
    fn open_at<P: AsUnixStr>(dir_fd: RawFd, path: P) -> Result<Directory> {
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
                rusl::unistd::unlink_at(self.0 .0, fname, AT_REMOVEDIR)?;
            } else {
                rusl::unistd::unlink_at(self.0 .0, sub_dir.file_unix_name()?, 0)?;
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
            Ok(core::mem::transmute(tgt))
        }
    }

    /// Opens this entity's file in read only mode
    /// # Errors
    /// This entity is not a file, check `file_type(&self)` first to be sure
    /// Os errors relating to permissions
    #[inline]
    pub fn open_file(&self) -> Result<File> {
        if self.file_type() == FileType::RegularFile {
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
pub fn remove_dir<P: AsUnixStr>(path: P) -> Result<()> {
    rusl::unistd::unlink_flags(path, AT_REMOVEDIR)?;
    Ok(())
}

/// Tries to recursively remove a directory and its contents from the filesystem at the specified path.
/// Potentially very destructive
/// # Errors
/// Os errors relating to file access/permissions
pub fn remove_dir_all<P: AsUnixStr>(path: P) -> Result<()> {
    let dir = Directory::open(&path)?;
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
            mode: Mode::from(0o0000666),
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
    pub fn open(&self, path: impl AsUnixStr) -> Result<File> {
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
