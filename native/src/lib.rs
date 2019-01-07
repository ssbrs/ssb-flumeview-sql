#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate failure;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate flumedb;
extern crate node_napi;

use node_napi::napi::*;
use node_napi::napi_sys::*;
use node_napi::value::*;
use std::debug_assert;
use std::os::raw::{c_char, c_void};
use std::ptr::{null, null_mut};
use std::slice;

use flumedb::FlumeViewSql;
use flumedb::OffsetLog;
use flumedb::Sequence;

struct SsbQuery {
    view: FlumeViewSql,
    log: OffsetLog<u32>,
}

impl SsbQuery {
    fn new(log_path: String, view_path: String) -> SsbQuery {
        let view = FlumeViewSql::new(&view_path);
        let log = OffsetLog::new(log_path);

        SsbQuery { view, log }
    }

    fn get_latest(&self) -> Sequence {
        self.view.latest
    }

    fn process(&mut self, num_items: usize) -> napi_value {
        unimplemented!();
    }

    fn query(&self, query_string: String) -> napi_value {
        unimplemented!();
    }
}

#[no_mangle]
extern "C" fn get_latest(env: napi_env, info: napi_callback_info) -> napi_value {
    let this = get_this(env, info);
    let mut result = null_mut(); 

    unsafe {napi_unwrap(env, this, &mut result)};

    let ssb_query = result as *mut SsbQuery;
    let latest = unsafe{
        (*ssb_query).get_latest()
    };

    wrap_unsafe_create::<i64>(env, latest as i64, napi_create_int64)

}

#[no_mangle]
pub extern "C" fn define_view_class(env: napi_env) -> napi_value {
    let latest_property: napi_property_descriptor = napi_property_descriptor {
        utf8name: null(),
        name: create_string_utf8(env, "getLatest"),
        method: Some(get_latest),
        getter: None,
        setter: None,
        value: null_mut(),
        attributes: napi_property_attributes_napi_default, 
        data: null_mut()
    };
    let properties = vec![latest_property];
    let data = null_mut();

    define_class(
        env,
        "SqlView",
        Some(construct_view_class),
        data,
        &properties,
    )
}

#[no_mangle]
pub extern "C" fn finalize_view(env: napi_env, data: *mut c_void, _: *mut c_void) {
    let ssb_query = data as *mut SsbQuery;
    unsafe { Box::from_raw(ssb_query) };
}

#[no_mangle]
pub extern "C" fn construct_view_class(env: napi_env, info: napi_callback_info) -> napi_value {
    let this = get_this(env, info);

    let path_to_offset_value = get_arg(env, info, 0);
    let path_to_db_value = get_arg(env, info, 1);

    let path_to_offset = get_string(env, path_to_offset_value).unwrap();
    let path_to_db = get_string(env, path_to_db_value).unwrap();

    let mut wrapped_ref: napi_ref = null_mut();
    let finalize_hint: *mut c_void = null_mut();

    let ssb_query = Box::new(SsbQuery::new(path_to_offset, path_to_db));

    let status = unsafe {
        napi_wrap(
            env,
            this,
            Box::into_raw(ssb_query) as *mut c_void,
            Some(finalize_view),
            finalize_hint,
            &mut wrapped_ref,
        )
    };

    debug_assert!(status == napi_status_napi_ok);

    this
}
