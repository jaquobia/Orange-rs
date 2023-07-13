const NIBBLE_MASK: u8 = 0b00001111;

pub fn nibble_get(slice: &[u8], index: usize) -> u8 {
    (slice[index >> 1] >> (4 * (index & 1))) & NIBBLE_MASK
}

pub fn nibble_set(slice: &mut [u8], index: usize, value: u8) {
    let real_index = index >> 1;
    let shift_amount = (4 * (index & 1)) as u8;
    let value_as_nibble = (value & NIBBLE_MASK) << shift_amount;
    let other_nibble = slice[real_index] & (NIBBLE_MASK << (4 - shift_amount));
    slice[real_index] = other_nibble | value_as_nibble;
}
