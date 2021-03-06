use super::{Decoder, Encoder};
use bytes::{Buf, BufMut, IntoBuf};
use error::{Error, ErrorKind, Result};
use std;

const MAX_INT_1: u64 = 0b00111111;
const MAX_INT_2: u64 = 0b00111111_11111111;
const MAX_INT_4: u64 = 0b00111111_11111111_11111111_11111111;
const MAX_INT_8: u64 = 0b00111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111;

const INT_1_FLAG: u8 = 0b00;
const INT_2_FLAG: u8 = 0b01;
const INT_4_FLAG: u8 = 0b10;
const INT_8_FLAG: u8 = 0b11;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct VarInt(u64);

impl std::convert::From<u64> for VarInt {
    fn from(v: u64) -> VarInt {
        VarInt(v)
    }
}

impl Encoder for VarInt {
    fn encode<T: BufMut>(&self, dst: &mut T) -> Result<usize> {
        Ok(match self.0 {
            0..=MAX_INT_1 => {
                dst.put_uint_be(self.0 | (INT_1_FLAG as u64) << 6, 1);
                1
            }
            0..=MAX_INT_2 => {
                dst.put_uint_be(self.0 | ((INT_2_FLAG as u64) << 14), 2);
                2
            }
            0..=MAX_INT_4 => {
                dst.put_uint_be(self.0 | ((INT_4_FLAG as u64) << 30), 4);
                4
            }
            0..=MAX_INT_8 => {
                dst.put_uint_be(self.0 | ((INT_8_FLAG as u64) << 62), 8);
                8
            }
            v => panic!(
                "variable-length integer {} has overflown, maximum is {}",
                v, MAX_INT_8
            ),
        })
    }
}

impl Decoder for VarInt {
    fn decode<T: Buf>(&mut self, src: &mut T) -> Result<usize> {
        let first = src.get_u8();
        let (v, n) = match first >> 6 {
            INT_1_FLAG => ((first as u64) & MAX_INT_1, 1),
            INT_2_FLAG => (((first as u64) << 8 | src.get_uint_be(1)) & MAX_INT_2, 2),
            INT_4_FLAG => (((first as u64) << 24 | src.get_uint_be(3)) & MAX_INT_4, 4),
            INT_8_FLAG => (((first as u64) << 56 | src.get_uint_be(7)) & MAX_INT_8, 8),
            _ => unreachable!(),
        };
        self.0 = v;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_var_int(input: Vec<u8>) -> (VarInt, usize) {
        let mut v: VarInt = 0.into();
        let n = v.decode(&mut input.into_buf()).unwrap();
        (v, n)
    }

    #[test]
    fn decode_var_int1_test() {
        assert_eq!(decode_var_int(vec![0b00000011]), (VarInt(3), 1));
    }

    #[test]
    fn decode_var_int2_test() {
        assert_eq!(
            decode_var_int(vec![0b01000001, 0b00000001]),
            (VarInt(257), 2)
        );
    }

    #[test]
    fn decode_var_int4_test() {
        assert_eq!(
            decode_var_int(vec![0b10000001, 0b00000001, 0b00000001, 0b0000000]),
            (VarInt(16843008), 4)
        );
    }

    #[test]
    fn decode_var_int8_test() {
        assert_eq!(
            decode_var_int(vec![
                0b11000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001,
                0b00000001,
            ]),
            (VarInt(72340172838076673u64), 8)
        );
    }

    fn encode_var_int(input: VarInt) -> Vec<u8> {
        let mut dst = vec![];
        input.encode(&mut dst).unwrap();
        dst
    }

    #[test]
    fn encode_var_int1_test() {
        assert_eq!(encode_var_int(3.into()), vec![0b00000011]);
    }

    #[test]
    fn encode_var_int2_test() {
        assert_eq!(encode_var_int(257.into()), vec![0b01000001, 0b00000001]);
    }

    #[test]
    fn encode_var_int4_test() {
        assert_eq!(
            encode_var_int(16843009.into()),
            vec![0b10000001, 0b00000001, 0b00000001, 0b0000001],
        );
    }

    #[test]
    fn encode_var_int8_test() {
        assert_eq!(
            encode_var_int(72340172838076673u64.into()),
            vec![
                0b11000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001, 0b00000001,
                0b00000001,
            ],
        );
    }
}
