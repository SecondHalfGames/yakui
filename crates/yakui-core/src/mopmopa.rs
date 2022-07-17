//! My Own Personal My Own Personal Any
//!
//! Implementation taken from Rust 1.61.0's `std::any::Any`.

macro_rules! mopmopafy {
    ($target:ident) => {
        #[allow(unused)]
        impl dyn $target {
            /// See `std::any::Any::is`
            #[inline]
            pub fn is<T: std::any::Any>(&self) -> bool {
                let t = TypeId::of::<T>();
                let concrete = self.type_id();
                t == concrete
            }

            /// See `std::any::Any::downcast_ref`.
            #[inline]
            pub fn downcast_ref<T: std::any::Any>(&self) -> Option<&T> {
                if self.is::<T>() {
                    unsafe { Some(self.downcast_ref_unchecked()) }
                } else {
                    None
                }
            }

            /// See `std::any::Any::downcast_mut`.
            #[inline]
            pub fn downcast_mut<T: std::any::Any>(&mut self) -> Option<&mut T> {
                if self.is::<T>() {
                    unsafe { Some(self.downcast_mut_unchecked()) }
                } else {
                    None
                }
            }

            #[inline]
            unsafe fn downcast_ref_unchecked<T: std::any::Any>(&self) -> &T {
                debug_assert!(self.is::<T>());
                &*(self as *const dyn $target as *const T)
            }

            #[inline]
            unsafe fn downcast_mut_unchecked<T: std::any::Any>(&mut self) -> &mut T {
                debug_assert!(self.is::<T>());
                &mut *(self as *mut dyn $target as *mut T)
            }
        }
    };
}
