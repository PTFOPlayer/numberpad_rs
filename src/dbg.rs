pub fn dbg_t(s: String) {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    println!("[{:?}]::{}", t, s);
}
