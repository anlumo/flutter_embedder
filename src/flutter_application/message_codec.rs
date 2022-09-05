use std::{
    array::TryFromSliceError, marker::PhantomData, mem::size_of, num::TryFromIntError,
    str::Utf8Error,
};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::cast::FromPrimitive;
use serde::{
    de::{self, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess},
    Deserialize,
};

#[derive(Debug)]
pub enum Error {
    TupleLength,
    ExpectedNil,
    Utf8Error(Utf8Error),
    ValueOutOfRange(TryFromIntError),
    InvalidFieldType,
    TrailingCharacters,
    Eof,
    Message(String),
}

impl de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<TryFromIntError> for Error {
    fn from(err: TryFromIntError) -> Self {
        Self::ValueOutOfRange(err)
    }
}

impl From<Utf8Error> for Error {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::TupleLength => formatter.write_str("Tuple length doesn't match list length"),
            Error::ExpectedNil => formatter.write_str("Expected nil"),
            Error::Utf8Error(err) => err.fmt(formatter),
            Error::ValueOutOfRange(err) => err.fmt(formatter),
            Error::InvalidFieldType => formatter.write_str("Invalid field type"),
            Error::Message(msg) => formatter.write_str(msg),
            Error::Eof => formatter.write_str("unexpected end of input"),
            Error::TrailingCharacters => formatter.write_str("trailing characters in input"),
            /* and so forth */
        }
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
enum FlutterStandardField {
    Nil,
    True,
    False,
    Int32,
    Int64,
    IntHex,
    Float64,
    String,
    UInt8Data,
    Int32Data,
    Int64Data,
    Float64Data,
    List,
    Map,
    Float32Data,
}

pub struct Deserializer<'de> {
    input: &'de [u8],
    pos: usize,
}

impl<'de> Deserializer<'de> {
    pub fn from_slice(input: &'de [u8]) -> Self {
        Deserializer { input, pos: 0 }
    }
}

pub fn from_slice<'a, T>(b: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_slice(b);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.len() == deserializer.pos {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer<'de> {
    fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        if self.pos + N > self.input.len() {
            Err(Error::Eof)
        } else {
            let mut result = [0; N];
            result.copy_from_slice(&self.input[self.pos..self.pos + N]);
            self.pos += N;
            Ok(result)
        }
    }
    fn read_byte(&mut self) -> Result<u8, Error> {
        Ok(self.read_bytes::<1>()?[0])
    }
    fn read_field_type(&mut self) -> Result<FlutterStandardField, Error> {
        FlutterStandardField::from_u8(self.read_byte()?).ok_or(Error::InvalidFieldType)
    }
    fn peek_field_type(&mut self) -> Result<FlutterStandardField, Error> {
        if self.pos >= self.input.len() {
            Err(Error::Eof)
        } else {
            FlutterStandardField::from_u8(self.input[self.pos]).ok_or(Error::InvalidFieldType)
        }
    }

    fn read_size(&mut self) -> Result<usize, Error> {
        let byte = self.read_byte()?;
        if byte < 254 {
            return Ok(byte as _);
        }
        if byte == 254 {
            return Ok(u16::from_le_bytes(self.read_bytes()?) as _);
        }
        Ok(u32::from_le_bytes(self.read_bytes()?) as _)
    }

    fn read_data(&mut self, len: usize) -> Result<&[u8], Error> {
        if self.pos + len > self.input.len() {
            Err(Error::Eof)
        } else {
            let result = &self.input[self.pos..self.pos + len];
            self.pos += len;
            Ok(result)
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.peek_field_type()? {
            FlutterStandardField::Nil => self.deserialize_option(visitor),
            FlutterStandardField::True | FlutterStandardField::False => {
                self.deserialize_bool(visitor)
            }
            FlutterStandardField::Int32 => self.deserialize_i32(visitor),
            FlutterStandardField::Int64 => self.deserialize_i64(visitor),
            FlutterStandardField::IntHex => self.deserialize_str(visitor),
            FlutterStandardField::Float64 => self.deserialize_f64(visitor),
            FlutterStandardField::String => self.deserialize_str(visitor),
            FlutterStandardField::UInt8Data => self.deserialize_bytes(visitor),
            FlutterStandardField::Int32Data => self.deserialize_seq(visitor),
            FlutterStandardField::Int64Data => self.deserialize_seq(visitor),
            FlutterStandardField::Float32Data => self.deserialize_seq(visitor),
            FlutterStandardField::Float64Data => self.deserialize_seq(visitor),
            FlutterStandardField::List => self.deserialize_seq(visitor),
            FlutterStandardField::Map => self.deserialize_map(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_field_type()? {
            FlutterStandardField::True => visitor.visit_bool(true),
            FlutterStandardField::False => visitor.visit_bool(false),
            _ => Err(Error::InvalidFieldType),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int32 {
            visitor.visit_i8(i32::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int32 {
            visitor.visit_i16(i32::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int32 {
            visitor.visit_i32(i32::from_le_bytes(self.read_bytes()?))
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int64 {
            visitor.visit_i64(i64::from_le_bytes(self.read_bytes()?))
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int32 {
            visitor.visit_u8(i32::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int32 {
            visitor.visit_u16(i32::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int64 {
            visitor.visit_u32(i64::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Int64 {
            visitor.visit_u32(i64::from_le_bytes(self.read_bytes()?).try_into()?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Float64 {
            visitor.visit_f32(f64::from_le_bytes(self.read_bytes()?) as _)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Float64 {
            visitor.visit_f64(f64::from_le_bytes(self.read_bytes()?))
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_field_type()? {
            FlutterStandardField::IntHex | FlutterStandardField::String => {
                let len = self.read_size()?;
                let bytes = self.read_data(len)?;
                visitor.visit_str(std::str::from_utf8(bytes)?)
            }
            _ => Err(Error::InvalidFieldType),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_field_type()? {
            FlutterStandardField::IntHex | FlutterStandardField::String => {
                let len = self.read_size()?;
                let bytes = self.read_data(len)?;
                visitor.visit_string(std::str::from_utf8(bytes)?.to_owned())
            }
            _ => Err(Error::InvalidFieldType),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::UInt8Data {
            let len = self.read_size()?;
            visitor.visit_bytes(self.read_data(len)?)
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::UInt8Data {
            let len = self.read_size()?;
            visitor.visit_byte_buf(self.read_data(len)?.to_vec())
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.peek_field_type()? == FlutterStandardField::Nil {
            self.read_field_type()?;
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Nil {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNil)
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.read_field_type()? {
            FlutterStandardField::List => {
                let len = self.read_size()?;
                visitor.visit_seq(ListDeserializer::new(self, len))
            }
            FlutterStandardField::Int32Data => {
                let len = self.read_size()?;
                visitor.visit_seq(PrimitiveListDeserializer::<i32>::new(self, len))
            }
            FlutterStandardField::Int64Data => {
                let len = self.read_size()?;
                visitor.visit_seq(PrimitiveListDeserializer::<i64>::new(self, len))
            }
            FlutterStandardField::Float32Data => {
                let len = self.read_size()?;
                visitor.visit_seq(PrimitiveListDeserializer::<f32>::new(self, len))
            }
            FlutterStandardField::Float64Data => {
                let len = self.read_size()?;
                visitor.visit_seq(PrimitiveListDeserializer::<f64>::new(self, len))
            }
            _ => Err(Error::InvalidFieldType),
        }
    }

    fn deserialize_tuple<V>(self, tuple_len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::List {
            let len = self.read_size()?;
            if len != tuple_len {
                Err(Error::TupleLength)
            } else {
                visitor.visit_seq(ListDeserializer::new(self, len))
            }
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if self.read_field_type()? == FlutterStandardField::Map {
            let len = self.read_size()?;
            visitor.visit_map(ListDeserializer::new(self, len))
        } else {
            Err(Error::InvalidFieldType)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(Enum::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        log::debug!(
            "deserialize_identifier, peek = {:?}",
            self.peek_field_type()?
        );
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct ListDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    len: usize,
}

impl<'a, 'de> ListDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        Self { de, len }
    }
}

impl<'de, 'a> SeqAccess<'de> for ListDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }
}

impl<'de, 'a> MapAccess<'de> for ListDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

trait EndianRead: Sized {
    type Array;
    fn from_le_bytes(bytes: Self::Array) -> Self;
    fn from_be_bytes(bytes: Self::Array) -> Self;
    fn try_from_le_bytes(bytes: &[u8]) -> Result<Self, TryFromSliceError>;
    fn try_from_be_bytes(bytes: &[u8]) -> Result<Self, TryFromSliceError>;
}

macro_rules! impl_EndianRead_for_nums (( $($num:ident),* ) => {
    $(
        impl EndianRead for $num {
            type Array = [u8; std::mem::size_of::<Self>()];
            fn from_le_bytes(bytes: Self::Array) -> Self { Self::from_le_bytes(bytes) }
            fn from_be_bytes(bytes: Self::Array) -> Self { Self::from_be_bytes(bytes) }
            fn try_from_le_bytes(bytes: &[u8]) -> Result<Self, TryFromSliceError> {
                Ok(Self::from_le_bytes(bytes.try_into()?))
            }
            fn try_from_be_bytes(bytes: &[u8]) -> Result<Self, TryFromSliceError> {
                Ok(Self::from_be_bytes(bytes.try_into()?))
            }
        }
    )*
});

impl_EndianRead_for_nums!(i32, i64, f32, f64);

struct PrimitiveListDeserializer<'a, 'de: 'a, N: EndianRead + IntoDeserializer<'de>> {
    de: &'a mut Deserializer<'de>,
    len: usize,
    type_: PhantomData<N>,
}

impl<'a, 'de, N: EndianRead + IntoDeserializer<'de>> PrimitiveListDeserializer<'a, 'de, N> {
    fn new(de: &'a mut Deserializer<'de>, len: usize) -> Self {
        let size = size_of::<N>();
        let modulo = de.pos % size;
        if modulo != 0 {
            de.pos += size - modulo;
        }

        Self {
            de,
            len,
            type_: PhantomData,
        }
    }
}

impl<'de, 'a, N: EndianRead + IntoDeserializer<'de>> SeqAccess<'de>
    for PrimitiveListDeserializer<'a, 'de, N>
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            Ok(None)
        } else {
            self.len -= 1;
            let bytes = self.de.read_data(size_of::<N>())?;
            let number = N::try_from_le_bytes(&bytes).unwrap();
            seed.deserialize(number.into_deserializer())
                .map(Some)
                .map_err(|_| Error::InvalidFieldType)
        }
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        // I don't think that the format has a way to verify this
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self.de, len, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}
