#![no_std]

use futures_core::Stream;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(all(not(feature = "send"), feature = "alloc"))]
pub type Hrc<T> = alloc::rc::Rc<T>;

#[cfg(all(feature = "send", feature = "alloc"))]
pub type Hrc<T> = alloc::sync::Arc<T>;

#[derive(Debug)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Lock could not be acquired")
    }
}

impl core::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;

pub trait Locklet<T> {
    type Read<'a>
    where
        Self: 'a;

    type Write<'a>
    where
        Self: 'a;

    fn try_read_lock<'a>(&'a self) -> Result<Self::Read<'a>>;

    fn try_write_lock<'a>(&'a self) -> Result<Self::Write<'a>>;

    fn read_lock<'a>(&'a self) -> Self::Read<'a>;

    fn write_lock<'a>(&'a self) -> Self::Write<'a>;
}
#[cfg(not(feature = "send"))]
impl<T> Locklet<T> for core::cell::RefCell<T>
where
    T: 'static,
{
    type Read<'a> = core::cell::Ref<'a, T>;
    type Write<'a> = core::cell::RefMut<'a, T>;

    fn try_read_lock<'a>(&'a self) -> Result<Self::Read<'a>> {
        self.try_borrow().map_err(|_| Error)
    }

    fn try_write_lock<'a>(&'a self) -> Result<Self::Write<'a>> {
        self.try_borrow_mut().map_err(|_| Error)
    }

    fn read_lock<'a>(&'a self) -> Self::Read<'a> {
        self.borrow()
    }

    fn write_lock<'a>(&'a self) -> Self::Write<'a> {
        self.borrow_mut()
    }
}

#[cfg(all(feature = "std", feature = "send", feature = "parking-lot"))]
impl<T> Locklet<T> for parking_lot::RwLock<T>
where
    T: 'static,
{
    type Read<'a> = parking_lot::RwLockReadGuard<'a, T>;
    type Write<'a> = parking_lot::RwLockWriteGuard<'a, T>;

    fn try_read_lock<'a>(&'a self) -> Result<Self::Read<'a>> {
        Ok(self.read())
    }

    fn try_write_lock<'a>(&'a self) -> Result<Self::Write<'a>> {
        Ok(self.write())
    }

    fn read_lock<'a>(&'a self) -> Self::Read<'a> {
        self.read()
    }

    fn write_lock<'a>(&'a self) -> Self::Write<'a> {
        self.write()
    }
}

#[cfg(all(feature = "std", feature = "send"))]
impl<T> Locklet<T> for std::sync::RwLock<T>
where
    T: 'static,
{
    type Read<'a> = std::sync::RwLockReadGuard<'a, T>;
    type Write<'a> = std::sync::RwLockWriteGuard<'a, T>;

    fn try_read_lock<'a>(&'a self) -> Result<Self::Read<'a>> {
        self.read().map_err(|_| Error)
    }

    fn try_write_lock<'a>(&'a self) -> Result<Self::Write<'a>> {
        self.write().map_err(|_| Error)
    }

    fn read_lock<'a>(&'a self) -> Self::Read<'a> {
        self.read().unwrap()
    }

    fn write_lock<'a>(&'a self) -> Self::Write<'a> {
        self.write().unwrap()
    }
}

#[cfg(all(not(feature = "std"), feature = "send"))]
impl<T> Locklet<T> for spin::RwLock<T>
where
    T: 'static,
{
    type Read<'a> = spin::RwLockReadGuard<'a, T>;
    type Write<'a> = spin::RwLockWriteGuard<'a, T>;

    fn try_read_lock<'a>(&'a self) -> Result<Self::Read<'a>> {
        Ok(self.read())
    }

    fn try_write_lock<'a>(&'a self) -> Result<Self::Write<'a>> {
        Ok(self.write())
    }

    fn read_lock<'a>(&'a self) -> Self::Read<'a> {
        self.read()
    }

    fn write_lock<'a>(&'a self) -> Self::Write<'a> {
        self.write()
    }
}

#[cfg(not(feature = "send"))]
pub type Lock<T> = core::cell::RefCell<T>;

#[cfg(all(not(feature = "std"), feature = "send"))]
pub type Lock<T> = spin::RwLock<T>;

#[cfg(all(feature = "std", feature = "send", feature = "parking-lot"))]
pub type Lock<T> = parking_lot::RwLock<T>;

#[cfg(all(feature = "std", feature = "send", not(feature = "parking-lot")))]
pub type Lock<T> = std::sync::RwLock<T>;

#[cfg(not(feature = "send"))]
pub trait HSend {}

#[cfg(not(feature = "send"))]
impl<T> HSend for T {}

#[cfg(not(feature = "send"))]
pub trait HSync {}

#[cfg(not(feature = "send"))]
impl<T> HSync for T {}

#[cfg(feature = "send")]
pub trait HSend: Send {}

#[cfg(feature = "send")]
impl<T> HSend for T where T: Send {}

#[cfg(feature = "send")]
pub trait HSync: Sync {}

#[cfg(feature = "send")]
impl<T> HSync for T where T: Sync {}

pub trait HSendSync: HSend + HSync {}

impl<T> HSendSync for T where T: HSend + HSync {}

#[cfg(all(feature = "alloc", not(feature = "send")))]
pub type HBoxFuture<'a, T> =
    core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = T> + 'a>>;

#[cfg(all(feature = "alloc", feature = "send"))]
pub type HBoxFuture<'a, T> =
    core::pin::Pin<alloc::boxed::Box<dyn core::future::Future<Output = T> + 'a + Send>>;

pub trait HFuture: Future + HSend {}

impl<T> HFuture for T where T: Future + HSend {}

pub trait HStream: Stream + HSend {}

impl<T> HStream for T where T: Stream + HSend {}

#[cfg(all(feature = "alloc", not(feature = "send")))]
pub type HBoxError<'a> = alloc::boxed::Box<dyn core::error::Error + 'a>;

#[cfg(all(feature = "alloc", feature = "send"))]
pub type HBoxError<'a> = alloc::boxed::Box<dyn core::error::Error + Send + Sync + 'a>;

#[cfg(all(feature = "alloc", not(feature = "send")))]
pub type HBoxAny<'a> = alloc::boxed::Box<dyn core::any::Any + 'a>;

#[cfg(all(feature = "alloc", feature = "send"))]
pub type HBoxAny<'a> = alloc::boxed::Box<dyn core::any::Any + Send + Sync + 'a>;
