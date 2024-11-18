#[inline]
// f64 类型相加
pub fn float64add(x1: f64, x2: f64) -> f64 {
    let m = 100000.0;
    let x = (x1 * m) + (x2 * m);
    f64::round(x) / m
}
