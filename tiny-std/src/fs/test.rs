#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use rusl::compat::unix_str::UnixStr;
use rusl::{EEXIST, ENOENT};

use crate::fs::{metadata, File, FileType, OpenOptions};
use crate::io::{Read, Write};

const TEST_1_CONTENT: &str = "Hello world!
";

#[test]
fn can_read_null_term_path() {
    let path = "test-files/fs/test1.txt\0";
    can_read_using_file_at_path(path);
}

fn can_read_using_file_at_path(path: &str) {
    let mut file = File::open(path).unwrap();
    let mut buf = [0; 128];
    let content_len = file.read(&mut buf).unwrap();
    let get_slice = &buf[..content_len];
    assert_eq!(TEST_1_CONTENT.as_bytes(), get_slice);
}

#[test]
#[cfg(feature = "alloc")]
fn can_read_to_vec() {
    let path = "test-files/fs/test1.txt";
    let mut opts = crate::fs::OpenOptions::new();
    opts.read(true);
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    assert_eq!(TEST_1_CONTENT.as_bytes(), buf.as_slice());
    let other_buf = crate::fs::read(path).unwrap();
    assert_eq!(TEST_1_CONTENT.as_bytes(), other_buf.as_slice());
}

#[test]
#[cfg(feature = "alloc")]
fn can_read_to_string() {
    let path = "test-files/fs/test1.txt";
    let mut opts = crate::fs::OpenOptions::new();
    opts.read(true);
    let mut file = File::open(path).unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    assert_eq!(TEST_1_CONTENT.as_bytes(), buf.as_bytes());
    let other_buf = crate::fs::read_to_string(path).unwrap();
    assert_eq!(TEST_1_CONTENT.as_bytes(), other_buf.as_bytes());
}

#[test]
fn can_stat() {
    let dir = "";
    let dir_meta = metadata(dir).unwrap();
    assert!(dir_meta.is_dir());
    assert!(!dir_meta.is_symlink());
    assert!(!dir_meta.is_file());
    let path = "src/fs/test.rs\0";
    let file_meta = metadata(path).unwrap();
    assert!(!file_meta.is_dir());
    assert!(!file_meta.is_symlink());
    assert!(file_meta.is_file());
}

#[test]
fn can_create_read_and_write_file() {
    let tgt = "test-files/fs/test_create_read1.txt\0";
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tgt)
        .unwrap();
    let write_bytes = b"Hello created file!";
    let written = file.write(write_bytes).unwrap();
    assert_eq!(written, write_bytes.len());
    assert_eq!(19, write_bytes.len());
    drop(file);
    let mut file = OpenOptions::new().read(true).open(tgt).unwrap();
    let mut buf = [0u8; 19];
    let read_bytes = file.read(&mut buf).unwrap();
    assert_eq!(read_bytes, buf.len());
    assert_eq!(&buf, write_bytes);
}

#[test]
fn can_create_and_delete_file() {
    let tgt = "test-files/fs/test_create_delete1.txt\0";
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tgt)
        .unwrap();
    let file_meta = metadata(tgt).unwrap();
    assert!(file_meta.is_file());
    drop(file);
    crate::fs::remove_file(tgt).unwrap();
    match metadata(tgt) {
        Ok(_) => panic!("Found deleted file!"),
        Err(e) => {
            assert!(e.matches_errno(ENOENT));
        }
    }
}

#[test]
fn can_create_and_delete_dir() {
    let tgt = "test-files/fs/dir-test\0";
    if metadata(tgt).is_ok() {
        crate::fs::remove_dir(tgt).unwrap();
    }
    crate::fs::create_dir(tgt).unwrap();
    let dir_meta = metadata(tgt).unwrap();
    assert!(dir_meta.is_dir());
    match crate::fs::create_dir(tgt) {
        Ok(_) => panic!("Could create on already existing dir"),
        Err(e) => {
            assert!(e.matches_errno(EEXIST));
        }
    }
    crate::fs::remove_dir(tgt).unwrap();
    assert!(metadata(tgt).is_err())
}

#[test]
fn can_open_and_read_dir() {
    let tgt = "test-files/fs/dir-test1\0";
    let dir = crate::fs::Directory::open(tgt).unwrap();
    let it = dir.read();
    for entry in it {
        let entry = entry.unwrap();
        match entry.file_name().unwrap() {
            "." => assert_eq!(FileType::Directory, entry.file_type()),
            ".." => assert_eq!(FileType::Directory, entry.file_type()),
            "dummy_file1.txt" => assert_eq!(FileType::RegularFile, entry.file_type()),
            "dummy_file2.txt" => assert_eq!(FileType::RegularFile, entry.file_type()),
            "dummy_dir" => assert_eq!(FileType::Directory, entry.file_type()),
            n => panic!("Unexpected entry found {n}"),
        }
        unsafe {
            let unix_name = entry.file_unix_name().unwrap();
            if unix_name == UnixStr::from_str_unchecked(".\0") {
                assert_eq!(FileType::Directory, entry.file_type());
            } else if unix_name == UnixStr::from_str_unchecked("..\0") {
                assert_eq!(FileType::Directory, entry.file_type());
            } else if unix_name == UnixStr::from_str_unchecked("dummy_file1.txt\0") {
                assert_eq!(FileType::RegularFile, entry.file_type());
            } else if unix_name == UnixStr::from_str_unchecked("dummy_file2.txt\0") {
                assert_eq!(FileType::RegularFile, entry.file_type());
            } else if unix_name == UnixStr::from_str_unchecked("dummy_dir\0") {
                assert_eq!(FileType::Directory, entry.file_type());
            } else {
                panic!("Unexpected entry found")
            }
        }
    }

    drop(dir);
}

#[test]
fn create_read_and_delete_dir_with_a_lot_of_files() {
    let tgt = "test-files/fs/dir-test2\0";
    if metadata(tgt).is_ok() {
        crate::fs::remove_dir_all(tgt).unwrap();
    } else {
        crate::fs::create_dir(tgt).unwrap();
    }

    let create_files = 512;
    for i in 0..create_files {
        let owned_path = format!("test-files/fs/dir-test2/test-file{}.txt\0", i);
        let path = owned_path.as_str();
        let mut f = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(path)
            .unwrap();
        f.write(format!("Test write {i}").as_bytes()).unwrap();
    }
    let dir = crate::fs::Directory::open(tgt).unwrap();
    let it = dir.read();
    let mut found_entries = 0;
    for entry in it {
        let entry = entry.unwrap();
        let null_term = entry.file_unix_name().unwrap();
        if null_term != unsafe { UnixStr::from_str_unchecked(".\0") }
            && null_term != unsafe { UnixStr::from_str_unchecked("..\0") }
        {
            assert_eq!(FileType::RegularFile, entry.file_type());
            let file_name = entry.file_name().unwrap();
            let mut num = String::new();
            for ch in file_name.chars() {
                if ch.is_numeric() {
                    num.push(ch);
                }
            }
            let expect_num = num.parse::<i32>().unwrap();
            let mut buf = [0u8; 256];
            let expect_write = format!("Test write {}", expect_num);
            let expect_write_bytes = expect_write.as_bytes();
            let mut file = entry.open_file().unwrap();
            file.read(&mut buf).unwrap();
            assert_eq!(expect_write_bytes, &buf[..expect_write_bytes.len()]);
            found_entries += 1;
        }
    }
    assert_eq!(512, found_entries);
    drop(dir);
    crate::fs::remove_dir_all(tgt).unwrap();
    assert!(metadata(tgt).is_err());
}

#[test]
fn can_create_remove_dir_all() {
    let base = "test-files/fs/dir-test3\0";
    if metadata(base).is_ok() {
        crate::fs::remove_dir_all(base).unwrap();
    }
    let sub_dirs = b"test-files/fs/dir-test3/dir0/dir1/dir2/dir3\0";
    let mut sub_dir_bytes = [0u8; 44];
    sub_dir_bytes.copy_from_slice(sub_dirs);
    crate::fs::create_dir_all(sub_dir_bytes.as_mut_slice()).unwrap();
    assert!(metadata(sub_dirs.as_slice()).is_ok());
    crate::fs::remove_dir_all(base).unwrap();
    assert!(metadata(base).is_err());
}
