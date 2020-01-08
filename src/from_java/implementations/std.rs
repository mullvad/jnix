use crate::{FromJava, JnixEnv};
use jni::objects::{AutoLocal, JObject, JValue};

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
