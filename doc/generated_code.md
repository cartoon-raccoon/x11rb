# Examples of the generated code

This crate uses a code generator to generate Rust code for the X11 protocol. You
might be curious what the generated code looks like. This document is there to
answer this question.

There are two crates involved:

- x11rb-protocol will contain most of the resulting code and all examples below
  are for this crate, unless explicitly mentioned
- x11rb contains some helper functions to simplify request sending

The code in x11rb-async is similar to what is generated for x11rb. Hence, it is
not shown here.

As you may know, the code generator uses an XML description of the X11 protocol
from `xcb-proto`. This document will show some examples of the XML description
followed by the Rust code that is generated for it.

The following code is generated at the beginning of a module (example for
`xproto` in x11rb-protocol; other modules and x11rb have some slight differences):
```rust
// This file contains generated code. Do not edit directly.
// To regenerate this, run 'make'.

//! Bindings to the core X11 protocol.
//!
//! For more documentation on the X11 protocol, see the
//! [protocol reference manual](https://www.x.org/releases/X11R7.6/doc/xproto/x11protocol.html).
//! This is especially recommended for looking up the exact semantics of
//! specific errors, events, or requests.

#![allow(clippy::too_many_arguments)]
// The code generator is simpler if it can always use conversions
#![allow(clippy::useless_conversion)]

#[allow(unused_imports)]
use alloc::borrow::Cow;
#[allow(unused_imports)]
use core::convert::TryInto;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryFrom;
use crate::errors::ParseError;
#[allow(unused_imports)]
use crate::x11_utils::TryIntoUSize;
use crate::BufWithFds;
#[allow(unused_imports)]
use crate::utils::{RawFdContainer, pretty_print_bitmask, pretty_print_enum};
#[allow(unused_imports)]
use crate::x11_utils::{Request, RequestHeader, Serialize, TryParse, TryParseFd};
```

## XID types

XIDs simply are numbers. They uniquely identify, for example, a window.

```xml
<xidtype name="WINDOW" />
```

Since all XIDs are 32 bit numbers, the generated code is a type alias:

```rust
pub type Window = u32;
```

## Structs

### Fixed length structs

Structs are used as building blocks for other things. For example, a request
could send a list of structs to the X11 server.
```xml
<struct name="POINT">
  <field type="INT16" name="x" />
  <field type="INT16" name="y" />
</struct>
```
The server can send structs to us. For this reason, there is a `TryParse` trait
that is implemented on structs. This trait is used by the generated code, for
example to parse a list of `Point`s.

We must also be able to send structs to the server. This is handled through the
`Serialize` trait that produces data in the native endian.

```rust
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    pub x: i16,
    pub y: i16,
}
impl_debug_if_no_extra_traits!(Point, "Point");
impl TryParse for Point {
    fn try_parse(remaining: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let (x, remaining) = i16::try_parse(remaining)?;
        let (y, remaining) = i16::try_parse(remaining)?;
        let result = Point { x, y };
        Ok((result, remaining))
    }
}
impl Serialize for Point {
    type Bytes = [u8; 4];
    fn serialize(&self) -> [u8; 4] {
        let x_bytes = self.x.serialize();
        let y_bytes = self.y.serialize();
        [
            x_bytes[0],
            x_bytes[1],
            y_bytes[0],
            y_bytes[1],
        ]
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        bytes.reserve(4);
        self.x.serialize_into(bytes);
        self.y.serialize_into(bytes);
    }
}
```

### Variable length structs

Structs can be a bit more complicated than the above example, because they can
contain lists of other structs. This example demonstrates this.
```xml
<struct name="DEPTH">
  <field type="CARD8" name="depth" />
  <pad bytes="1" />
  <field type="CARD16" name="visuals_len" />
  <pad bytes="4" />
  <list type="VISUALTYPE" name="visuals">
    <fieldref>visuals_len</fieldref>
  </list>
</struct>
```
The field `visuals_len` is not part of the generated struct since it is
represented implicitly as the length of the `visuals` `Vec`. To make this less
confusing, a function `visuals_len` is generated.
```rust
#[derive(Clone, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Depth {
    pub depth: u8,
    pub visuals: Vec<Visualtype>,
}
impl_debug_if_no_extra_traits!(Depth, "Depth");
impl TryParse for Depth {
    fn try_parse(remaining: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let (depth, remaining) = u8::try_parse(remaining)?;
        let remaining = remaining.get(1..).ok_or(ParseError::InsufficientData)?;
        let (visuals_len, remaining) = u16::try_parse(remaining)?;
        let remaining = remaining.get(4..).ok_or(ParseError::InsufficientData)?;
        let (visuals, remaining) = crate::x11_utils::parse_list::<Visualtype>(remaining, visuals_len.try_to_usize()?)?;
        let result = Depth { depth, visuals };
        Ok((result, remaining))
    }
}
impl Serialize for Depth {
    type Bytes = Vec<u8>;
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        self.serialize_into(&mut result);
        result
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        bytes.reserve(8);
        self.depth.serialize_into(bytes);
        bytes.extend_from_slice(&[0; 1]);
        let visuals_len = u16::try_from(self.visuals.len()).expect("`visuals` has too many elements");
        visuals_len.serialize_into(bytes);
        bytes.extend_from_slice(&[0; 4]);
        self.visuals.serialize_into(bytes);
    }
}
impl Depth {
    /// Get the value of the `visuals_len` field.
    ///
    /// The `visuals_len` field is used as the length field of the `visuals` field.
    /// This function computes the field's value again based on the length of the list.
    ///
    /// # Panics
    ///
    /// Panics if the value cannot be represented in the target type. This
    /// cannot happen with values of the struct received from the X11 server.
    pub fn visuals_len(&self) -> u16 {
        self.visuals.len()
            .try_into().unwrap()
    }
}
```

## Enumerations

Enumerations have a set of defined values, similar to rust `enum`s. However,
they are not represented as `enum`s since there are cases where the X11 server
violates the X11 protocol. These cases previously caused `ParseError`. Thus,
enumerations are now represented as a newtype around numbers.

### 'Real' enumerations

```xml
<enum name="BackingStore">
  <item name="NotUseful"> <value>0</value></item>
  <item name="WhenMapped"><value>1</value></item>
  <item name="Always">    <value>2</value></item>
</enum>
```
Depending on the largest value, appropriate `From` and `TryFrom` implementations
are generated.

```rust
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackingStore(u32);
impl BackingStore {
    pub const NOT_USEFUL: Self = Self(0);
    pub const WHEN_MAPPED: Self = Self(1);
    pub const ALWAYS: Self = Self(2);
}
impl From<BackingStore> for u32 {
    #[inline]
    fn from(input: BackingStore) -> Self {
        input.0
    }
}
impl From<BackingStore> for Option<u32> {
    #[inline]
    fn from(input: BackingStore) -> Self {
        Some(input.0)
    }
}
impl From<u8> for BackingStore {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value.into())
    }
}
impl From<u16> for BackingStore {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value.into())
    }
}
impl From<u32> for BackingStore {
    #[inline]
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl core::fmt::Debug for BackingStore  {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let variants = [
            (Self::NOT_USEFUL.0, "NOT_USEFUL", "NotUseful"),
            (Self::WHEN_MAPPED.0, "WHEN_MAPPED", "WhenMapped"),
            (Self::ALWAYS.0, "ALWAYS", "Always"),
        ];
        pretty_print_enum(fmt, self.0, &variants)
    }
}
```

### Bitmask enumerations

Bitmasks also get an invocation of the `bitmask_binop!` macro. This creates
implementations of `BitOr` and `BitOrAssign`.
```xml
<enum name="ConfigWindow">
  <item name="X">          <bit>0</bit></item>
  <item name="Y">          <bit>1</bit></item>
  <item name="Width">      <bit>2</bit></item>
  <item name="Height">     <bit>3</bit></item>
  <item name="BorderWidth"><bit>4</bit></item>
  <item name="Sibling">    <bit>5</bit></item>
  <item name="StackMode">  <bit>6</bit></item>
</enum>
```

```rust
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigWindow(u16);
impl ConfigWindow {
    pub const X: Self = Self(1 << 0);
    pub const Y: Self = Self(1 << 1);
    pub const WIDTH: Self = Self(1 << 2);
    pub const HEIGHT: Self = Self(1 << 3);
    pub const BORDER_WIDTH: Self = Self(1 << 4);
    pub const SIBLING: Self = Self(1 << 5);
    pub const STACK_MODE: Self = Self(1 << 6);
}
impl From<ConfigWindow> for u16 {
    #[inline]
    fn from(input: ConfigWindow) -> Self {
        input.0
    }
}
impl From<ConfigWindow> for Option<u16> {
    #[inline]
    fn from(input: ConfigWindow) -> Self {
        Some(input.0)
    }
}
impl From<ConfigWindow> for u32 {
    #[inline]
    fn from(input: ConfigWindow) -> Self {
        u32::from(input.0)
    }
}
impl From<ConfigWindow> for Option<u32> {
    #[inline]
    fn from(input: ConfigWindow) -> Self {
        Some(u32::from(input.0))
    }
}
impl From<u8> for ConfigWindow {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value.into())
    }
}
impl From<u16> for ConfigWindow {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl core::fmt::Debug for ConfigWindow  {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let variants = [
            (Self::X.0.into(), "X", "X"),
            (Self::Y.0.into(), "Y", "Y"),
            (Self::WIDTH.0.into(), "WIDTH", "Width"),
            (Self::HEIGHT.0.into(), "HEIGHT", "Height"),
            (Self::BORDER_WIDTH.0.into(), "BORDER_WIDTH", "BorderWidth"),
            (Self::SIBLING.0.into(), "SIBLING", "Sibling"),
            (Self::STACK_MODE.0.into(), "STACK_MODE", "StackMode"),
        ];
        pretty_print_bitmask(fmt, self.0.into(), &variants)
    }
}
bitmask_binop!(ConfigWindow, u16);
```

## Unions

Sadly, there are unions in X11. In core X11, there is only one union:
ClientMessageData.

```xml
<union name="ClientMessageData">
  <!-- The format member of the ClientMessage event determines which array
       to use. -->
  <list type="CARD8"  name="data8" ><value>20</value></list> <!--  8 -->
  <list type="CARD16" name="data16"><value>10</value></list> <!-- 16 -->
  <list type="CARD32" name="data32"><value>5</value></list>  <!-- 32 -->
</union>
```

The union contains an array of unparsed data. The data is then parsed as the
requested type in the accessor functions.

```rust
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClientMessageData([u8; 20]);
impl ClientMessageData {
    pub fn as_data8(&self) -> [u8; 20] {
        fn do_the_parse(remaining: &[u8]) -> Result<[u8; 20], ParseError> {
            let (data8, remaining) = crate::x11_utils::parse_u8_array::<20>(remaining)?;
            let _ = remaining;
            Ok(data8)
        }
        do_the_parse(&self.0).unwrap()
    }
    pub fn as_data16(&self) -> [u16; 10] {
        fn do_the_parse(remaining: &[u8]) -> Result<[u16; 10], ParseError> {
            let (data16_0, remaining) = u16::try_parse(remaining)?;
            let (data16_1, remaining) = u16::try_parse(remaining)?;
            let (data16_2, remaining) = u16::try_parse(remaining)?;
            let (data16_3, remaining) = u16::try_parse(remaining)?;
            let (data16_4, remaining) = u16::try_parse(remaining)?;
            let (data16_5, remaining) = u16::try_parse(remaining)?;
            let (data16_6, remaining) = u16::try_parse(remaining)?;
            let (data16_7, remaining) = u16::try_parse(remaining)?;
            let (data16_8, remaining) = u16::try_parse(remaining)?;
            let (data16_9, remaining) = u16::try_parse(remaining)?;
            let data16 = [
                data16_0,
                data16_1,
                data16_2,
                data16_3,
                data16_4,
                data16_5,
                data16_6,
                data16_7,
                data16_8,
                data16_9,
            ];
            let _ = remaining;
            Ok(data16)
        }
        do_the_parse(&self.0).unwrap()
    }
    pub fn as_data32(&self) -> [u32; 5] {
        fn do_the_parse(remaining: &[u8]) -> Result<[u32; 5], ParseError> {
            let (data32_0, remaining) = u32::try_parse(remaining)?;
            let (data32_1, remaining) = u32::try_parse(remaining)?;
            let (data32_2, remaining) = u32::try_parse(remaining)?;
            let (data32_3, remaining) = u32::try_parse(remaining)?;
            let (data32_4, remaining) = u32::try_parse(remaining)?;
            let data32 = [
                data32_0,
                data32_1,
                data32_2,
                data32_3,
                data32_4,
            ];
            let _ = remaining;
            Ok(data32)
        }
        do_the_parse(&self.0).unwrap()
    }
}
impl Serialize for ClientMessageData {
    type Bytes = [u8; 20];
    fn serialize(&self) -> [u8; 20] {
        self.0
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.0);
    }
}
impl TryParse for ClientMessageData {
    fn try_parse(value: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let inner: [u8; 20] = value.get(..20)
            .ok_or(ParseError::InsufficientData)?
            .try_into()
            .unwrap();
        let result = ClientMessageData(inner);
        Ok((result, &value[20..]))
    }
}
impl From<[u8; 20]> for ClientMessageData {
    fn from(data8: [u8; 20]) -> Self {
        Self(data8)
    }
}
impl From<[u16; 10]> for ClientMessageData {
    fn from(data16: [u16; 10]) -> Self {
        let data16_0_bytes = data16[0].serialize();
        let data16_1_bytes = data16[1].serialize();
        let data16_2_bytes = data16[2].serialize();
        let data16_3_bytes = data16[3].serialize();
        let data16_4_bytes = data16[4].serialize();
        let data16_5_bytes = data16[5].serialize();
        let data16_6_bytes = data16[6].serialize();
        let data16_7_bytes = data16[7].serialize();
        let data16_8_bytes = data16[8].serialize();
        let data16_9_bytes = data16[9].serialize();
        let value = [
            data16_0_bytes[0],
            data16_0_bytes[1],
            data16_1_bytes[0],
            data16_1_bytes[1],
            data16_2_bytes[0],
            data16_2_bytes[1],
            data16_3_bytes[0],
            data16_3_bytes[1],
            data16_4_bytes[0],
            data16_4_bytes[1],
            data16_5_bytes[0],
            data16_5_bytes[1],
            data16_6_bytes[0],
            data16_6_bytes[1],
            data16_7_bytes[0],
            data16_7_bytes[1],
            data16_8_bytes[0],
            data16_8_bytes[1],
            data16_9_bytes[0],
            data16_9_bytes[1],
        ];
        Self(value)
    }
}
impl From<[u32; 5]> for ClientMessageData {
    fn from(data32: [u32; 5]) -> Self {
        let data32_0_bytes = data32[0].serialize();
        let data32_1_bytes = data32[1].serialize();
        let data32_2_bytes = data32[2].serialize();
        let data32_3_bytes = data32[3].serialize();
        let data32_4_bytes = data32[4].serialize();
        let value = [
            data32_0_bytes[0],
            data32_0_bytes[1],
            data32_0_bytes[2],
            data32_0_bytes[3],
            data32_1_bytes[0],
            data32_1_bytes[1],
            data32_1_bytes[2],
            data32_1_bytes[3],
            data32_2_bytes[0],
            data32_2_bytes[1],
            data32_2_bytes[2],
            data32_2_bytes[3],
            data32_3_bytes[0],
            data32_3_bytes[1],
            data32_3_bytes[2],
            data32_3_bytes[3],
            data32_4_bytes[0],
            data32_4_bytes[1],
            data32_4_bytes[2],
            data32_4_bytes[3],
        ];
        Self(value)
    }
}
```

## Events

```xml
<event name="KeyPress" number="2">
  <field type="KEYCODE" name="detail" />
  <field type="TIMESTAMP" name="time" />
  <field type="WINDOW" name="root" />
  <field type="WINDOW" name="event" />
  <field type="WINDOW" name="child" altenum="Window" />
  <field type="INT16" name="root_x" />
  <field type="INT16" name="root_y" />
  <field type="INT16" name="event_x" />
  <field type="INT16" name="event_y" />
  <field type="CARD16" name="state" mask="KeyButMask" />
  <field type="BOOL" name="same_screen" />
  <pad bytes="1" />
  <doc>[SNIP]</doc>
</event>
```

```rust
/// Opcode for the KeyPress event
pub const KEY_PRESS_EVENT: u8 = 2;
/// [SNIP]
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyPressEvent {
    pub response_type: u8,
    pub detail: Keycode,
    pub sequence: u16,
    pub time: Timestamp,
    pub root: Window,
    pub event: Window,
    pub child: Window,
    pub root_x: i16,
    pub root_y: i16,
    pub event_x: i16,
    pub event_y: i16,
    pub state: KeyButMask,
    pub same_screen: bool,
}
impl_debug_if_no_extra_traits!(KeyPressEvent, "KeyPressEvent");
impl TryParse for KeyPressEvent {
    fn try_parse(initial_value: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let remaining = initial_value;
        let (response_type, remaining) = u8::try_parse(remaining)?;
        let (detail, remaining) = Keycode::try_parse(remaining)?;
        let (sequence, remaining) = u16::try_parse(remaining)?;
        let (time, remaining) = Timestamp::try_parse(remaining)?;
        let (root, remaining) = Window::try_parse(remaining)?;
        let (event, remaining) = Window::try_parse(remaining)?;
        let (child, remaining) = Window::try_parse(remaining)?;
        let (root_x, remaining) = i16::try_parse(remaining)?;
        let (root_y, remaining) = i16::try_parse(remaining)?;
        let (event_x, remaining) = i16::try_parse(remaining)?;
        let (event_y, remaining) = i16::try_parse(remaining)?;
        let (state, remaining) = u16::try_parse(remaining)?;
        let (same_screen, remaining) = bool::try_parse(remaining)?;
        let remaining = remaining.get(1..).ok_or(ParseError::InsufficientData)?;
        let state = state.into();
        let result = KeyPressEvent { response_type, detail, sequence, time, root, event, child, root_x, root_y, event_x, event_y, state, same_screen };
        let _ = remaining;
        let remaining = initial_value.get(32..)
            .ok_or(ParseError::InsufficientData)?;
        Ok((result, remaining))
    }
}
impl Serialize for KeyPressEvent {
    type Bytes = [u8; 32];
    fn serialize(&self) -> [u8; 32] {
        let response_type_bytes = self.response_type.serialize();
        let detail_bytes = self.detail.serialize();
        let sequence_bytes = self.sequence.serialize();
        let time_bytes = self.time.serialize();
        let root_bytes = self.root.serialize();
        let event_bytes = self.event.serialize();
        let child_bytes = self.child.serialize();
        let root_x_bytes = self.root_x.serialize();
        let root_y_bytes = self.root_y.serialize();
        let event_x_bytes = self.event_x.serialize();
        let event_y_bytes = self.event_y.serialize();
        let state_bytes = u16::from(self.state).serialize();
        let same_screen_bytes = self.same_screen.serialize();
        [
            response_type_bytes[0],
            detail_bytes[0],
            sequence_bytes[0],
            sequence_bytes[1],
            time_bytes[0],
            time_bytes[1],
            time_bytes[2],
            time_bytes[3],
            root_bytes[0],
            root_bytes[1],
            root_bytes[2],
            root_bytes[3],
            event_bytes[0],
            event_bytes[1],
            event_bytes[2],
            event_bytes[3],
            child_bytes[0],
            child_bytes[1],
            child_bytes[2],
            child_bytes[3],
            root_x_bytes[0],
            root_x_bytes[1],
            root_y_bytes[0],
            root_y_bytes[1],
            event_x_bytes[0],
            event_x_bytes[1],
            event_y_bytes[0],
            event_y_bytes[1],
            state_bytes[0],
            state_bytes[1],
            same_screen_bytes[0],
            0,
        ]
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        bytes.reserve(32);
        self.response_type.serialize_into(bytes);
        self.detail.serialize_into(bytes);
        self.sequence.serialize_into(bytes);
        self.time.serialize_into(bytes);
        self.root.serialize_into(bytes);
        self.event.serialize_into(bytes);
        self.child.serialize_into(bytes);
        self.root_x.serialize_into(bytes);
        self.root_y.serialize_into(bytes);
        self.event_x.serialize_into(bytes);
        self.event_y.serialize_into(bytes);
        u16::from(self.state).serialize_into(bytes);
        self.same_screen.serialize_into(bytes);
        bytes.extend_from_slice(&[0; 1]);
    }
}
impl From<&KeyPressEvent> for [u8; 32] {
    fn from(input: &KeyPressEvent) -> Self {
        let response_type_bytes = input.response_type.serialize();
        let detail_bytes = input.detail.serialize();
        let sequence_bytes = input.sequence.serialize();
        let time_bytes = input.time.serialize();
        let root_bytes = input.root.serialize();
        let event_bytes = input.event.serialize();
        let child_bytes = input.child.serialize();
        let root_x_bytes = input.root_x.serialize();
        let root_y_bytes = input.root_y.serialize();
        let event_x_bytes = input.event_x.serialize();
        let event_y_bytes = input.event_y.serialize();
        let state_bytes = u16::from(input.state).serialize();
        let same_screen_bytes = input.same_screen.serialize();
        [
            response_type_bytes[0],
            detail_bytes[0],
            sequence_bytes[0],
            sequence_bytes[1],
            time_bytes[0],
            time_bytes[1],
            time_bytes[2],
            time_bytes[3],
            root_bytes[0],
            root_bytes[1],
            root_bytes[2],
            root_bytes[3],
            event_bytes[0],
            event_bytes[1],
            event_bytes[2],
            event_bytes[3],
            child_bytes[0],
            child_bytes[1],
            child_bytes[2],
            child_bytes[3],
            root_x_bytes[0],
            root_x_bytes[1],
            root_y_bytes[0],
            root_y_bytes[1],
            event_x_bytes[0],
            event_x_bytes[1],
            event_y_bytes[0],
            event_y_bytes[1],
            state_bytes[0],
            state_bytes[1],
            same_screen_bytes[0],
            0,
        ]
    }
}
impl From<KeyPressEvent> for [u8; 32] {
    fn from(input: KeyPressEvent) -> Self {
        Self::from(&input)
    }
}
```

## Errors

```xml
<error name="Request" number="1">
  <field type="CARD32" name="bad_value" />
  <field type="CARD16" name="minor_opcode" />
  <field type="CARD8" name="major_opcode" />
  <pad bytes="1" />
</error>
```

All X11 errors contain the same fields. Thus, we only need to remember which
number represents this error.
```rust
/// Opcode for the Request error
pub const REQUEST_ERROR: u8 = 1;
```
The actual representation of an X11 error can be found in
[`x11rb::x11_utils::X11Error`](../x11rb-protocol/src/x11_utils.rs).

## Requests

For requests, we generate an extension trait in x11rb. Individual requests are
available on this trait and as global functions. The generic structure looks
like this:
```rust
/// Extension trait defining the requests of this extension.
pub trait ConnectionExt: RequestConnection {
    // Following code examples are in here
}
impl<C: RequestConnection + ?Sized> ConnectionExt for C {}
```

### Request without a reply

```xml
<request name="NoOperation" opcode="127" />
```
The request is represented by a structure that contains all of the request's
fields. This `struct` can be constructed explicitly and then
`conn.send_trait_request_without_reply()` to an X11 server.

This code is generated in the module in `x11rb-protocol`:
```rust
/// Opcode for the NoOperation request
pub const NO_OPERATION_REQUEST: u8 = 127;
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NoOperationRequest;
impl_debug_if_no_extra_traits!(NoOperationRequest, "NoOperationRequest");
impl NoOperationRequest {
    /// Serialize this request into bytes for the provided connection
    pub fn serialize(self) -> BufWithFds<[Cow<'static, [u8]>; 1]> {
        let length_so_far = 0;
        let mut request0 = vec![
            NO_OPERATION_REQUEST,
            0,
            0,
            0,
        ];
        let length_so_far = length_so_far + request0.len();
        assert_eq!(length_so_far % 4, 0);
        let length = u16::try_from(length_so_far / 4).unwrap_or(0);
        request0[2..4].copy_from_slice(&length.to_ne_bytes());
        ([request0.into()], vec![])
    }
    /// Parse this request given its header, its body, and any fds that go along with it
    #[cfg(feature = "request-parsing")]
    pub fn try_parse_request(header: RequestHeader, value: &[u8]) -> Result<Self, ParseError> {
        if header.major_opcode != NO_OPERATION_REQUEST {
            return Err(ParseError::InvalidValue);
        }
        let remaining = &[header.minor_opcode];
        let remaining = remaining.get(1..).ok_or(ParseError::InsufficientData)?;
        let _ = remaining;
        let _ = value;
        Ok(NoOperationRequest
        )
    }
}
impl Request for NoOperationRequest {
    const EXTENSION_NAME: Option<&'static str> = None;

    fn serialize(self, _major_opcode: u8) -> BufWithFds<Vec<u8>> {
        let (bufs, fds) = self.serialize();
        // Flatten the buffers into a single vector
        let buf = bufs.iter().flat_map(|buf| buf.iter().copied()).collect();
        (buf, fds)
    }
}
impl crate::x11_utils::VoidRequest for NoOperationRequest {
}
```
In the `x11rb`-crate, a function for sending the request via a function call is
generated.
```rust
pub fn no_operation<Conn>(conn: &Conn) -> Result<VoidCookie<'_, Conn>, ConnectionError>
where
    Conn: RequestConnection + ?Sized,
{
    let request0 = NoOperationRequest;
    let (bytes, fds) = request0.serialize();
    let slices = [IoSlice::new(&bytes[0])];
    assert_eq!(slices.len(), bytes.len());
    conn.send_request_without_reply(&slices, fds)
}
```
The request sending function is also available on the extension trait:
```rust
    fn no_operation(&self) -> Result<VoidCookie<'_, Self>, ConnectionError>
    {
        no_operation(self)
    }
```

### Request with a reply

```xml
<request name="GetInputFocus" opcode="43">
  <reply>
    <field type="CARD8" name="revert_to" enum="InputFocus" />
    <field type="WINDOW" name="focus" altenum="InputFocus" />
  </reply>
</request>
```
There is again a structure generated in `x11rb-protocol` that represents the request:
```rust
/// Opcode for the GetInputFocus request
pub const GET_INPUT_FOCUS_REQUEST: u8 = 43;
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetInputFocusRequest;
impl_debug_if_no_extra_traits!(GetInputFocusRequest, "GetInputFocusRequest");
impl GetInputFocusRequest {
    /// Serialize this request into bytes for the provided connection
    pub fn serialize(self) -> BufWithFds<[Cow<'static, [u8]>; 1]> {
        let length_so_far = 0;
        let mut request0 = vec![
            GET_INPUT_FOCUS_REQUEST,
            0,
            0,
            0,
        ];
        let length_so_far = length_so_far + request0.len();
        assert_eq!(length_so_far % 4, 0);
        let length = u16::try_from(length_so_far / 4).unwrap_or(0);
        request0[2..4].copy_from_slice(&length.to_ne_bytes());
        ([request0.into()], vec![])
    }
    /// Parse this request given its header, its body, and any fds that go along with it
    #[cfg(feature = "request-parsing")]
    pub fn try_parse_request(header: RequestHeader, value: &[u8]) -> Result<Self, ParseError> {
        if header.major_opcode != GET_INPUT_FOCUS_REQUEST {
            return Err(ParseError::InvalidValue);
        }
        let remaining = &[header.minor_opcode];
        let remaining = remaining.get(1..).ok_or(ParseError::InsufficientData)?;
        let _ = remaining;
        let _ = value;
        Ok(GetInputFocusRequest
        )
    }
}
impl Request for GetInputFocusRequest {
    const EXTENSION_NAME: Option<&'static str> = None;

    fn serialize(self, _major_opcode: u8) -> BufWithFds<Vec<u8>> {
        let (bufs, fds) = self.serialize();
        // Flatten the buffers into a single vector
        let buf = bufs.iter().flat_map(|buf| buf.iter().copied()).collect();
        (buf, fds)
    }
}
impl crate::x11_utils::ReplyRequest for GetInputFocusRequest {
    type Reply = GetInputFocusReply;
}
```
The reply is handled similar to a `struct`:
```rust
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GetInputFocusReply {
    pub revert_to: InputFocus,
    pub sequence: u16,
    pub length: u32,
    pub focus: Window,
}
impl_debug_if_no_extra_traits!(GetInputFocusReply, "GetInputFocusReply");
impl TryParse for GetInputFocusReply {
    fn try_parse(initial_value: &[u8]) -> Result<(Self, &[u8]), ParseError> {
        let remaining = initial_value;
        let (response_type, remaining) = u8::try_parse(remaining)?;
        let (revert_to, remaining) = u8::try_parse(remaining)?;
        let (sequence, remaining) = u16::try_parse(remaining)?;
        let (length, remaining) = u32::try_parse(remaining)?;
        let (focus, remaining) = Window::try_parse(remaining)?;
        if response_type != 1 {
            return Err(ParseError::InvalidValue);
        }
        let revert_to = revert_to.into();
        let result = GetInputFocusReply { revert_to, sequence, length, focus };
        let _ = remaining;
        let remaining = initial_value.get(32 + length as usize * 4..)
            .ok_or(ParseError::InsufficientData)?;
        Ok((result, remaining))
    }
}
impl Serialize for GetInputFocusReply {
    type Bytes = [u8; 12];
    fn serialize(&self) -> [u8; 12] {
        let response_type_bytes = &[1];
        let revert_to_bytes = u8::from(self.revert_to).serialize();
        let sequence_bytes = self.sequence.serialize();
        let length_bytes = self.length.serialize();
        let focus_bytes = self.focus.serialize();
        [
            response_type_bytes[0],
            revert_to_bytes[0],
            sequence_bytes[0],
            sequence_bytes[1],
            length_bytes[0],
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
            focus_bytes[0],
            focus_bytes[1],
            focus_bytes[2],
            focus_bytes[3],
        ]
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>) {
        bytes.reserve(12);
        let response_type_bytes = &[1];
        bytes.push(response_type_bytes[0]);
        u8::from(self.revert_to).serialize_into(bytes);
        self.sequence.serialize_into(bytes);
        self.length.serialize_into(bytes);
        self.focus.serialize_into(bytes);
    }
}
```
In `x11rb`, there is a function to send the request:
```rust
pub fn get_input_focus<Conn>(conn: &Conn) -> Result<Cookie<'_, Conn, GetInputFocusReply>, ConnectionError>
where
    Conn: RequestConnection + ?Sized,
{
    let request0 = GetInputFocusRequest;
    let (bytes, fds) = request0.serialize();
    let slices = [IoSlice::new(&bytes[0])];
    assert_eq!(slices.len(), bytes.len());
    conn.send_request_with_reply(&slices, fds)
}
```
There is also a function for sending the request in the extension trait:
```rust
    fn get_input_focus(&self) -> Result<Cookie<'_, Self, GetInputFocusReply>, ConnectionError>
    {
        get_input_focus(self)
    }
```

### Requests with a switch

Some requests have optional fields. A bit in the request then indicates the
presence of this extra field. For this kind of requests, a helper structure is
generated.
```xml
<request name="CreateWindow" opcode="1">
  <field type="CARD8" name="depth" />
  <field type="WINDOW" name="wid" />
  <field type="WINDOW" name="parent" />
  <field type="INT16" name="x" />
  <field type="INT16" name="y" />
  <field type="CARD16" name="width" />
  <field type="CARD16" name="height" />
  <field type="CARD16" name="border_width" />
  <field type="CARD16" name="class" enum="WindowClass" />
  <field type="VISUALID" name="visual" />
  <field type="CARD32" name="value_mask" mask="CW" />
  <switch name="value_list">
      <fieldref>value_mask</fieldref>
      <bitcase>
        <enumref ref="CW">BackPixmap</enumref>
        <field type="PIXMAP" name="background_pixmap" altenum="BackPixmap"/>
      </bitcase>
      <bitcase>
        <enumref ref="CW">BackPixel</enumref>
        <field type="CARD32" name="background_pixel" />
      </bitcase>
      [SNIP - you get the idea]
  </switch>
  <doc>[SNIP]</doc>
</request>
```
The switch is represented via a helper struct:
```rust
/// Auxiliary and optional information for the `create_window` function
#[derive(Clone, Copy, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateWindowAux {
    pub background_pixmap: Option<Pixmap>,
    pub background_pixel: Option<u32>,
    [SNIP - you get the idea]
}
impl_debug_if_no_extra_traits!(CreateWindowAux, "CreateWindowAux");
impl CreateWindowAux {
    #[cfg_attr(not(feature = "request-parsing"), allow(dead_code))]
    fn try_parse(value: &[u8], value_mask: u32) -> Result<(Self, &[u8]), ParseError> {
        let switch_expr = u32::from(value_mask);
        let mut outer_remaining = value;
        let background_pixmap = if switch_expr & u32::from(CW::BACK_PIXMAP) != 0 {
            let remaining = outer_remaining;
            let (background_pixmap, remaining) = Pixmap::try_parse(remaining)?;
            outer_remaining = remaining;
            Some(background_pixmap)
        } else {
            None
        };
        let background_pixel = if switch_expr & u32::from(CW::BACK_PIXEL) != 0 {
            let remaining = outer_remaining;
            let (background_pixel, remaining) = u32::try_parse(remaining)?;
            outer_remaining = remaining;
            Some(background_pixel)
        } else {
            None
        };
        [SNIP - you get the idea]
        let result = CreateWindowAux { background_pixmap, background_pixel, border_pixmap, border_pixel, bit_gravity, win_gravity, backing_store, backing_planes, backing_pixel, override_redirect, save_under, event_mask, do_not_propogate_mask, colormap, cursor };
        Ok((result, outer_remaining))
    }
}
impl CreateWindowAux {
    #[allow(dead_code)]
    fn serialize(&self, value_mask: u32) -> Vec<u8> {
        let mut result = Vec::new();
        self.serialize_into(&mut result, u32::from(value_mask));
        result
    }
    fn serialize_into(&self, bytes: &mut Vec<u8>, value_mask: u32) {
        assert_eq!(self.switch_expr(), u32::from(value_mask), "switch `value_list` has an inconsistent discriminant");
        if let Some(background_pixmap) = self.background_pixmap {
            background_pixmap.serialize_into(bytes);
        }
        if let Some(background_pixel) = self.background_pixel {
            background_pixel.serialize_into(bytes);
        }
        [SNIP - you get the idea]
    }
}
impl CreateWindowAux {
    fn switch_expr(&self) -> u32 {
        let mut expr_value = 0;
        if self.background_pixmap.is_some() {
            expr_value |= u32::from(CW::BACK_PIXMAP);
        }
        if self.background_pixel.is_some() {
            expr_value |= u32::from(CW::BACK_PIXEL);
        }
        [SNIP - you get the idea]
        expr_value
    }
}
impl CreateWindowAux {
    /// Create a new instance with all fields unset / not present.
    pub fn new() -> Self {
        Default::default()
    }
    /// Set the `background_pixmap` field of this structure.
    #[must_use]
    pub fn background_pixmap<I>(mut self, value: I) -> Self where I: Into<Option<Pixmap>> {
        self.background_pixmap = value.into();
        self
    }
    /// Set the `background_pixel` field of this structure.
    #[must_use]
    pub fn background_pixel<I>(mut self, value: I) -> Self where I: Into<Option<u32>> {
        self.background_pixel = value.into();
        self
    }
    [SNIP - you get the idea]
}
```
This code is generated for the actual request:
```rust
/// Opcode for the CreateWindow request
pub const CREATE_WINDOW_REQUEST: u8 = 1;
/// [SNIP]
#[derive(Clone, Default)]
#[cfg_attr(feature = "extra-traits", derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateWindowRequest<'input> {
    pub depth: u8,
    pub wid: Window,
    pub parent: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
    pub class: WindowClass,
    pub visual: Visualid,
    pub value_list: Cow<'input, CreateWindowAux>,
}
impl_debug_if_no_extra_traits!(CreateWindowRequest<'_>, "CreateWindowRequest");
impl<'input> CreateWindowRequest<'input> {
    /// Serialize this request into bytes for the provided connection
    pub fn serialize(self) -> BufWithFds<[Cow<'input, [u8]>; 3]> {
        let length_so_far = 0;
        let depth_bytes = self.depth.serialize();
        let wid_bytes = self.wid.serialize();
        let parent_bytes = self.parent.serialize();
        let x_bytes = self.x.serialize();
        let y_bytes = self.y.serialize();
        let width_bytes = self.width.serialize();
        let height_bytes = self.height.serialize();
        let border_width_bytes = self.border_width.serialize();
        let class_bytes = u16::from(self.class).serialize();
        let visual_bytes = self.visual.serialize();
        let value_mask: u32 = self.value_list.switch_expr();
        let value_mask_bytes = value_mask.serialize();
        let mut request0 = vec![
            CREATE_WINDOW_REQUEST,
            depth_bytes[0],
            0,
            0,
            wid_bytes[0],
            wid_bytes[1],
            wid_bytes[2],
            wid_bytes[3],
            parent_bytes[0],
            parent_bytes[1],
            parent_bytes[2],
            parent_bytes[3],
            x_bytes[0],
            x_bytes[1],
            y_bytes[0],
            y_bytes[1],
            width_bytes[0],
            width_bytes[1],
            height_bytes[0],
            height_bytes[1],
            border_width_bytes[0],
            border_width_bytes[1],
            class_bytes[0],
            class_bytes[1],
            visual_bytes[0],
            visual_bytes[1],
            visual_bytes[2],
            visual_bytes[3],
            value_mask_bytes[0],
            value_mask_bytes[1],
            value_mask_bytes[2],
            value_mask_bytes[3],
        ];
        let length_so_far = length_so_far + request0.len();
        let value_list_bytes = self.value_list.serialize(u32::from(value_mask));
        let length_so_far = length_so_far + value_list_bytes.len();
        let padding0 = &[0; 3][..(4 - (length_so_far % 4)) % 4];
        let length_so_far = length_so_far + padding0.len();
        assert_eq!(length_so_far % 4, 0);
        let length = u16::try_from(length_so_far / 4).unwrap_or(0);
        request0[2..4].copy_from_slice(&length.to_ne_bytes());
        ([request0.into(), value_list_bytes.into(), padding0.into()], vec![])
    }
    /// Parse this request given its header, its body, and any fds that go along with it
    #[cfg(feature = "request-parsing")]
    pub fn try_parse_request(header: RequestHeader, value: &'input [u8]) -> Result<Self, ParseError> {
        if header.major_opcode != CREATE_WINDOW_REQUEST {
            return Err(ParseError::InvalidValue);
        }
        let remaining = &[header.minor_opcode];
        let (depth, remaining) = u8::try_parse(remaining)?;
        let _ = remaining;
        let (wid, remaining) = Window::try_parse(value)?;
        let (parent, remaining) = Window::try_parse(remaining)?;
        let (x, remaining) = i16::try_parse(remaining)?;
        let (y, remaining) = i16::try_parse(remaining)?;
        let (width, remaining) = u16::try_parse(remaining)?;
        let (height, remaining) = u16::try_parse(remaining)?;
        let (border_width, remaining) = u16::try_parse(remaining)?;
        let (class, remaining) = u16::try_parse(remaining)?;
        let class = class.into();
        let (visual, remaining) = Visualid::try_parse(remaining)?;
        let (value_mask, remaining) = u32::try_parse(remaining)?;
        let (value_list, remaining) = CreateWindowAux::try_parse(remaining, u32::from(value_mask))?;
        let _ = remaining;
        Ok(CreateWindowRequest {
            depth,
            wid,
            parent,
            x,
            y,
            width,
            height,
            border_width,
            class,
            visual,
            value_list: Cow::Owned(value_list),
        })
    }
    /// Clone all borrowed data in this CreateWindowRequest.
    pub fn into_owned(self) -> CreateWindowRequest<'static> {
        CreateWindowRequest {
            depth: self.depth,
            wid: self.wid,
            parent: self.parent,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            border_width: self.border_width,
            class: self.class,
            visual: self.visual,
            value_list: Cow::Owned(self.value_list.into_owned()),
        }
    }
}
impl<'input> Request for CreateWindowRequest<'input> {
    const EXTENSION_NAME: Option<&'static str> = None;

    fn serialize(self, _major_opcode: u8) -> BufWithFds<Vec<u8>> {
        let (bufs, fds) = self.serialize();
        // Flatten the buffers into a single vector
        let buf = bufs.iter().flat_map(|buf| buf.iter().copied()).collect();
        (buf, fds)
    }
}
impl<'input> crate::x11_utils::VoidRequest for CreateWindowRequest<'input> {
}
```
The code in `x11rb` looks like this:
```rust
/// [SNIP]
pub fn create_window<'c, 'input, Conn>(conn: &'c Conn, depth: u8, wid: Window, parent: Window, x: i16, y: i16, width: u16, height: u16, border_width: u16, class: WindowClass, visual: Visualid, value_list: &'input CreateWindowAux) -> Result<VoidCookie<'c, Conn>, ConnectionError>
where
    Conn: RequestConnection + ?Sized,
{
    let request0 = CreateWindowRequest {
        depth,
        wid,
        parent,
        x,
        y,
        width,
        height,
        border_width,
        class,
        visual,
        value_list: Cow::Borrowed(value_list),
    };
    let (bytes, fds) = request0.serialize();
    let slices = [IoSlice::new(&bytes[0]), IoSlice::new(&bytes[1]), IoSlice::new(&bytes[2])];
    assert_eq!(slices.len(), bytes.len());
    conn.send_request_without_reply(&slices, fds)
}
```
And this code is in the extension trait:
```rust
    /// [SNIP]
    fn create_window<'c, 'input>(&'c self, depth: u8, wid: Window, parent: Window, x: i16, y: i16, width: u16, height: u16, border_width: u16, class: WindowClass, visual: Visualid, value_list: &'input CreateWindowAux) -> Result<VoidCookie<'c, Self>, ConnectionError>
    {
        create_window(self, depth, wid, parent, x, y, width, height, border_width, class, visual, value_list)
    }
```

## Common code

The above showed examples for the code that is generated in a single module.
There is also some common code in
[`x11rb_protocol::protocol`](../x11rb-protocol/src/protocol/mod.rs).  This
contains `enum`s over all possible requests, replies, errors, and events.  Via
these, you can e.g. get the `sequence_number` contained in an event without
having to write a big `match` over all possible events.
