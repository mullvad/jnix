# jnix

This crate provides high-level extensions to help with the usage of [JNI] in Rust code. Internally,
it uses the [`jni-rs`] crate for the low-level JNI operations.

Some helper traits are provided, such as:

- [`AsJValue`]: for allowing a JNI type to be convected to a `JValue` wrapper type.
- [`IntoJava`]: for allowing a Rust type to be converted to a Java type.
- [`FromJava`]: for allowing a Rust type to be created from a Java type.

A [`JnixEnv`] helper type is also provided, which is a [`JNIEnv`] wrapper that contains an
internal class cache for preloaded classes.

If compiled with the `derive` feature flag, the crate also exports procedural macros to
[derive `IntoJava`] and to [derive `FromJava`], which makes writing conversion code a lot
easier.  An example would be:

```rust
use jnix::{
    jni::{objects::JObject, JNIEnv},
    JnixEnv, FromJava, IntoJava,
};

// Rust type definition
#[derive(Default, FromJava, IntoJava)]
#[jnix(package = "my.package")]
pub struct MyData {
    number: i32,
    string: String,
}

// A JNI function called from Java that creates a `MyData` Rust type, converts it to a Java
// type and returns it.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_my_package_JniClass_getData<'env>(
    env: JNIEnv<'env>,
    _this: JObject<'env>,
    data: JObject<'env>,
) -> JObject<'env> {
    // Create the `JnixEnv` wrapper
    let env = JnixEnv::from(env);

    // Convert parameter to Rust type
    let data = MyData::from_java(&env, data);

    // Create a new `MyData` object by converting from the Rust type. Since a smart pointer is
    // returned from `into_java`, the inner object must be "leaked" sothat the garbage collector
    // can own it afterwards
    data.into_java(&env).forget()
}
```

```java
package my.package;

public class MyData {
    public MyData(int number, String string) {
        // This is the constructor that is called by the generated `IntoJava` code
        //
        // Note that the fields don't actually have to exist, the only thing that's necessary
        // is for the target Java class to have a constructor with the expected type signature
        // following the field order of the Rust type.
    }

    // These getters are called by the generated `FromJava` code
    public int getNumber() {
        return 10;
    }

    public String getString() {
        return "string value";
    }
}
```

[JNI]: https://en.wikipedia.org./wiki/Java_Native_Interface
[`jni-rs`]: https://crates.io/crates/jni
[`JNIEnv`]: https://docs.rs/jni/0.14.0/jni/struct.JNIEnv.html
[`AsJValue`]: https://docs.rs/jnix/0.4.0/jnix/as_jvalue/trait.AsJValue.html
[`IntoJava`]: https://docs.rs/jnix/0.4.0/jnix/into_java/trait.IntoJava.html
[`FromJava`]: https://docs.rs/jnix/0.4.0/jnix/from_java/trait.FromJava.html
[`JnixEnv`]: https://docs.rs/jnix/0.4.0/jnix/jnix_env/struct.JnixEnv.html
[derive `IntoJava`]: https://docs.rs/jnix-macros/0.4.0/jnix_macros/derive.IntoJava.html
[derive `FromJava`]: https://docs.rs/jnix-macros/0.4.0/jnix_macros/derive.FromJava.html

License: Apache-2.0 OR MIT
