use crate::{IntoJava, JnixEnv};
use jni::sys::{jboolean, jshort, JNI_FALSE, JNI_TRUE};

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

impl<'borrow, 'env: 'borrow> IntoJava<'borrow, 'env> for i16 {
    const JNI_SIGNATURE: &'static str = "S";

    type JavaType = jshort;

    fn into_java(self, _: &'borrow JnixEnv<'env>) -> Self::JavaType {
        self as jshort
    }
}
