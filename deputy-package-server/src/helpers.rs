pub trait CeilingDiv
where
    Self: Sized,
{
    fn ceiling_div(&self, divisor: u64) -> Self;
}
impl CeilingDiv for u64 {
    fn ceiling_div(&self, divisor: u64) -> Self {
        self / divisor + (self % divisor != 0) as u64
    }
}
