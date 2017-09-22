use std;
use std::os::raw::{c_void, c_char};
use std::ffi::CStr;
use cervus;

pub struct Caller {
    ee: cervus::engine::ExecutionEngine
}

#[no_mangle]
pub extern "C" fn cervus_ffi_build_caller_with_context(
    target: *const c_void,
    call_context: *const c_void,
    ret_type: *const c_char,
    param_types: *const *const c_char,
    num_params: u32
) -> *mut Caller {
    let ret_type = unsafe {
        CStr::from_ptr(ret_type).to_str().unwrap()
    };
    let param_types = unsafe {
        std::slice::from_raw_parts(param_types, num_params as usize)
            .iter().map(|v| CStr::from_ptr(*v).to_str().unwrap())
    };

    let m = cervus::engine::Module::new("caller");
    let mut param_types: Vec<cervus::patcher::Argument> = param_types
        .map(parse_value_type)
        .map(|v| match v {
            Some(v) => v,
            None => panic!("Invalid param type(s)")
        })
        .map(|v| cervus::patcher::Argument::FromCall(v))
        .collect();

    let ret_type = match parse_value_type(ret_type) {
        Some(v) => v,
        None => panic!("Invalid return type")
    };

    param_types.push(cervus::patcher::Argument::LocalRaw(call_context));
    let m = cervus::patcher::add_local_fn(m, "target", target, ret_type, param_types);

    let caller = Box::new(Caller {
        ee: cervus::engine::ExecutionEngine::new(m)
    });

    Box::into_raw(caller)
}

pub unsafe extern "C" fn cervus_ffi_destroy_caller(
    caller: *mut Caller
) {
    Box::from_raw(caller);
}

pub unsafe extern "C" fn cervus_ffi_get_callable(
    caller: &Caller
) -> *const c_void {
    caller.ee.get_raw_callable(&cervus::engine::Function::new_null_handle(
        "target",
        cervus::value_type::ValueType::Void,
        Vec::new()
    ))
}

fn parse_value_type(t: &str) -> Option<cervus::value_type::ValueType> {
    use cervus::value_type::ValueType::*;
    match t {
        "i8" => Some(Int8),
        "i16" => Some(Int16),
        "i32" => Some(Int32),
        "i64" => Some(Int64),
        "f64" => Some(Float64),
        "pointer" => Some(Pointer(Box::new(Void))),
        _ => None
    }
}
