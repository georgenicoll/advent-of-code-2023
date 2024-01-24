fn main() {
    let raw_p: *const u32 = &10;

    unsafe {
        println!("Value is {}", *raw_p);
    }
}
