use cervus;
pub use cervus::ext;

use std;
use std::any::Any;
use std::ops::Deref;
use std::os::raw::{c_char, c_void};
use std::ffi::CStr;

pub struct UnknownData {}

pub struct UnknownDataWrapper {
    inner: *const UnknownData
}

unsafe impl Send for UnknownDataWrapper {}
unsafe impl Sync for UnknownDataWrapper {}

impl Deref for UnknownDataWrapper {
    type Target = *const UnknownData;

    fn deref(&self) -> &*const UnknownData {
        &self.inner
    }
}

impl From<*const UnknownData> for UnknownDataWrapper {
    fn from(s: *const UnknownData) -> UnknownDataWrapper {
        UnknownDataWrapper {
            inner: s
        }
    }
}

struct BridgeHookContext {
    inner: *const UnknownData
}

impl From<*const UnknownData> for BridgeHookContext {
    fn from(s: *const UnknownData) -> BridgeHookContext {
        BridgeHookContext {
            inner: s
        }
    }
}

impl Into<Box<Any>> for Box<BridgeHookContext> {
    fn into(self) -> Box<Any> {
        self
    }
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_create() -> *mut cervus::manager::Modules {
    Box::into_raw(Box::new(cervus::manager::Modules::new()))
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_destroy(m: *mut cervus::manager::Modules) {
    Box::from_raw(m);
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_load(
    m: *mut cervus::manager::Modules,
    name: *const c_char,
    code: *const u8,
    code_len: u32
) {
    let m = &*m;
    let name = CStr::from_ptr(name).to_str().unwrap();
    let code = std::slice::from_raw_parts(code, code_len as usize);

    m.load(name, code, cervus::manager::ExternalResources::new());
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_unload(
    m: *mut cervus::manager::Modules,
    name: *const c_char
) -> bool {
    let m = &*m;
    let name = CStr::from_ptr(name).to_str().unwrap();

    m.unload(name)
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_run_hooks_by_name(
    m: *mut cervus::manager::Modules,
    name: *const c_char,
    ctx: *const UnknownData
) {
    let m = &*m;
    let name = CStr::from_ptr(name).to_str().unwrap();
    let ctx = Box::new(BridgeHookContext::from(ctx));

    m.run_hooks_by_name(name, ctx);
}

#[no_mangle]
pub unsafe fn cervus_manager_modules_add_downcast_provider(
    m: *mut cervus::manager::Modules,
    name: *const c_char,
    cb: extern fn (call_with: *const UnknownData, data: *const UnknownData) -> *const c_void,
    call_with: *const UnknownData
) {
    let m = &*m;
    let name = CStr::from_ptr(name).to_str().unwrap();
    let call_with = UnknownDataWrapper::from(call_with);

    m.add_downcast_provider(name, Box::new(move |v| {
        let ctx = v.downcast_ref::<BridgeHookContext>().unwrap();
        cb(*call_with, ctx.inner)
    }));
}
