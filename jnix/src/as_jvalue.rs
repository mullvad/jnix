use jni::objects::{AutoLocal, JValue};

/// Returns a value as it's [`JValue`] representation.
///
/// [`JValue`]: https://docs.rs/jni/0.14.0/jni/objects/enum.JValue.html
pub trait AsJValue<'env> {
    /// Returns the [`JValue`] representation of the type.
    ///
    /// [`JValue`]: https://docs.rs/jni/0.14.0/jni/objects/enum.JValue.html
    fn as_jvalue<'borrow>(&'borrow self) -> JValue<'borrow>
    where
        'env: 'borrow;
}

impl<'env_borrow, 'env: 'env_borrow> AsJValue<'env> for AutoLocal<'env, 'env_borrow> {
    fn as_jvalue<'borrow>(&'borrow self) -> JValue<'borrow>
    where
        'env: 'borrow,
    {
        JValue::Object(self.as_obj())
    }
}
