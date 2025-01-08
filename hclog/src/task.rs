/*
 * this code is actually a boiled down copy of tokio::task::task_local/LocalKey. For further
 * questions please refere to:
 *      https://github.com/tokio-rs/tokio/tree/master/tokio/src/task/task_local.rs
 *
 * We can't use the tokio macro/type in this lib because its scope is only valid for the contained
 * F: Future. If called or setup from an external Future (e.g. via tokio::spawn) the task_local is
 * not in scope of the external Future. Instead provide the same but slightly modified interface to
 * provide task local logging/context.
 * Another positive side effect is that creating a copy removes the dependency to tokio because
 * this module is based on std modules (+pin_project_lite).
 *
 * Modifications made to the tokio module:
 *  -> Rework Error interface instead of throwing a panic
 *  -> Remove functions we do not need in our scope
 *  -> add try_with_mut to allow mutable access to the inner RefCell
 */
use pin_project_lite::pin_project;
use std::{
    cell::{RefCell, RefMut},
    future::{Future},
    task::{Context, Poll},
    pin::Pin,
    marker::PhantomPinned,
    error::Error,
    thread,
    fmt,
    mem,
};

#[macro_export]
#[doc(hidden)]
macro_rules! task_local {
     // empty (base case for the recursion)
    () => {};

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty; $($rest:tt)*) => {
        $crate::__task_local_inner!($(#[$attr])* $vis $name, $t);
        $crate::task_local!($($rest)*);
    };

    ($(#[$attr:meta])* $vis:vis static $name:ident: $t:ty) => {
        $crate::__task_local_inner!($(#[$attr])* $vis $name, $t);
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __task_local_inner {
    ($(#[$attr:meta])* $vis:vis $name:ident, $t:ty) => {
        $(#[$attr])*
        $vis static $name: $crate::task::LocalKey<$t> = {
            std::thread_local! {
                static __KEY: std::cell::RefCell<Option<$t>> = std::cell::RefCell::new(None);
            }

            $crate::task::LocalKey { inner: __KEY }
        };
    };
}


pub struct LocalKey<T: 'static> {
    #[doc(hidden)]
    pub inner: thread::LocalKey<RefCell<Option<T>>>,
}

impl<T: 'static> LocalKey<T> {
    pub fn scope<F>(&'static self, value: T, f: F) -> TaskLocalFuture<T, F>
    where
        F: Future,
    {
        TaskLocalFuture {
            local: self,
            slot: Some(value),
            future: Some(f),
            _pinned: PhantomPinned,
        }
    }

    fn scope_inner<F, R>(&'static self, slot: &mut Option<T>, f: F) -> Result<R, TaskLocalErr>
    where
        F: FnOnce() -> R,
    {
        struct Guard<'a, T: 'static> {
            local: &'static LocalKey<T>,
            slot: &'a mut Option<T>,
        }

        impl<'a, T: 'static> Drop for Guard<'a, T> {
            fn drop(&mut self) {
                // This should not panic.
                //
                // We know that the RefCell was not borrowed before the call to
                // `scope_inner`, so the only way for this to panic is if the
                // closure has created but not destroyed a RefCell guard.
                // However, we never give user-code access to the guards, so
                // there's no way for user-code to forget to destroy a guard.
                //
                // The call to `with` also should not panic, since the
                // thread-local wasn't destroyed when we first called
                // `scope_inner`, and it shouldn't have gotten destroyed since
                // then.
                self.local.inner.with(|inner| {
                    let mut ref_mut = inner.borrow_mut();
                    mem::swap(self.slot, &mut *ref_mut);
                });
            }
        }

        self.inner.try_with(|inner| {
            inner
                .try_borrow_mut()
                .map(|mut ref_mut| mem::swap(slot, &mut *ref_mut))
        })??;

        let guard = Guard { local: self, slot };

        let res = f();

        drop(guard);

        Ok(res)
    }

    #[allow(dead_code)]
    pub fn with<F, R>(&'static self, f: F) -> Result<R, TaskLocalErr>
    where
        F: FnOnce(&T) -> R,
    {
        match self.try_with(f) {
            Ok(res) => Ok(res),
            Err(_) => Err(TaskLocalErr::AccessError),
        }
    }

    pub fn try_with_mut<R, F>(&'static self, f: F) -> Result<R, TaskLocalErr>
    where
        F: FnOnce(&mut T) -> R,
    {
        let try_with_mut_res = self.inner.try_with(|v| {
            let mut borrow: RefMut<'_, Option<T>> = v.borrow_mut();
            borrow.as_mut().map(f)
        });

        match try_with_mut_res {
            Ok(Some(res)) => Ok(res),
            Ok(None) | Err(_) => Err(TaskLocalErr::AccessError),
        }
    }

    pub fn try_with<F, R>(&'static self, f: F) -> Result<R, TaskLocalErr>
    where
        F: FnOnce(&T) -> R,
    {
        // If called after the thread-local storing the task-local is destroyed,
        // then we are outside of a closure where the task-local is set.
        //
        // Therefore, it is correct to return an AccessError if `try_with`
        // returns an error.
        let try_with_res = self.inner.try_with(|v| {
            // This call to `borrow` cannot panic because no user-defined code
            // runs while a `borrow_mut` call is active.
            v.borrow().as_ref().map(f)
        });

        match try_with_res {
            Ok(Some(res)) => Ok(res),
            Ok(None) | Err(_) => Err(TaskLocalErr::AccessError),
        }
    }
}

impl<T: Copy + 'static> LocalKey<T> {
    #[allow(dead_code)]
    pub fn get(&'static self) -> Result<T, TaskLocalErr> {
        self.with(|v| *v)
    }
    #[allow(dead_code)]
    pub fn set(&'static self, t: T) -> Result<(), TaskLocalErr> {
        self.inner.try_with(|v| {
            *v.borrow_mut() = Some(t);
        })?;
        Ok(())
    }
}

impl<T: 'static> fmt::Debug for LocalKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad("LocalKey { .. }")
    }
}

pin_project! {
    pub struct TaskLocalFuture<T, F>
    where
        T: 'static,
    {
        local: &'static LocalKey<T>,
        slot: Option<T>,
        #[pin]
        future: Option<F>,
        #[pin]
        _pinned: PhantomPinned,
    }

    impl<T: 'static, F> PinnedDrop for TaskLocalFuture<T, F> {
        fn drop(this: Pin<&mut Self>) {
            let this = this.project();
            if mem::needs_drop::<F>() && this.future.is_some() {
                // Drop the future while the task-local is set, if possible. Otherwise
                // the future is dropped normally when the `Option<F>` field drops.
                let mut future = this.future;
                let _ = this.local.scope_inner(this.slot, || {
                    future.set(None);
                });
            }
        }
    }
}

impl<T: 'static, F: Future> Future for TaskLocalFuture<T, F> {
    type Output = F::Output;

    #[track_caller]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut future_opt = this.future;

        let res = this
            .local
            .scope_inner(this.slot, || match future_opt.as_mut().as_pin_mut() {
                Some(fut) => {
                    let res = fut.poll(cx);
                    if res.is_ready() {
                        future_opt.set(None);
                    }
                    Some(res)
                }
                None => None,
            });

        match res {
            Ok(Some(res)) => res,
            Ok(None) => panic!("`TaskLocalFuture` polled after completion"),
            Err(err) => err.panic(),
        }
    }
}

impl<T: 'static, F> fmt::Debug for TaskLocalFuture<T, F>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        /// Format the Option without Some.
        struct TransparentOption<'a, T> {
            value: &'a Option<T>,
        }
        impl<'a, T: fmt::Debug> fmt::Debug for TransparentOption<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self.value.as_ref() {
                    Some(value) => value.fmt(f),
                    // Hitting the None branch should not be possible.
                    None => f.pad("<missing>"),
                }
            }
        }

        f.debug_struct("TaskLocalFuture")
            .field("value", &TransparentOption { value: &self.slot })
            .finish()
    }
}

#[derive(PartialEq, Eq, Ord, PartialOrd)]
pub enum TaskLocalErr {
    BorrowError,
    AccessError,
}
impl Error for TaskLocalErr {}
impl fmt::Debug for TaskLocalErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::BorrowError => f.pad(
                "cannot enter a task-local scope while the task-local storage is borrowed"
            ),
            Self::AccessError => f.pad(
                "cannot enter a task-local scope during or after destruction \
                    of the underlying thread-local"
            ),
        }
    }
}
impl fmt::Display for TaskLocalErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::BorrowError => f.pad("task-local value already borrowed"),
            Self::AccessError => f.pad("task-local value not set"),
        }
    }
}
impl TaskLocalErr {
    #[track_caller]
    fn panic(&self) -> ! {
        match self {
            Self::BorrowError => panic!(
                "cannot enter a task-local scope while the task-local storage is borrowed"
            ),
            Self::AccessError => panic!(
                "cannot enter a task-local scope during or after destruction \
                    of the underlying thread-local"
            ),
        }
    }
    #[inline(always)]
    #[track_caller]
    pub fn is_access_err(&self) -> bool {
        *self == Self::AccessError
    }
}

impl From<std::cell::BorrowMutError> for TaskLocalErr {
    fn from(_: std::cell::BorrowMutError) -> Self {
        Self::BorrowError
    }
}

impl From<std::thread::AccessError> for TaskLocalErr {
    fn from(_: std::thread::AccessError) -> Self {
        Self::AccessError
    }
}
