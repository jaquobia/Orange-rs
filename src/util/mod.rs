use std::ffi::OsStr;

// use ultraviolet::IVec3;
pub mod pos;
pub mod workers;
pub mod frustrum;
pub mod nibble;

pub fn os_str_to_string(s: &OsStr) -> String {
    s.to_string_lossy().to_string()
}

// pub struct IteratorYZX(IVec3, IVec3, u32);
//
// impl IteratorYZX {
//     fn between(pos_a: IVec3, pos_b: IVec3) -> Self {
//         let difference = pos_b - pos_a;
//         let total_elements = difference.x * difference.y * difference.z;
//         Self(pos_a, pos_b, 0)
//     }
// }
//
// impl Iterator for IteratorYZX {
//     type Item = IVec3;
//     #[inline]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.2 += 1;
//         Some(IVec3::one())
//     }
// }
