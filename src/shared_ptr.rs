use libc::c_void;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ops::DerefMut;

#[repr(C)]
#[derive(Default)]

pub struct shared_ptr<T: ?Sized>([usize; 2], PhantomData<T>);
pub fn new<T>(value: T) -> shared_ptr<T> {
    extern "C" fn deleter<T>(ptr: *mut c_void) {
        drop(unsafe { Box::from_raw(ptr as *mut T) });
    }
    let ptr = Box::into_raw(Box::new(value)) as *mut c_void;
    let mut out = MaybeUninit::uninit();
    let out_ptr = out.as_mut_ptr() as *mut shared_ptr<()>;
    unsafe {
        shared_ptr__construct(ptr, deleter::<T>, out_ptr);
        out.assume_init()
    }
}

impl<T> shared_ptr<T> {
    fn raw(&self) -> *const shared_ptr<()> {
        self as *const shared_ptr<T> as *const shared_ptr<()>
    }
}

impl<T> Clone for shared_ptr<T> {
    fn clone(&self) -> Self {
        let mut out = MaybeUninit::uninit();
        let out_ptr = out.as_mut_ptr() as *mut shared_ptr<()>;
        unsafe {
            shared_ptr__copy(self.raw(), out_ptr);
            out.assume_init()
        }
    }
}

impl<T: ?Sized> Drop for shared_ptr<T> {
    fn drop(&mut self) {
        let raw = self as *mut shared_ptr<T> as *mut shared_ptr<()>;
        unsafe { shared_ptr__destruct(raw) }
    }
}

impl<T> Deref for shared_ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(shared_ptr__get(self.raw()) as *const T) }
    }
}

impl<T> DerefMut for shared_ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(shared_ptr__get(self.raw()) as *mut T) }
    }
}

extern "C" {
    fn shared_ptr__construct(
        ptr: *mut c_void,
        deleter: extern "C" fn(*mut c_void),
        out: *mut shared_ptr<()>,
    );
    fn shared_ptr__destruct(sp: *mut shared_ptr<()>);
    fn shared_ptr__copy(sp: *const shared_ptr<()>, out: *mut shared_ptr<()>);
    fn shared_ptr__get(sp: *const shared_ptr<()>) -> *mut ();
}

#[cfg(test)]
mod test {
    #[test]
    fn test_drop() {
        struct S<'a>(&'a mut &'static str);

        impl Drop for S<'_> {
            fn drop(&mut self) {
                *self.0 = "yes";
            }
        }

        let mut s = "no";
        let t = super::new(S(&mut s));

        assert_eq!(*(*t).0, "no");
        drop(t);
        assert_eq!(s, "yes");
    }

    #[test]
    fn test_clone() {
        struct S(&'static str);

        let a = super::new(S("a"));
        let mut b = a.clone();

        assert_eq!((*a).0, "a");
        assert_eq!((*b).0, "a");

        (*b).0 = "b";

        assert_eq!((*a).0, "b");
        assert_eq!((*b).0, "b");
    }

    #[test]
    fn test_into() {
        trait T {
            fn m(&mut self);
        }

        struct S(u32);

        impl T for S {
            fn m(&mut self) {
                self.0 += 1;
            }
        }

        let mut s = super::new(S(0));
        s.m();
        assert_eq!((*s).0, 1);

        let b = Box::new(S(42));
        let t = b as Box<dyn T>;

        let b = super::new(S(42));
        let t = b as super::shared_ptr<dyn T>; // FIXME
    }
}
