fn main() {
    extern "C" {
        fn my_add(a: i32, b: i32) -> i32;
    }
    let val = unsafe { my_add(5, 6) };
    println!("{val}");
}
