use std::io::Write;

use tiny_bench::*;

use tiny_std::io::Write as TinyWrite;

pub fn main() {
    let num_files = 4;
    let tiny_std_tgt = "test-files/fs/bench-tiny-std-fs\0";
    if tiny_std::fs::metadata(tiny_std_tgt).is_err() {
        tiny_std::fs::create_dir(tiny_std_tgt).unwrap();
    }
    bench_with_setup_labeled(
        "tiny-std fs create remove files",
        || {
            tiny_std::fs::remove_dir_all(tiny_std_tgt).unwrap();
            tiny_std::fs::create_dir(tiny_std_tgt).unwrap();
        },
        |_| {
            for i in 0..num_files {
                let path = format!("test-files/fs/bench-tiny-std-fs/test-file{i}.txt\0");
                let mut file = tiny_std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .unwrap();
                file.write_all(b"Hello!").unwrap();
            }
        },
    );
    let std_tgt = "test-files/fs/bench-std-fs";
    if std::fs::metadata(std_tgt).is_err() {
        std::fs::create_dir(std_tgt).unwrap();
    }
    bench_with_setup_labeled(
        "std fs create remove files",
        || {
            std::fs::remove_dir_all(std_tgt).unwrap();
            std::fs::create_dir(std_tgt).unwrap();
        },
        |_| {
            for i in 0..num_files {
                let path = format!("{}/test-file{i}.txt", std_tgt);
                let mut file = std::fs::OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(path)
                    .unwrap();
                file.write_all(b"Hello!").unwrap();
            }
        },
    );
}
