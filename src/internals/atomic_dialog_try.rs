use cursive::view::{IntoBoxedView, View, ViewWrapper};
use cursive::views::Dialog;
pub struct AtomicDialog<T: View> {
    view: T,
    inner: Dialog,
}
impl<T> AtomicDialog<T>
where
    T: View,
{
    /* pub fn around<V: IntoBoxedView>(view: V) -> Self {
        Dialog::around(V) as AtomicDialog<T>
    }*/
}
#[macro_export]
macro_rules! wrap_impl {
    (self.$v:ident: $t:ty) => {
        type V = $t;

        fn with_view<F, R>(&self, f: F) -> ::std::option::Option<R>
        where
            F: ::std::ops::FnOnce(&Self::V) -> R,
        {
            ::std::option::Option::Some(f(&self.$v))
        }

        fn with_view_mut<F, R>(&mut self, f: F) -> ::std::option::Option<R>
        where
            F: ::std::ops::FnOnce(&mut Self::V) -> R,
        {
            ::std::option::Option::Some(f(&mut self.$v))
        }

        fn into_inner(self) -> ::std::result::Result<Self::V, Self>
        where
            Self::V: ::std::marker::Sized,
        {
            ::std::result::Result::Ok(self.$v)
        }
    };
}
impl<T: View> ViewWrapper for AtomicDialog<T> {
    cursive::wrap_impl!(self.view: T);
}
