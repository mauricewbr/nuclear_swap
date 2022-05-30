library math_lib;

pub fn abs(x: u64, y: u64) -> u64 {
    let mut res: u64 = 0;
    if x >= y {
        res = x - y;
    } else {
        res = y - x;
    }
    res
}