use std::io::{Read, Write};

use common::mem::{IsEven, ReadU16, ToU16, WriteU16};

pub struct Symbol {}

impl Symbol {
    const BYTES: u16 = 6;
}

pub struct Aout {
    pub text: Vec<u8>,
    pub data: Vec<u8>,
    pub bss: Vec<u8>,
    pub symbol_table: Vec<Symbol>,
    pub entry_point: u16,
}

impl Aout {
    const MAGIC: u16 = 0o407;

    pub fn empty() -> Aout {
        Aout {
            text: vec![],
            data: vec![],
            bss: vec![],
            symbol_table: vec![],
            entry_point: 0,
        }
    }

    pub fn read_from(reader: &mut impl Read) -> Aout {
        let magic = reader.read_u16();
        assert_eq!(magic, Self::MAGIC);

        // All bytes
        let text_size = reader.read_u16();

        let data_size = reader.read_u16();
        assert_eq!(data_size, 0);
        assert!(data_size.is_even());

        let bss_size = reader.read_u16();
        assert_eq!(bss_size, 0);
        assert!(bss_size.is_even());

        let symbol_table_size = reader.read_u16();
        assert_eq!(symbol_table_size, 0);

        let entry_point = reader.read_u16();
        assert!(entry_point < text_size);
        assert!(entry_point.is_even());

        // Unused
        let _ = reader.read_u16();

        let _relocation_suppressed = reader.read_u16();

        let mut text = vec![0u8; text_size as usize];
        reader.read_exact(&mut text).unwrap();

        let mut data = vec![0u8; data_size as usize];
        reader.read_exact(&mut data).unwrap();

        let mut bss = vec![0u8; bss_size as usize];
        reader.read_exact(&mut bss).unwrap();

        // Make sure we read the whole file.
        let mut buf = [0u8; 1];
        let res = reader.read_exact(&mut buf);
        assert_eq!(res.unwrap_err().kind(), std::io::ErrorKind::UnexpectedEof);

        Aout {
            text,
            data,
            bss,
            entry_point,
            symbol_table: vec![],
        }
    }

    pub fn write_to(&self, writer: &mut impl Write) {
        assert!(self.entry_point.is_even());
        assert!(self.entry_point < self.text.len().to_u16());

        writer.write_u16(Self::MAGIC);
        writer.write_u16(self.text.len().to_u16());
        writer.write_u16(self.data.len().to_u16());
        writer.write_u16(self.bss.len().to_u16());

        assert_eq!(self.symbol_table.len(), 0);
        writer.write_u16(self.symbol_table.len().to_u16() * Symbol::BYTES);
        writer.write_u16(self.entry_point);
        writer.write_u16(0); // Unused.
        writer.write_u16(0); // "Relocation bits suppressed" flag, not yet implemented.

        writer.write_all(&self.text).unwrap();
        writer.write_all(&self.data).unwrap();
        writer.write_all(&self.bss).unwrap();
    }
}
