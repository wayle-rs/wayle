//! Tokio mainloop for PulseAudio
//!
//! This code is from Daniel De Graaf's pulse-binding-rust project.
//! Original: <https://github.com/danieldg/pulse-binding-rust/tree/master/pulse-tokio>
//! Copyright (c) Daniel De Graaf <code@danieldg.net>
//! Licensed under MIT OR Apache-2.0
//!
//! Modified for this project with slight adjustments.
//! Incorporated into this GPL-3.0 project under compatible license terms.

#![allow(unsafe_code)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cognitive_complexity)]
use std::{
    cell::{Cell, UnsafeCell},
    cmp, fmt,
    future::{Future, poll_fn},
    os::{
        raw::c_void,
        unix::io::{AsRawFd, RawFd},
    },
    pin::Pin,
    rc::{Rc, Weak},
    task,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use libc::timeval;
use libpulse_binding::{
    context::{self, Context},
    def::{Retval, RetvalActual},
    mainloop::{
        api::{
            DeferEventCb, DeferEventDestroyCb, IoEventCb, IoEventDestroyCb,
            Mainloop as MainloopTrait, MainloopApi, MainloopInnerType, MainloopInternalType,
            TimeEventCb, TimeEventDestroyCb,
        },
        events::{
            deferred::DeferEventInternal,
            io::{FlagSet as IoEventFlagSet, IoEventInternal},
            timer::TimeEventInternal,
        },
    },
};
use tokio::io::unix::AsyncFd;

struct Fd(RawFd);

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

enum Item {
    Defer {
        main: Weak<MainInner>,
        dead: bool,
        enabled: bool,
        cb: Option<DeferEventCb>,
        userdata: *mut c_void,
        free: Option<DeferEventDestroyCb>,
    },
    Timer {
        main: Weak<MainInner>,
        dead: bool,
        ts: Cell<Option<Duration>>,
        cb: Option<TimeEventCb>,
        userdata: *mut c_void,
        free: Option<TimeEventDestroyCb>,
    },
    Event {
        main: Weak<MainInner>,
        dead: Cell<bool>,
        fd: i32,
        afd: Cell<Option<AsyncFd<Fd>>>,
        cb: Option<IoEventCb>,
        events: Cell<IoEventFlagSet>,
        userdata: *mut c_void,
        free: Option<IoEventDestroyCb>,
    },
}

impl Item {
    fn is_dead(&self) -> bool {
        match self {
            Item::Defer { dead, .. } | Item::Timer { dead, .. } => *dead,
            Item::Event { dead, .. } => dead.get(),
        }
    }

    fn kill(&mut self) {
        match self {
            Item::Defer { dead, .. } | Item::Timer { dead, .. } => {
                *dead = true;
            }
            Item::Event { dead, .. } => {
                dead.set(true);
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct TokioMain {
    mi: Rc<MainInner>,
}

pub(crate) struct MainInner {
    api: MainloopApi,
    items: UnsafeCell<Vec<*mut Item>>,
    sleep: UnsafeCell<Option<tokio::time::Sleep>>,
    waker: Cell<Option<task::Waker>>,
    quit: Cell<Option<RetvalActual>>,
}

impl fmt::Debug for MainInner {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "MainInner")
    }
}

impl MainloopTrait for TokioMain {
    type MI = MainInner;
    fn inner(&self) -> Rc<MainInner> {
        self.mi.clone()
    }
}

impl MainloopInternalType for MainInner {}

impl MainloopInnerType for MainInner {
    type I = Self;
    fn get_ptr(&self) -> *mut Self {
        panic!("This function is not well-defined and is never called")
    }

    fn get_api_ptr(&self) -> *const MainloopApi {
        &self.api
    }

    fn get_api(&self) -> &MainloopApi {
        &self.api
    }

    fn supports_rtclock(&self) -> bool {
        false
    }
}

impl Drop for MainInner {
    fn drop(&mut self) {
        // SAFETY: userdata is the Weak ptr from new_cyclic's into_raw()
        unsafe {
            Weak::from_raw(self.api.userdata as *mut MainInner);

            let api = &self.api;
            for item in self.items.get_mut().drain(..) {
                let mut boxed = Box::from_raw(item);
                let raw_item = &mut *boxed as *mut Item;

                // SAFETY: free callbacks expect the same api/item/userdata from registration
                match boxed.as_ref() {
                    Item::Defer {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    Item::Timer {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    Item::Event {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    _ => {}
                }
                drop(boxed);
            }
        }
    }
}

impl Default for TokioMain {
    fn default() -> Self {
        Self::new()
    }
}

impl TokioMain {
    /// Create a new tokio mainloop for PulseAudio.
    pub fn new() -> Self {
        let mi = Rc::new_cyclic(|weak| {
            let weak_ptr = weak.clone().into_raw();

            MainInner {
                api: MainloopApi {
                    userdata: weak_ptr as *mut c_void,
                    io_new: Some(MainInner::io_new),
                    io_enable: Some(MainInner::io_enable),
                    io_free: Some(MainInner::io_free),
                    io_set_destroy: Some(MainInner::io_set_destroy),
                    time_new: Some(MainInner::time_new),
                    time_restart: Some(MainInner::time_restart),
                    time_free: Some(MainInner::time_free),
                    time_set_destroy: Some(MainInner::time_set_destroy),
                    defer_new: Some(MainInner::defer_new),
                    defer_enable: Some(MainInner::defer_enable),
                    defer_free: Some(MainInner::defer_free),
                    defer_set_destroy: Some(MainInner::defer_set_destroy),
                    quit: Some(MainInner::quit),
                },
                items: UnsafeCell::new(Vec::new()),
                sleep: UnsafeCell::new(None),
                waker: Cell::new(None),
                quit: Cell::new(None),
            }
        });
        TokioMain { mi }
    }

    fn iter_get_item(&mut self, i: usize) -> Option<(&MainloopApi, &Item)> {
        let api = &self.mi.api;
        let items = unsafe { &mut *self.mi.items.get() };
        loop {
            if i >= items.len() {
                return None;
            }
            if unsafe { (*items[i]).is_dead() } {
                let mut dead = unsafe { Box::from_raw(items.swap_remove(i)) };
                let raw_item = &mut *dead as *mut Item;
                match dead.as_ref() {
                    Item::Defer {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    Item::Timer {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    Item::Event {
                        free: Some(cb),
                        userdata,
                        ..
                    } => {
                        cb(api, raw_item as *mut _, *userdata);
                    }
                    _ => {}
                }
                drop(dead);
                continue;
            }
            let item = unsafe { &*items[i] };
            return Some((api, item));
        }
    }

    /// Run callbacks and register wakers for the pulse mainloop.
    pub fn tick(&mut self, ctx: &mut task::Context) -> task::Poll<Option<Retval>> {
        let inow = tokio::time::Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        let mut wake = None::<Duration>;
        let mut rv = task::Poll::Pending;
        let mut i = 0;
        self.mi.waker.set(Some(ctx.waker().clone()));
        while let Some((api, item)) = self.iter_get_item(i) {
            let raw_item = item as *const Item;
            i += 1;
            match item {
                Item::Defer {
                    enabled: true,
                    cb: Some(cb),
                    userdata,
                    ..
                } => {
                    cb(api, raw_item as *mut _, *userdata);
                }
                Item::Defer { .. } => continue,
                Item::Timer { cb: None, .. } => continue,
                Item::Timer {
                    cb: Some(cb),
                    userdata,
                    ts,
                    ..
                } => {
                    match ts.replace(None) {
                        Some(ts) if ts < now => {
                            rv = task::Poll::Ready(None);
                            let tv = timeval {
                                tv_sec: ts.as_secs() as i64,
                                tv_usec: ts.subsec_micros() as i64,
                            };
                            cb(api, raw_item as *mut _, &tv, *userdata);
                        }
                        later => ts.set(later),
                    }

                    if let Some(ts) = ts.get() {
                        if wake.is_some() {
                            wake = cmp::min(wake, Some(ts));
                        } else {
                            wake = Some(ts);
                        }
                    }
                }
                Item::Event { cb: None, .. } => continue,
                Item::Event {
                    cb: Some(cb),
                    userdata,
                    fd,
                    afd,
                    events,
                    dead,
                    ..
                } => {
                    let mut local_fd = afd.take();

                    let async_fd = match local_fd {
                        Some(ref fd) => fd,
                        None => {
                            let Ok(fd) = AsyncFd::new(Fd(*fd)) else {
                                afd.set(None);
                                continue;
                            };
                            local_fd.insert(fd)
                        }
                    };
                    let mut ready = IoEventFlagSet::NULL;
                    let mut rg = None;
                    let mut wg = None;
                    let current_events = unsafe { events.as_ptr().read() };
                    if current_events.contains(IoEventFlagSet::INPUT) {
                        match async_fd.poll_read_ready(ctx) {
                            task::Poll::Ready(Ok(g)) => {
                                ready |= IoEventFlagSet::INPUT;
                                rg = Some(g);
                            }
                            task::Poll::Ready(Err(_)) => ready |= IoEventFlagSet::ERROR,
                            task::Poll::Pending => {}
                        }
                    }
                    if current_events.contains(IoEventFlagSet::OUTPUT) {
                        match async_fd.poll_write_ready(ctx) {
                            task::Poll::Ready(Ok(g)) => {
                                ready |= IoEventFlagSet::OUTPUT;
                                wg = Some(g);
                            }
                            task::Poll::Ready(Err(_)) => ready |= IoEventFlagSet::ERROR,
                            task::Poll::Pending => {}
                        }
                    }
                    if ready.is_empty() {
                        afd.set(local_fd);
                        continue;
                    }

                    rv = task::Poll::Ready(None);
                    let ready_events_for_check = if rg.is_some() {
                        IoEventFlagSet::INPUT
                    } else {
                        IoEventFlagSet::NULL
                    } | if wg.is_some() {
                        IoEventFlagSet::OUTPUT
                    } else {
                        IoEventFlagSet::NULL
                    };
                    cb(api, raw_item as *mut _, *fd, ready, *userdata);
                    if dead.get() {
                        continue;
                    }
                    let wants = current_events;
                    if wants.intersects(ready_events_for_check) {
                        let mut pfd = libc::pollfd {
                            fd: *fd,
                            events: 0,
                            revents: 0,
                        };
                        if wants.contains(IoEventFlagSet::INPUT) && rg.is_some() {
                            pfd.events |= libc::POLLIN;
                        }
                        if wants.contains(IoEventFlagSet::OUTPUT) && wg.is_some() {
                            pfd.events |= libc::POLLOUT;
                        }
                        unsafe {
                            libc::poll(&mut pfd, 1, 0);
                        }
                        if let Some(mut g) = rg {
                            if (pfd.revents & libc::POLLIN) != 0 {
                                g.retain_ready();
                            } else {
                                g.clear_ready();
                            }
                        }
                        if let Some(mut g) = wg {
                            if (pfd.revents & libc::POLLOUT) != 0 {
                                g.retain_ready();
                            } else {
                                g.clear_ready();
                            }
                        }
                    }
                    afd.set(local_fd);
                }
            }
        }
        if let Some(ret) = self.mi.quit.replace(None) {
            return task::Poll::Ready(Some(Retval(ret)));
        }
        if rv.is_pending() {
            let mut sleep = unsafe { Pin::new_unchecked(&mut *self.mi.sleep.get()) };
            if let Some(d) = wake {
                sleep.set(Some(tokio::time::sleep_until(inow + d)));
                if let Some(task::Poll::Ready(())) =
                    sleep.as_mut().as_pin_mut().map(|f| f.poll(ctx))
                {
                    sleep.set(None);
                    rv = task::Poll::Ready(None);
                }
            } else {
                sleep.set(None);
            }
        }
        rv
    }

    /// Run the mainloop until the given context is either Ready or Failed/Terminated.
    ///
    /// # Errors
    /// Returns error if the mainloop quits with an error retval
    pub async fn wait_for_ready(&mut self, ctx: &Context) -> Result<context::State, Retval> {
        loop {
            if let Some(rv) = poll_fn(|ctx| self.tick(ctx)).await {
                return Err(rv);
            }
            let s = ctx.get_state();
            match s {
                context::State::Ready | context::State::Failed | context::State::Terminated => {
                    return Ok(s);
                }
                _ => {}
            }
        }
    }
}

impl Drop for TokioMain {
    fn drop(&mut self) {
        let mut sleep = unsafe { Pin::new_unchecked(&mut *self.mi.sleep.get()) };
        sleep.set(None);
    }
}

impl MainInner {
    unsafe fn from_api(api: *const MainloopApi) -> Rc<Self> {
        let ptr = unsafe { Weak::from_raw((*api).userdata as *const Self) };
        let rv = ptr.upgrade().expect(
            "MainloopApi callback called after TokioMain was dropped. \
             This indicates a shutdown ordering bug - ensure Context is \
             fully disconnected before dropping the mainloop.",
        );
        let _ = ptr.into_raw();
        rv
    }

    fn push(&self, item: Box<Item>) {
        let items = unsafe { &mut *self.items.get() };
        items.push(Box::into_raw(item));
    }

    fn wake(main: &Weak<MainInner>) {
        if let Some(inner) = main.upgrade() {
            inner.wake_real();
        }
    }

    fn wake_real(&self) {
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }

    extern "C" fn io_new(
        a: *const MainloopApi,
        fd: i32,
        events: IoEventFlagSet,
        cb: Option<IoEventCb>,
        userdata: *mut c_void,
    ) -> *mut IoEventInternal {
        unsafe {
            let inner = MainInner::from_api(a);
            let events = Cell::new(events);
            let mut item = Box::new(Item::Event {
                fd,
                cb,
                events,
                userdata,
                free: None,
                afd: Cell::new(None),
                dead: Cell::new(false),
                main: Rc::downgrade(&inner),
            });
            let rv = &mut *item as *mut Item as *mut _;
            inner.push(item);
            inner.wake_real();
            rv
        }
    }
    extern "C" fn io_enable(e: *mut IoEventInternal, new: IoEventFlagSet) {
        unsafe {
            let item: *mut Item = e.cast();
            if let Item::Event { main, events, .. } = &*item {
                events.set(new);
                MainInner::wake(main);
            }
        }
    }
    extern "C" fn io_free(e: *mut IoEventInternal) {
        unsafe {
            let item: *mut Item = e.cast();
            if let Item::Event { dead, afd, .. } = &*item {
                dead.set(true);
                afd.set(None);
            }
        }
    }
    extern "C" fn io_set_destroy(e: *mut IoEventInternal, cb: Option<IoEventDestroyCb>) {
        unsafe {
            let item: *mut Item = e.cast();
            match &mut *item {
                Item::Event { free, .. } => {
                    *free = cb;
                }
                _ => panic!(),
            }
        }
    }
    extern "C" fn time_new(
        a: *const MainloopApi,
        tv: *const timeval,
        cb: Option<TimeEventCb>,
        userdata: *mut c_void,
    ) -> *mut TimeEventInternal {
        unsafe {
            let inner = MainInner::from_api(a);
            let tv = tv.read();
            let ts = Cell::new(Some(
                Duration::from_secs(tv.tv_sec as u64) + Duration::from_micros(tv.tv_usec as u64),
            ));
            let mut item = Box::new(Item::Timer {
                main: Rc::downgrade(&inner),
                ts,
                cb,
                userdata,
                free: None,
                dead: false,
            });
            let rv = &mut *item as *mut Item as *mut _;
            inner.push(item);
            inner.wake_real();
            rv
        }
    }
    extern "C" fn time_restart(e: *mut TimeEventInternal, tv: *const timeval) {
        unsafe {
            let item: *mut Item = e.cast();
            match &*item {
                Item::Timer { main, ts, .. } => {
                    let tv = tv.read();
                    ts.set(Some(
                        Duration::from_secs(tv.tv_sec as u64)
                            + Duration::from_micros(tv.tv_usec as u64),
                    ));
                    MainInner::wake(main);
                }
                _ => panic!(),
            }
        }
    }
    extern "C" fn time_free(e: *mut TimeEventInternal) {
        unsafe {
            let item: *mut Item = e.cast();
            (*item).kill();
        }
    }
    extern "C" fn time_set_destroy(e: *mut TimeEventInternal, cb: Option<TimeEventDestroyCb>) {
        unsafe {
            let item: *mut Item = e.cast();
            match &mut *item {
                Item::Timer { free, .. } => {
                    *free = cb;
                }
                _ => panic!(),
            }
        }
    }
    extern "C" fn defer_new(
        a: *const MainloopApi,
        cb: Option<DeferEventCb>,
        userdata: *mut c_void,
    ) -> *mut DeferEventInternal {
        unsafe {
            let inner = MainInner::from_api(a);
            let mut item = Box::new(Item::Defer {
                main: Rc::downgrade(&inner),
                cb,
                userdata,
                free: None,
                dead: false,
                enabled: true,
            });
            let rv = &mut *item as *mut Item as *mut _;
            inner.push(item);
            inner.wake_real();
            rv
        }
    }
    extern "C" fn defer_enable(e: *mut DeferEventInternal, b: i32) {
        unsafe {
            let item: *mut Item = e.cast();
            match &mut *item {
                Item::Defer { main, enabled, .. } => {
                    *enabled = b != 0;
                    if b != 0 {
                        MainInner::wake(main);
                    }
                }
                Item::Timer { .. } => {}
                Item::Event { .. } => {}
            }
        }
    }
    extern "C" fn defer_free(e: *mut DeferEventInternal) {
        unsafe {
            let item: *mut Item = e.cast();
            (*item).kill();
        }
    }
    extern "C" fn defer_set_destroy(e: *mut DeferEventInternal, cb: Option<DeferEventDestroyCb>) {
        unsafe {
            let item: *mut Item = e.cast();
            match &mut *item {
                Item::Defer { free, .. } => {
                    *free = cb;
                }
                _ => panic!(),
            }
        }
    }
    extern "C" fn quit(a: *const MainloopApi, retval: RetvalActual) {
        unsafe {
            let inner = MainInner::from_api(a);
            inner.quit.set(Some(retval));
            inner.wake_real();
        }
    }
}
