//! Raw FFI bindings for wayland-client, linked directly at build time.

use std::{
    ffi::{CStr, c_char, c_int, c_void},
    ptr,
    sync::OnceLock,
};

use tracing::debug;

// === === === === === === === === === ===
// ===          Opaque Types           ===
// === === === === === === === === === ===

pub enum WlDisplay {}
pub enum WlRegistry {}
pub enum WlProxy {}
pub enum WlSurface {}
pub enum IdleInhibitManager {}
pub enum Inhibitor {}
pub enum GdkWaylandDisplay {}
pub enum GdkWaylandSurface {}

// === === === === === === === === === ===
// ===       Protocol Constants        ===
// === === === === === === === === === ===

const WL_MARSHAL_FLAG_DESTROY: u32 = 1;
const WL_DISPLAY_GET_REGISTRY: u32 = 1;
const REGISTRY_BIND: u32 = 0;
const MANAGER_CREATE_INHIBITOR: u32 = 1;
const INHIBITOR_DESTROY: u32 = 0;

const IDLE_INHIBIT_MANAGER_NAME: &[u8] = b"zwp_idle_inhibit_manager_v1\0";
const IDLE_INHIBITOR_NAME: &[u8] = b"zwp_idle_inhibitor_v1\0";

// === === === === === === === === === === ===
// ===      Wayland Protocol Structs       ===
// === === === === === === === === === === ===

#[repr(C)]
pub struct WlMessage {
    pub name: *const c_char,
    pub signature: *const c_char,
    pub types: *const *const WlInterface,
}

#[repr(C)]
pub struct WlInterface {
    pub name: *const c_char,
    pub version: c_int,
    pub method_count: c_int,
    pub methods: *const WlMessage,
    pub event_count: c_int,
    pub events: *const WlMessage,
}

/// # Safety
///
/// These types contain only `*const` pointers to static data (string literals
/// and static arrays). No mutable aliasing is possible.
unsafe impl Sync for WlInterface {}
unsafe impl Sync for WlMessage {}

// === === === === === === === === === ===
// ===      Protol Interface Defs      ===
// === === === === === === === === === ===

static mut MANAGER_REQUESTS_TYPES: [*const WlInterface; 2] = [ptr::null(), ptr::null()];

static MANAGER_REQUESTS: [WlMessage; 2] = [
    WlMessage {
        name: c"destroy".as_ptr(),
        signature: c"".as_ptr(),
        types: ptr::null(),
    },
    WlMessage {
        name: c"create_inhibitor".as_ptr(),
        signature: c"no".as_ptr(),
        types: ptr::addr_of!(MANAGER_REQUESTS_TYPES) as *const _,
    },
];

pub static MANAGER_INTERFACE: WlInterface = WlInterface {
    name: IDLE_INHIBIT_MANAGER_NAME.as_ptr() as *const c_char,
    version: 1,
    method_count: 2,
    methods: MANAGER_REQUESTS.as_ptr(),
    event_count: 0,
    events: ptr::null(),
};

static INHIBITOR_REQUESTS: [WlMessage; 1] = [WlMessage {
    name: c"destroy".as_ptr(),
    signature: c"".as_ptr(),
    types: ptr::null(),
}];

pub static INHIBITOR_INTERFACE: WlInterface = WlInterface {
    name: IDLE_INHIBITOR_NAME.as_ptr() as *const c_char,
    version: 1,
    method_count: 1,
    methods: INHIBITOR_REQUESTS.as_ptr(),
    event_count: 0,
    events: ptr::null(),
};

// === === === === === === === === === ===
// ===        Registry Listener        ===
// === === === === === === === === === ===

#[repr(C)]
pub struct RegistryListener {
    pub global: extern "C" fn(*mut c_void, *mut WlRegistry, u32, *const c_char, u32),
    pub global_remove: extern "C" fn(*mut c_void, *mut WlRegistry, u32),
}

pub struct RegistryState {
    pub manager: *mut IdleInhibitManager,
}

extern "C" fn on_registry_global(
    data: *mut c_void,
    registry: *mut WlRegistry,
    name: u32,
    interface: *const c_char,
    version: u32,
) {
    // SAFETY: `data` is a valid pointer to RegistryState passed by the caller.
    // `interface` is a null-terminated string from libwayland.
    unsafe {
        let iface = CStr::from_ptr(interface);
        if iface.to_bytes() != b"zwp_idle_inhibit_manager_v1" {
            return;
        }

        debug!(name, version, "found idle_inhibit_manager");
        let state = &mut *(data as *mut RegistryState);

        init_protocol_types();
        state.manager = sys::wl_proxy_marshal_flags(
            registry as *mut WlProxy,
            REGISTRY_BIND,
            &MANAGER_INTERFACE,
            version.min(1),
            0,
            name,
            MANAGER_INTERFACE.name,
            version.min(1),
            ptr::null::<c_void>(),
        ) as *mut IdleInhibitManager;
    }
}

extern "C" fn on_registry_global_remove(
    _data: *mut c_void,
    _registry: *mut WlRegistry,
    _name: u32,
) {
}

pub static REGISTRY_LISTENER: RegistryListener = RegistryListener {
    global: on_registry_global,
    global_remove: on_registry_global_remove,
};

// === === === === === === === === === ===
// ===      Directly Linked Symbols    ===
// === === === === === === === === === ===

mod sys {
    use std::ffi::{c_int, c_void};

    use super::{WlInterface, WlProxy};

    // Declare the library these symbols come from so rustc emits
    // `-lwayland-client` adjacent to this crate's objects. This keeps it out of
    // reach of the linker's `--as-needed` pruning (which can otherwise drop a
    // `-lwayland-client` that appears before the code referencing it, depending
    // on link ordering) and propagates the link requirement to every binary and
    // test that depends on this crate. gtk4-layer-shell interposition ordering
    // is still the final binary's concern (see the shell's build script).
    #[link(name = "wayland-client")]
    unsafe extern "C" {
        pub fn wl_display_roundtrip(display: *mut super::WlDisplay) -> c_int;
        pub fn wl_proxy_add_listener(
            proxy: *mut WlProxy,
            listener: *const c_void,
            data: *mut c_void,
        ) -> c_int;
        pub fn wl_proxy_get_version(proxy: *mut WlProxy) -> u32;
        pub fn wl_proxy_marshal_flags(
            proxy: *mut WlProxy,
            opcode: u32,
            interface: *const WlInterface,
            version: u32,
            flags: u32,
            ...
        ) -> *mut WlProxy;

        pub static wl_registry_interface: WlInterface;
        pub static wl_surface_interface: WlInterface;
    }
}

static PROTOCOL_TYPES_INIT: OnceLock<()> = OnceLock::new();

fn init_protocol_types() {
    PROTOCOL_TYPES_INIT.get_or_init(|| unsafe {
        MANAGER_REQUESTS_TYPES[0] = &INHIBITOR_INTERFACE;
        MANAGER_REQUESTS_TYPES[1] = &sys::wl_surface_interface;
    });
}

pub fn is_available() -> bool {
    true
}

// === === === === === === === === === ===
// ===      Public FFI Functoins       ===
// === === === === === === === === === ===

/// # Safety
///
/// `display` must be a valid `wl_display` pointer from GDK.
pub unsafe fn wl_display_roundtrip(display: *mut WlDisplay) -> c_int {
    unsafe { sys::wl_display_roundtrip(display) }
}

/// # Safety
///
/// `proxy` must be a valid Wayland proxy. `listener` must point to the correct
/// listener struct for the proxy's interface. `data` is passed to callbacks.
pub unsafe fn wl_proxy_add_listener(
    proxy: *mut WlProxy,
    listener: *const c_void,
    data: *mut c_void,
) -> c_int {
    unsafe { sys::wl_proxy_add_listener(proxy, listener, data) }
}

/// # Safety
///
/// `proxy` must be a valid Wayland proxy.
pub unsafe fn wl_proxy_get_version(proxy: *mut WlProxy) -> u32 {
    unsafe { sys::wl_proxy_get_version(proxy) }
}

/// # Safety
///
/// `display` must be a valid `wl_display` pointer from GDK.
pub unsafe fn wl_display_get_registry(display: *mut WlDisplay) -> *mut WlRegistry {
    init_protocol_types();
    unsafe {
        sys::wl_proxy_marshal_flags(
            display as *mut WlProxy,
            WL_DISPLAY_GET_REGISTRY,
            &sys::wl_registry_interface,
            1,
            0,
            ptr::null::<c_void>(),
        ) as *mut WlRegistry
    }
}

/// # Safety
///
/// `manager` must be a valid `zwp_idle_inhibit_manager_v1` proxy.
/// `surface` must be a valid `wl_surface` pointer from GDK.
pub unsafe fn create_inhibitor(
    manager: *mut IdleInhibitManager,
    surface: *mut WlSurface,
) -> *mut Inhibitor {
    init_protocol_types();
    unsafe {
        sys::wl_proxy_marshal_flags(
            manager as *mut WlProxy,
            MANAGER_CREATE_INHIBITOR,
            &INHIBITOR_INTERFACE,
            wl_proxy_get_version(manager as *mut WlProxy),
            0,
            ptr::null::<c_void>(),
            surface,
        ) as *mut Inhibitor
    }
}

/// # Safety
///
/// `inhibitor` must be a valid `zwp_idle_inhibitor_v1` proxy that has not
/// been destroyed. After this call, `inhibitor` is invalid.
pub unsafe fn destroy_inhibitor(inhibitor: *mut Inhibitor) {
    init_protocol_types();
    unsafe {
        sys::wl_proxy_marshal_flags(
            inhibitor as *mut WlProxy,
            INHIBITOR_DESTROY,
            ptr::null::<WlInterface>(),
            wl_proxy_get_version(inhibitor as *mut WlProxy),
            WL_MARSHAL_FLAG_DESTROY,
        );
    }
}

// === === === === === === === === === ===
// ===      GDK Wayland Functions      ===
// === === === === === === === === === ===

#[link(name = "gtk-4")]
unsafe extern "C" {
    pub fn gdk_wayland_display_get_wl_display(display: *mut GdkWaylandDisplay) -> *mut WlDisplay;
    pub fn gdk_wayland_surface_get_wl_surface(surface: *mut GdkWaylandSurface) -> *mut WlSurface;
}
