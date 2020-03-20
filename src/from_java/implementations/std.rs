use crate::{FromJava, JnixEnv};
use jni::{
    objects::{AutoLocal, JObject, JString, JValue},
    signature::{JavaType, Primitive},
    sys::{jboolean, JNI_FALSE},
};

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

impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for i32
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/lang/Integer;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        let class = env.get_class("java/lang/Integer");
        let method_id = env
            .get_method_id(&class, "intValue", "()I")
            .expect("Failed to get method ID for Integer.intValue()");
        let return_type = JavaType::Primitive(Primitive::Int);

        env.call_method_unchecked(source, method_id, return_type, &[])
            .expect("Failed to call Integer.intValue()")
            .i()
            .expect("Call to Integer.intValue() did not return an int primitive")
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
