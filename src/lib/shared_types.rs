pub(crate) type ClientId = u16;
pub(crate) type TxId = u32;
pub(crate) type AmountValue = i64;
pub(crate) type RawAmountValue = f64;

#[derive(Debug)]
pub(crate) struct Amount {
    pub(crate) value: AmountValue,
}
impl Amount {
    pub(crate) fn new() -> Self {
        Self { value: 0 }
    }
}