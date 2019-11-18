use crate::{IntoJava, JnixEnv};
use jni::sys::{jboolean, JNI_FALSE, JNI_TRUE};

impl<'borrow, 'env: 'borrow> IntoJava<'borrow, 'env> for bool {
    const JNI_SIGNATURE: &'static str = "Z";

    type JavaType = jboolean;

    fn into_java(self, _: &'borrow JnixEnv<'env>) -> Self::JavaType {
        if self {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    }
}
