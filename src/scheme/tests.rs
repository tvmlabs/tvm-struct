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
use ton_types::{read_boc, write_boc};

use super::*;
use hex_literal::hex;

#[test]
fn test_temp() {
    const DESCSTR: &str = r#"
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
    const DESCSTR: &str = r#"
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
small he may seem, he had a big heart and a lot to offer the world.
"#;

    const CODEBOC: [u8; 27] = hex!("B5EE9C7241010101000C000014FF00F8008101008011A1EF0ED546");
    let code = &read_boc(&CODEBOC).unwrap().roots[0];

    let tvc = TVC::new(
        Some(code.to_owned()), 
        Some(DESCSTR.to_string())
    );

    let cell = tvc.write_to_new_cell().unwrap().into_cell().unwrap();
    let boc = write_boc(&cell).unwrap();

    let deserialized = TVC::construct_from_bytes(&boc).unwrap();
    assert_eq!(tvc, deserialized);
}
