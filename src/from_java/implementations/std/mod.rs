mod net;

use crate::{FromJava, JnixEnv};
use jni::{
    objects::{AutoLocal, JObject, JString, JValue},
    signature::{JavaType, Primitive},
    sys::{jboolean, jint, jlong, jshort, JNI_FALSE},
};
use std::collections::HashSet;
use std::iter::FromIterator;

impl<'env, 'sub_env, T> FromJava<'env, JValue<'sub_env>> for T
where
    'env: 'sub_env,
    T: FromJava<'env, JObject<'sub_env>>,
{
    const JNI_SIGNATURE: &'static str = T::JNI_SIGNATURE;

    fn from_java(env: &JnixEnv<'env>, source: JValue<'sub_env>) -> Self {
        match source {
            JValue::Object(object) => T::from_java(env, object),
            _ => panic!(
                "Can't convert non-object Java type. Expected type signature {}",
                Self::JNI_SIGNATURE
            ),
        }
    }
}

impl<'env, 'sub_env, 'borrow, T> FromJava<'env, AutoLocal<'sub_env, 'borrow>> for T
where
    'env: 'sub_env,
    'sub_env: 'borrow,
    T: for<'inner_borrow> FromJava<'env, JObject<'inner_borrow>>,
{
    const JNI_SIGNATURE: &'static str = T::JNI_SIGNATURE;

    fn from_java(env: &JnixEnv<'env>, source: AutoLocal<'sub_env, 'borrow>) -> Self {
        T::from_java(env, source.as_obj())
    }
}

impl<'env> FromJava<'env, jboolean> for bool {
    const JNI_SIGNATURE: &'static str = "Z";

    fn from_java(_: &JnixEnv<'env>, source: jboolean) -> Self {
        source != JNI_FALSE
    }
}

impl<'env, 'sub_env> FromJava<'env, JValue<'sub_env>> for bool
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Z";

    fn from_java(env: &JnixEnv<'env>, source: JValue<'sub_env>) -> Self {
        match source {
            JValue::Bool(boolean) => bool::from_java(env, boolean),
            _ => panic!("Can't convert Java type, expected a boolean primitive"),
        }
    }
}

impl<'env> FromJava<'env, jlong> for i64 {
    const JNI_SIGNATURE: &'static str = "J";

    fn from_java(_: &JnixEnv<'env>, source: jlong) -> Self {
        source as i64
    }
}

impl<'env, 'sub_env> FromJava<'env, JValue<'sub_env>> for i64
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "J";

    fn from_java(env: &JnixEnv<'env>, source: JValue<'sub_env>) -> Self {
        match source {
            JValue::Long(long) => i64::from_java(env, long),
            _ => panic!("Can't convert Java type, expected a long primitive"),
        }
    }
}

impl<'env> FromJava<'env, jint> for i32 {
    const JNI_SIGNATURE: &'static str = "I";

    fn from_java(_: &JnixEnv<'env>, source: jint) -> Self {
        source as i32
    }
}

impl<'env, 'sub_env> FromJava<'env, JValue<'sub_env>> for i32
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "I";

    fn from_java(env: &JnixEnv<'env>, source: JValue<'sub_env>) -> Self {
        match source {
            JValue::Int(integer) => i32::from_java(env, integer),
            _ => panic!("Can't convert Java type, expected an integer primitive"),
        }
    }
}

impl<'env> FromJava<'env, jshort> for i16 {
    const JNI_SIGNATURE: &'static str = "S";

    fn from_java(_: &JnixEnv<'env>, source: jshort) -> Self {
        source as i16
    }
}

impl<'env, 'sub_env> FromJava<'env, JValue<'sub_env>> for i16
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "S";

    fn from_java(env: &JnixEnv<'env>, source: JValue<'sub_env>) -> Self {
        match source {
            JValue::Short(short) => i16::from_java(env, short),
            _ => panic!("Can't convert Java type, expected a short primitive"),
        }
    }
}

impl<'env, 'sub_env> FromJava<'env, JString<'sub_env>> for String
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/lang/String;";

    fn from_java(env: &JnixEnv<'env>, source: JString<'sub_env>) -> Self {
        String::from(
            env.get_string(source)
                .expect("Failed to convert from Java String"),
        )
    }
}

impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for String
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/lang/String;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        String::from_java(env, JString::from(source))
    }
}

impl<'env, 'sub_env, T> FromJava<'env, JObject<'sub_env>> for Option<T>
where
    'env: 'sub_env,
    T: FromJava<'env, JObject<'sub_env>>,
{
    const JNI_SIGNATURE: &'static str = T::JNI_SIGNATURE;

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        if source.is_null() {
            None
        } else {
            Some(T::from_java(env, source))
        }
    }
}

impl<'env, 'sub_env, T> FromJava<'env, JString<'sub_env>> for Option<T>
where
    'env: 'sub_env,
    T: FromJava<'env, JString<'sub_env>>,
{
    const JNI_SIGNATURE: &'static str = T::JNI_SIGNATURE;

    fn from_java(env: &JnixEnv<'env>, source: JString<'sub_env>) -> Self {
        if source.is_null() {
            None
        } else {
            Some(T::from_java(env, source))
        }
    }
}

impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for Option<i32> {
    const JNI_SIGNATURE: &'static str = "Ljava/lang/Integer;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        if source.is_null() {
            None
        } else {
            let class = env.get_class("java/lang/Integer");
            let method_id = env
                .get_method_id(&class, "intValue", "()I")
                .expect("Failed to get method ID for Integer.intValue()");
            let return_type = JavaType::Primitive(Primitive::Int);

            let int_value = env
                .call_method_unchecked(source, method_id, return_type, &[])
                .expect("Failed to call Integer.intValue()")
                .i()
                .expect("Call to Integer.intValue() did not return an int primitive");

            Some(int_value)
        }
    }
}

impl<'env, 'sub_env, T> FromJava<'env, JObject<'sub_env>> for Vec<T>
where
    'env: 'sub_env,
    T: FromJava<'env, JObject<'sub_env>>,
{
    const JNI_SIGNATURE: &'static str = "Ljava/util/ArrayList;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        let class = env.get_class("java/util/ArrayList");
        let size_method_id = env
            .get_method_id(&class, "size", "()I")
            .expect("Failed to get method ID for ArrayList.size()");
        let size_return_type = JavaType::Primitive(Primitive::Int);

        let item_count = env
            .call_method_unchecked(source, size_method_id, size_return_type, &[])
            .expect("Failed to call ArrayList.size()")
            .i()
            .expect("Call to ArrayList.size() did not return an int primitive");

        let mut target = Vec::with_capacity(item_count as usize);

        let get_method_id = env
            .get_method_id(&class, "get", "(I)Ljava/lang/Object;")
            .expect("Failed to get method ID for ArrayList.get()");
        let get_return_type = JavaType::Object("java/lang/Object".to_owned());

        for index in 0..item_count {
            let object = env
                .call_method_unchecked(
                    source,
                    get_method_id,
                    get_return_type.clone(),
                    &[JValue::Int(index)],
                )
                .expect("Failed to call ArrayList.get()")
                .l()
                .expect("Call to ArrayList.get() did not return an object");
            let item = T::from_java(env, object);

            target.push(item);
        }

        target
    }
}

impl<'env, 'sub_env, T: Eq + std::hash::Hash> FromJava<'env, JObject<'sub_env>> for HashSet<T>
where
    'env: 'sub_env,
    T: FromJava<'env, JObject<'sub_env>>,
{
    const JNI_SIGNATURE: &'static str = "Ljava/util/HashSet;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        let class = env.get_class("java/util/ArrayList");

        let list_object = env
            .new_object(&class, "(Ljava/util/Collection;)V", &[JValue::from(source)])
            .expect("Failed to create ArrayList object from HashSet");

        let vector = Vec::from_java(&env, list_object);
        HashSet::from_iter(vector)
    }
}
