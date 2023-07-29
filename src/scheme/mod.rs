/*
    Copyright 2023 EverX Labs.

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

use ton_block::{Deserializable, Serializable};
use ton_types::{fail, BuilderData, Cell, IBitstring, Result, SliceData};

#[derive(Debug, failure::Fail)]
pub enum DeserializationError {
    #[fail(display = "unexpected tlb tag")]
    UnexpectedTLBTag,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct TVC {
    pub code: Option<Cell>,
    pub desc: Option<String>,
}

impl TVC {
    const TVC_TAG: u32 = 0xa2f0b81c;

    pub fn new(code: Option<Cell>, desc: Option<String>) -> Self {
        Self { code, desc }
    }
}

fn builder_store_bytes_ref(b: &mut BuilderData, data: &[u8]) -> Result<()> {
    const CELL_LEN: usize = 127;

    let mut tpb = BuilderData::new();
    let mut len = data.len();
    let mut cap = match len % CELL_LEN {
        0 => CELL_LEN,
        x => x,
    };

    while len > 0 {
        len -= cap;
        tpb.append_raw(&data[len..len + cap], cap * 8)?;

        if len > 0 {
            let mut nb = BuilderData::new();
            nb.checked_append_reference(tpb.clone().into_cell()?)?;
            cap = std::cmp::min(CELL_LEN, len);
            tpb = nb;
        }
    }

    b.checked_append_reference(tpb.into_cell()?)?;
    Ok(())
}

pub fn builder_store_string_ref(builder: &mut BuilderData, data: &str) -> Result<()> {
    builder_store_bytes_ref(builder, data.as_bytes())
}

pub fn slice_load_bytes_ref(slice: &mut SliceData) -> Result<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    let rrb = slice.remaining_references();
    let mut curr: Cell = Cell::construct_from(slice)?;
    assert_eq!(
        rrb - 1,
        slice.remaining_references(),
        "ref not loaded from slice"
    );

    loop {
        let cs = SliceData::load_cell(curr)?;
        let bb = cs.get_bytestring(0);
        bytes.append(&mut bb.clone());

        if cs.remaining_references() > 0 {
            curr = cs.reference(0)?;
        } else {
            break;
        }
    }

    Ok(bytes)
}

pub fn slice_load_string_ref(slice: &mut SliceData) -> Result<String> {
    Ok(String::from_utf8(slice_load_bytes_ref(slice)?)?)
}

impl Serializable for TVC {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        builder.append_u32(Self::TVC_TAG)?;

        if let Some(c) = &self.code {
            builder.append_bit_one()?;
            builder.checked_append_reference(c.to_owned())?;
        } else {
            builder.append_bit_zero()?;
        }

        if let Some(s) = &self.desc {
            builder.append_bit_one()?;
            builder_store_string_ref(builder, s)?;
        } else {
            builder.append_bit_zero()?;
        }

        Ok(())
    }
}

impl Deserializable for TVC {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let tag = slice.get_next_u32()?;
        if tag != Self::TVC_TAG {
            return Err(DeserializationError::UnexpectedTLBTag.into());
        }

        if slice.get_next_bit()? {
            self.code = Some(Cell::construct_from(slice)?);
        }

        if slice.get_next_bit()? {
            self.desc = Some(slice_load_string_ref(slice)?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use hex_literal::hex;
    use ton_block::{Deserializable, Serializable};
    use ton_types::{base64_encode, read_boc, write_boc};

    static DESCSTR: &'static str = r#"
Once upon a time, there was a little kitten named Whiskers. Whiskers was the runt of the
litter and always had trouble keeping up with his siblings. His mother loved him dearly,
but he often felt left out during playtime.

One day, while exploring the yard, Whiskers stumbled upon a butterfly. Mesmerized by its
beauty, Whiskers chased the butterfly around the garden, forgetting all about his troubles.
From that day on, Whiskers became fascinated with the natural world and spent all his time
exploring and learning about the creatures that lived around him.

As he grew older, Whiskers became known throughout the neighborhood for his knowledge of
nature and his ability to make friends with all kinds of animals. His siblings may have
been faster and stronger, but Whiskers had found his own special talent and became a
beloved member of the community.

Despite his humble beginnings, Whiskers learned that he could achieve great things by
following his passions and staying true to himself. And he knew that no matter how
small he may seem, he had a big heart and a lot to offer the world."#;

    #[test]
    fn test_temp() {
        let mut builder = BuilderData::new();
        builder.append_u64(u64::MAX).unwrap();
        builder_store_string_ref(&mut builder, DESCSTR).unwrap();

        let cell = builder.into_cell().unwrap();

        let mut cs = SliceData::load_cell(cell).unwrap();
        let str = slice_load_string_ref(&mut cs).unwrap();

        assert_eq!(DESCSTR, str, "strings are not equal");
    }

    #[test]
    fn test_tvm_contract() {
        const CODEBOC: [u8; 27] = hex!("B5EE9C7241010101000C000014FF00F8008101008011A1EF0ED546");
        let code = &read_boc(&CODEBOC).unwrap().roots[0];

        let tvc = TVC::new(Some(code.to_owned()), Some(DESCSTR.to_string()));

        let cell = tvc.write_to_new_cell().unwrap().into_cell().unwrap();
        let boc = write_boc(&cell).unwrap();

        println!("{}", base64_encode(&boc));

        let deserialized = TVC::construct_from_bytes(&boc).unwrap();
        assert_eq!(tvc, deserialized);
    }
}
