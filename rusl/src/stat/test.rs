use super::*;

#[test]
fn stat_test() {
    #[cfg(feature = "alloc")]
        let (a, b) = { ("test-files/can_stat.txt", "") };
    #[cfg(not(feature = "alloc"))]
        let (a, b) = { ("test-files/can_stat.txt\0", "\0") };
    do_stat_cwd(a, b);
}

fn do_stat_cwd(cwd_path: &str, empty_path: &str) {
    stat(cwd_path).unwrap();
    stat(empty_path).unwrap();
    stat(()).unwrap();
}