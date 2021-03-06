#[allow(dead_code)]
pub fn rand_int(max: u32) -> u32 {
    use rand::Rng;
    rand::thread_rng().gen_range(0, max)
}
#[allow(dead_code)]
pub fn rand_usize(max: usize) -> usize {
    use rand::Rng;
    rand::thread_rng().gen_range(0, max)
}
#[allow(dead_code)]
pub fn rand_intr(min: i32, max: i32) -> i32 {
    use rand::Rng;
    rand::thread_rng().gen_range(min, max)
}
#[allow(dead_code)]
pub fn uid() -> usize {
    use rand::Rng;
    rand::thread_rng().gen_range(100000, 999999)
}
#[allow(dead_code)]
pub fn rand_float(min: f32, max: f32) -> f32 {
    use rand::Rng;
    rand::thread_rng().gen_range(min, max)
}
#[allow(dead_code)]
pub fn pick(values: &[usize]) -> usize {
    use rand::Rng;
    values[rand::thread_rng().gen_range(0, values.len())]
}
#[allow(dead_code)]
pub fn maybe(pty: f32) -> bool{
    rand_float(0., 1.) < pty
}

#[allow(dead_code)]
pub fn now() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ste = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
    ste.as_secs() as f64 + ste.subsec_micros() as f64 / 1_000_000.0
}

#[allow(dead_code)]
pub fn same_col(a: &(f32, f32, f32), b: &(f32, f32, f32)) -> bool {
    a.0 == b.0 && a.1 == b.1 && a.2 == b.2
}

#[allow(dead_code)]
pub fn sleep(millis: u64) {
    let ten_millis = std::time::Duration::from_millis(millis);
    std::thread::sleep(ten_millis);
}

pub fn rand_color() -> (f32, f32, f32) {
    (rand_float(0.7, 1.0), rand_float(0.7, 1.0), rand_float(0.4, 1.0))
}
