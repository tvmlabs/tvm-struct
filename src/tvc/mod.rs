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

#[cfg(test)]
mod tests;

use ton_block::{Deserializable, Serializable};
use ton_types::{BuilderData, Cell, IBitstring, SliceData};

pub const MAX_UINT7: usize = 127; // 2 ** 7 - 1

// small_str#_ len:uint7 string:(len * [ uint8 ]) = SmallStr;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SmallStr {
    pub string: String,
}

impl SmallStr {
    pub fn new<S: Into<String>>(string: S) -> Self {
        Self {
            string: string.into(),
        }
    }
}

#[derive(Debug, failure::Fail)]
#[fail(display = "string length must be <= 127")]
pub struct TooLargeError();

impl Serializable for SmallStr {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let str_bytes = self.string.as_bytes();
        let str_bytes_len = str_bytes.len();

        if str_bytes_len > MAX_UINT7 {
            return Err(TooLargeError().into());
        }

        builder.append_bits(str_bytes_len, 7)?;
        builder.append_raw(str_bytes, str_bytes_len * 8)?;

        Ok(())
    }
}

impl Deserializable for SmallStr {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        let str_bytes_len = slice.get_bits(0, 7)?;
        slice.move_by(7)?;

        let str_bytes = slice.get_next_bytes(str_bytes_len.into())?;

        self.string = String::from_utf8(str_bytes)?;
        Ok(())
    }
}

// version#_ commit:bits160 file_sha256:bits256 semantic:bits = Version;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Version {
    pub commit: [u8; 20],
    pub semantic: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            commit: [0; 20],
            semantic: Default::default(),
        }
    }
}

impl Version {
    pub fn new<S: Into<String>>(commit: [u8; 20], semantic: S) -> Self {
        Self {
            commit,
            semantic: semantic.into(),
        }
    }
}

impl Serializable for Version {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let semantic_bytes = self.semantic.as_bytes();

        builder.append_raw(self.commit.as_slice(), 20 * 8)?;
        builder.append_raw(semantic_bytes, semantic_bytes.len() * 8)?;

        Ok(())
    }
}

impl Deserializable for Version {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        self.commit = slice.get_next_bytes(20)?.try_into().unwrap();
        self.semantic = String::from_utf8(slice.remaining_data().data().to_vec())?;

        Ok(())
    }
}

// metadata#_ sold:^Version linker:^Version
//            compiled_at:uint64 name:SmallStr
//            desc:bits = Metadata;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Metadata {
    pub sold: Version,
    pub linker: Version,
    pub compiled_at: u64,
    pub name: SmallStr,
    pub desc: String,
}

impl Metadata {
    pub fn new<S: Into<String>>(
        sold: Version,
        linker: Version,
        compiled_at: u64,
        name: SmallStr,
        desc: S,
    ) -> Self {
        Self {
            sold,
            linker,
            compiled_at,
            name,
            desc: desc.into(),
        }
    }
}

impl Serializable for Metadata {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        let desc_bytes = self.desc.as_bytes();

        builder.checked_append_reference(self.sold.serialize()?)?;
        builder.checked_append_reference(self.linker.serialize()?)?;
        builder.append_u64(self.compiled_at)?;
        builder.append_builder(&self.name.write_to_new_cell()?)?;
        builder.append_raw(desc_bytes, desc_bytes.len() * 8)?;

        Ok(())
    }
}

impl Deserializable for Metadata {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        self.sold = Version::construct_from_cell(slice.reference(0)?)?;
        self.linker = Version::construct_from_cell(slice.reference(1)?)?;
        self.compiled_at = slice.get_next_u64()?;

        let mut name = SmallStr::default();
        name.read_from(slice)?;

        self.name = name;
        self.desc = String::from_utf8(slice.remaining_data().data().to_vec())?;

        Ok(())
    }
}

// TVMContractE0 – TVC edition 0
// tvm_contract#4f511203 e:uint8
//     code:^Cell meta:(Maybe ^Metadata)
//     { e = 0 } = TVMContract;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TVMContractE0 {
    pub code: Cell,
    pub meta: Option<Metadata>,
}

impl TVMContractE0 {
    const TLB_TAG: u32 = 0x4f511203;
    const EDITION: u8 = 0;

    pub fn new(code: Cell, meta: Option<Metadata>) -> Self {
        Self { code, meta }
    }
}

impl Serializable for TVMContractE0 {
    fn write_to(&self, builder: &mut BuilderData) -> ton_types::Result<()> {
        builder.append_u32(Self::TLB_TAG)?;
        builder.append_u8(Self::EDITION)?;

        builder.checked_append_reference(self.code.serialize()?)?;

        if let Some(meta) = &self.meta {
            builder.append_bit_one()?;
            builder.checked_append_reference(meta.serialize()?)?;
        } else {
            builder.append_bit_zero()?;
        }

        Ok(())
    }
}

#[derive(Debug, failure::Fail)]
pub enum TVMContractE0Error {
    #[fail(display = "unexpected tlb tag, must be 32 bits of 0x4f511203")]
    UnexpectedTLBTag,
    #[fail(display = "unexpected edition, for current struct must be 0")]
    UnexpectedEdition,
}

impl Deserializable for TVMContractE0 {
    fn read_from(&mut self, slice: &mut SliceData) -> ton_types::Result<()> {
        if slice.get_next_u32()? != Self::TLB_TAG {
            return Err(TVMContractE0Error::UnexpectedTLBTag.into());
        }

        if slice.get_next_byte()? != Self::EDITION {
            return Err(TVMContractE0Error::UnexpectedEdition.into());
        }

        self.code = Cell::construct_from_cell(slice.reference(0)?)?;

        if slice.get_next_bit()? == true {
            self.meta = Some(Metadata::construct_from_cell(slice.reference(1)?)?);
        }

        Ok(())
    }
}
