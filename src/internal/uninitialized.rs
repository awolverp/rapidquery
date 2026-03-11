//! Helpers for creating __init__ methods for types.

use ::std::cell::UnsafeCell;
use ::std::sync::Arc;

/// An uninitialized value which can nominally be written to only once.
#[repr(transparent)]
pub struct ImmutableUninit<T>(UnsafeCell<Option<T>>);

/// An uninitialized Arc-Mutex.
#[repr(transparent)]
pub struct MutableUninit<T>(Arc<::parking_lot::Mutex<Option<T>>>);

impl<T> ImmutableUninit<T> {
    /// Creates a new uninitialized instance.
    #[inline]
    #[must_use]
    pub const fn uninit() -> Self {
        Self(UnsafeCell::new(None))
    }

    /// Creates a new initialized instance.
    #[inline]
    #[must_use]
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(Some(val)))
    }

    pub fn is_initialized(&self) -> bool {
        unsafe {
            let inner = &*self.0.get();
            inner.is_some()
        }
    }

    /// Initializes the self by `val`.
    ///
    /// # Safety
    /// - It is Undefined Behavior to call this while any other reference(s) to the wrapped value are alive.
    /// - Mutating the wrapped value through other means while the returned reference is alive is Undefined Behavior.
    #[inline]
    pub unsafe fn set(&self, val: T) {
        let _ = (*self.0.get()).replace(val);
    }

    /// Gets a reference to the wrapped value.
    #[inline]
    pub fn get_checked(&self) -> Option<&T> {
        unsafe {
            let inner = &*self.0.get();
            inner.as_ref()
        }
    }

    /// Unwraps the value, consuming the [`ImmutableUninit`].
    pub fn into_inner(self) -> Option<T> {
        self.0.into_inner()
    }
}

impl<T> AsRef<T> for ImmutableUninit<T> {
    /// Gets a reference to the wrapped value.
    ///
    /// # Panics
    /// Panics if you forgot to call [`ImmutableUninit::set()`]
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe {
            let inner = &*self.0.get();

            inner.as_ref().expect(
                "Object did not initialized yet. This happens when you forget to call __init__ \
                 method. This is a critical issue.",
            )
        }
    }
}
impl<T> Default for ImmutableUninit<T> {
    /// Creates a new uninitialized instance, like [`ImmutableUninit::uninit`]
    #[inline]
    fn default() -> Self {
        Self::uninit()
    }
}
impl<T: Clone> Clone for ImmutableUninit<T> {
    fn clone(&self) -> Self {
        let res = Self::uninit();

        if let Some(x) = self.get_checked() {
            unsafe {
                res.set(x.clone());
            }
        }

        res
    }
}
impl<T: ::std::fmt::Debug> ::std::fmt::Debug for ImmutableUninit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("ImmutableUninit");

        match self.get_checked() {
            Some(x) => tuple.field(&x).finish(),
            None => tuple.field(&"<uninit>").finish(),
        }
    }
}
impl<T> From<T> for ImmutableUninit<T> {
    fn from(value: T) -> Self {
        ImmutableUninit::new(value)
    }
}
unsafe impl<T> Send for ImmutableUninit<T> {}
unsafe impl<T> Sync for ImmutableUninit<T> {}

impl<T> MutableUninit<T> {
    /// Creates a new uninitialized instance.
    #[inline]
    #[must_use]
    pub fn uninit() -> Self {
        Self(Arc::new(::parking_lot::Mutex::new(None)))
    }

    /// Creates a new initialized instance.
    #[inline]
    #[must_use]
    pub fn new(val: T) -> Self {
        Self(Arc::new(::parking_lot::Mutex::new(Some(val))))
    }

    pub fn is_initialized(&self) -> bool {
        self.0.lock().is_some()
    }

    /// Initializes the self by `val`.
    #[inline]
    pub fn set(&self, val: T) {
        let _ = (*self.0.lock()).replace(val);
    }

    /// Gets a reference to the wrapped value.
    ///
    /// # Panics
    /// Panics if you forgot to call [`ImmutableUninit::set()`]
    pub fn lock(&self) -> parking_lot::MappedMutexGuard<'_, T> {
        let inner = self.0.lock();
        assert!(
            inner.as_ref().is_some(),
            "Object did not initialized yet. This happens when you forget to call __init__ \
             method. This is a critical issue.",
        );

        unsafe { parking_lot::MutexGuard::map(inner, |x| x.as_mut().unwrap_unchecked()) }
    }

    /// Unwraps the value, consuming the [`MutableInit`].
    pub fn into_inner(self) -> Arc<::parking_lot::Mutex<Option<T>>> {
        self.0
    }
}

impl<T> Default for MutableUninit<T> {
    /// Creates a new uninitialized instance, like [`MutableUninit::uninit`]
    #[inline]
    fn default() -> Self {
        Self::uninit()
    }
}
impl<T> Clone for MutableUninit<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: ::std::fmt::Debug> ::std::fmt::Debug for MutableUninit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tuple = f.debug_tuple("MutableUninit");

        match &*self.0.lock() {
            Some(x) => tuple.field(&x).finish(),
            None => tuple.field(&"<uninit>").finish(),
        }
    }
}
impl<T> From<T> for MutableUninit<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}
