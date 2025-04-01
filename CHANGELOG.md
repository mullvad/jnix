# Changelog

This file lists the meaningful changes between released versions.

## Format

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

Entries should have the imperative form, just like commit messages. Start each entry with words like
add, fix, increase, force, etc.. Not added, fixed, increased, forced, etc..

Line wrap the file at 100 characters. That is over here: ------------------------------------------|

### Categories each change falls into

- **Added**: for new features.
- **Changed**: for changes in existing functionality.
- **Deprecated**: for soon-to-be removed features.
- **Removed**: for now removed features.
- **Fixed**: for any bug fixes.
- **Security**: in case of vulnerabilities.

## [Unreleased]

## [0.5.3] - 2025-04-01
### Added
- Implement `FromJava` for `HashMap<K, V>`.

## [0.5.2] - 2025-01-30
### Added
- Implement `FromJava` for `i64`.
- Implement `FromJava` for `i16`.

## [0.5.1] - 2023-10-24
### Fixed
- Fix `FromJava` derivation not working when a field is named "source".

### Added
- Implement `FromJava` for `HashSet<T>`.
- Implement `IntoJava` for `HashSet<T>`.

## [0.5.0] - 2022-09-14
### Fixed
- Fix broken derivation of `FromJava` for enums with only unit variants.

### Changed
- Upgrade `jni` to `0.19`.

## [0.4.0] - 2021-02-17
### Added
- Allow using a `#[jnix(bounds = "T: my.package.MyClass")]` attribute to specify the underlying
  erased type used for a generic type parameter.

### Changed
- Derivation of `FromJava` for unnamed fields (i.e., tuple structs and tuple variants) now assumes
  that each field with index N has a `component{N+1}` getter method instead of the previously
  assumed `get{N}` method. This makes it easier to interface with Kotlin data classes, because it
  automatically generates these `component{N+1}` getter methods for a `data class`.
- Derivation of `IntoJava` for the unit variants of non-generic enums (i.e., the variants that have
  no fields) now assume that the Java representation is a singleton class that has its instance
  stored in a `INSTANCE` field. This allows Kotlin `object` variants to be used as the sub-classes
  to represent the variants in a `sealed class`. Note that this does not change the assumption that
  an enum that only has unit variants is represented as an `enum class`.

## [0.3.0] - 2020-11-27
### Added
- Implement `FromJava` for `Option<i32>`.

### Changed
- Implementation of `FromJava<JValue>` for `i32` now expects an `int` Java primitive instead of a
  boxed `Integer` object, this means that when deriving `FromJava` for custom types, `i32` fields
  must now have a respective `int` field in the respective Java class. If `Integer` object fields
  are desired, the Rust field type should be `Option<i32>`.

### Removed
- Implementation of `FromJava<JObject>` for `i32`. If conversion from `Integer` objects is needed,
  it's possible to use `Option<i32>` as the target Rust type.

## [0.2.4] - 2020-11-17
### Added
- Implement `FromJava` for `Vec<T>`.
- Implement `FromJava` for `IpAddr`, `Ipv4Addr` and `Ipv6Addr`.
- Implement `FromJava<JValue>` for `bool` so that it can be used in structs that derive `FromJava`.

## [0.2.3] - 2020-05-07
### Added
- Implement `IntoJava` for `i64`.

## [0.2.2] - 2020-03-23
### Added
- Implement `FromJava<JObject>` for `i32` to convert from a boxed `Integer` object.
- Implement `IntoJava` for `Option<i32>` to convert to a boxed `Integer` object.

## [0.2.1] - 2020-03-10
### Added
- Implement `IntoJava` for `Option<bool>` to convert to a boxed `Boolean` object.

## [0.2.0] - 2020-02-05
### Added
- Added `FromJava` trait.
- Added derive macro for `FromJava`.

## [0.1.2] - 2020-01-22
### Fixed
- Fix another instance of a local reference leak when calling `.into_java(env)` on an `IpAddr`,
  `Ipv4Addr` or `Ipv6Addr`.

## [0.1.1] - 2020-01-15
### Fixed
- Fix skipping fields in tuple structs and tuple enum variants when deriving `IntoJava`.
- Fix local reference leak when calling `.into_java(env)` on an `IpAddr`, `Ipv4Addr` or `Ipv6Addr`.
