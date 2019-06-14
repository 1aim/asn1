use crate::Value;

pub fn to_der<A: AsRef<[u8]>, I: Into<Value<A>>>(value: I) -> Vec<u8> {
    let value = value.into();
    let mut buffer = Vec::with_capacity(value.len());

    value.tag.encode(&mut buffer);
    encode_contents(value.contents.as_ref(), &mut buffer);

    buffer
}

fn encode_contents(contents: &[u8], buffer: &mut Vec<u8>) {
    if contents.len() <= 127 {
        buffer.push(contents.len() as u8);
    } else {
        let mut length = contents.len();
        let mut length_buffer = Vec::new();

        while length != 0 {
            length_buffer.push((length & 0xff) as u8);
            length >>= 8;
        }

        buffer.push(length_buffer.len() as u8 | 0x80);
        buffer.append(&mut length_buffer);
    }

    buffer.extend_from_slice(contents);
}

#[cfg(test)]
mod tests {
    use core::types::ObjectIdentifier;

    use super::*;

    #[test]
    fn bool() {
        assert_eq!(to_der(true), &[1, 1, 255]);
        assert_eq!(to_der(false), &[1, 1, 0]);
    }

    #[test]
    fn object_identifier_to_bytes() {
        let itu: Vec<u8> = to_der(ObjectIdentifier::new(vec![2, 999, 3]).unwrap());
        let rsa: Vec<u8> = to_der(ObjectIdentifier::new(vec![1, 2, 840, 113549]).unwrap());

        assert_eq!(&[0x6, 0x3, 0x88, 0x37, 0x03][..], &*itu);
        assert_eq!(&[0x6, 0x6, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d][..], &*rsa);
    }
}
