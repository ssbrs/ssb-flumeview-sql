//! Data structures for storing and manipulating arbitrary legacy data.

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{BTreeMap, btree_map};
use std::fmt;
use std::ptr;
use std::marker::PhantomData;

use indexmap::{IndexMap, map};
use serde::{
    ser::{Serialize, Serializer, SerializeSeq, SerializeMap},
    de::{Deserialize, DeserializeSeed, Deserializer, Visitor, SeqAccess, MapAccess, Error},
};

use ssb_legacy_msg_data::{LegacyF64, legacy_length};
use napi_sys::*;
use napi::*;

// The maximum capacity of entries to preallocate for arrays and objects. Even if malicious input
// claims to contain a much larger collection, only this much memory will be blindly allocated.
static MAX_ALLOC: usize = 2048;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Value{
    env: napi_env,
    value: napi_value
}

impl Value {
    fn get_typeof(&self)-> napi_valuetype {
        get_typeof(self.env, self.value)
    }
}

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.get_typeof() {
            napi_valuetype_napi_null => serializer.serialize_unit(),
            napi_valuetype_napi_boolean =>{
                let b = wrap_unsafe_get(self.env, self.value, napi_get_value_bool);
                serializer.serialize_bool(b)
            },
            napi_valuetype_napi_number => {
                let n = wrap_unsafe_get(self.env, self.value, napi_get_value_double);
                serializer.serialize_f64(n)
            }
            //Value::String(ref s) => serializer.serialize_str(&s.value),
            //Value::Array(ref v) => {
            //    let mut s = serializer.serialize_seq(Some(v.value.len()))?;
            //    for inner in v.value {
            //        s.serialize_element(&inner)?;
            //    }
            //    s.end()
            //},
            //
            napi_valuetype_napi_object => {
                let mut m = serializer.serialize_map(None)?;
                for (key, value) in get_object_map(self.env, self.value) {
                    m.serialize_entry(&key, &Value{env: self.env, value: self.value})?;
                }
                m.end()
            },
            _ => serializer.serialize_unit()
        }
    }
}

struct ValueVisitor{
    env: napi_env
}

impl<'de> DeserializeSeed<'de> for NapiEnv {
    type Value = Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ValueVisitor{env: self.env})
    }
}


impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("any valid legacy ssb value")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        let value = wrap_unsafe_create(self.env, v, napi_get_boolean); 
        Ok(Value{env: self.env, value})
    }

    fn visit_f64<E: Error>(self, v: f64) -> Result<Self::Value, E> {
        match LegacyF64::from_f64(v) {
            Some(_) => {
                let value = wrap_unsafe_create(self.env, v, napi_create_double);
                Ok(Value{env: self.env, value})
            },
            None => Err(E::custom("invalid float"))
        }
    }
//
//    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
//        self.visit_string(v.to_string())
//    }
//
//    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
//        Ok(Value::String(v))
//    }
//
    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        let val = get_null_value(self.env); 
        Ok(Value{env: self.env, value: val})
    }
//
//    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
//        // use the size hint, but put a maximum to the allocation because we can't trust the input
//        let mut v = Vec::with_capacity(std::cmp::min(seq.size_hint().unwrap_or(0), MAX_ALLOC));
//
//        while let Some(inner) = seq.next_element()? {
//            v.push(inner);
//        }
//
//        Ok(Value::Array(v))
//    }
//
//    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
//        // use the size hint, but put a maximum to the allocation because we can't trust the input
//        let mut m = RidiculousStringMap::with_capacity(std::cmp::min(map.size_hint().unwrap_or(0),
//                                                         MAX_ALLOC));
//
//        while let Some((key, val)) = map.next_entry()? {
//            if let Some(_) = m.insert(key, val) {
//                return Err(A::Error::custom("map had duplicate key"));
//            }
//        }
//
//        Ok(Value::Object(m))
//    }
}
