use crate::{FromJava, JnixEnv};
use jni::{
    objects::JObject,
    signature::{JavaType, Primitive},
};
use std::net::IpAddr;

impl<'env, 'sub_env> FromJava<'env, JObject<'sub_env>> for IpAddr
where
    'env: 'sub_env,
{
    const JNI_SIGNATURE: &'static str = "Ljava/net/InetAddress;";

    fn from_java(env: &JnixEnv<'env>, source: JObject<'sub_env>) -> Self {
        let class = env.get_class("java/net/InetAddress");
        let method_id = env
            .get_method_id(&class, "getAddress", "()[B")
            .expect("Failed to get method ID for InetAddress.getAddress()");
        let return_type = JavaType::Array(Box::new(JavaType::Primitive(Primitive::Byte)));

        let octets_object = env
            .call_method_unchecked(source, method_id, return_type, &[])
            .expect("Failed to call InetAddress.getAddress()")
            .l()
            .expect("Call to InetAddress.getAddress() did not return an object");
        let octet_count = env
            .get_array_length(octets_object.into_inner())
            .expect("Failed to get length of byte array returned by InetAddress.getAddress()");

        match octet_count {
            4 => {
                let mut octets = [0u8; 4];
                let mut signed_octets = [0i8; 4];

                env.get_byte_array_region(octets_object.into_inner(), 0, &mut signed_octets)
                    .expect("Failed to read 4 octets returned by InetAddress.getAddress()");

                for index in 0..4 {
                    octets[index] = signed_octets[index] as u8;
                }

                IpAddr::from(octets)
            }
            16 => {
                let mut octets = [0u8; 16];
                let mut signed_octets = [0i8; 16];

                env.get_byte_array_region(octets_object.into_inner(), 0, &mut signed_octets)
                    .expect("Failed to read 16 octets returned by InetAddress.getAddress()");

                for index in 0..16 {
                    octets[index] = signed_octets[index] as u8;
                }

                IpAddr::from(octets)
            }
            count => panic!(
                "Invalid number of octets returned by InetAddress.getAddress(): {}",
                count
            ),
        }
    }
}
