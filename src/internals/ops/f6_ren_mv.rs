use crate::internals::ops::f5_cpy::cpy_mv_helper;
pub fn ren_mv(siv: &mut cursive::Cursive) {
    cpy_mv_helper(siv, false);
}
