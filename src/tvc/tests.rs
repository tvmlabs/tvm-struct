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

use hex_literal::hex;
use ton_block::{Deserializable, Serializable};

use crate::tvc::*;

#[test]
fn test_small_str() {
    const CASE: &str = "hello cute kitty!";

    let dat = SmallStr::new(CASE);
    let boc = dat.write_to_bytes().unwrap();

    let deserialized = SmallStr::construct_from_bytes(boc.as_slice()).unwrap();
    assert_eq!(dat, deserialized);
}

#[test]
fn test_small_str_too_long() {
    let case = &str::repeat("1", MAX_UINT7 + 1);

    let dat = SmallStr::new(case);
    let boc = dat.write_to_bytes().unwrap_err();

    assert!(boc.downcast_ref::<TooLargeError>().is_some());
}

#[test]
fn test_version() {
    const COMMIT: [u8; 20] = hex!("4e97449a48c05600af00027d652519de61190b53");
    const SEMANTIC: &str = "v0.18.4";

    let ver = Version::new(COMMIT, SEMANTIC);

    let boc = ver.write_to_bytes().unwrap();
    let deserialized = Version::construct_from_bytes(boc.as_slice()).unwrap();

    assert_eq!(ver, deserialized);
}

#[test]
fn test_metadata() {
    const COMMIT: [u8; 20] = hex!("4e97449a48c05600af00027d652519de61190b53");
    const COMPILED_AT: u64 = 1676912859;
    const DESC: &str = "Simple wallet v3 contract with seqno";

    let sold_version = Version::new(COMMIT, "v0.1.1");
    let linker_version = Version::new(COMMIT, "v0.2.2");
    let name = SmallStr::new("WalletV3");

    let meta = Metadata::new(
        sold_version.clone(),
        linker_version.clone(),
        COMPILED_AT,
        name.clone(),
        DESC,
    );

    let boc = meta.write_to_bytes().unwrap();
    let deserialized = Metadata::construct_from_bytes(boc.as_slice()).unwrap();

    assert_eq!(meta, deserialized);
}

#[test]
fn test_tvm_contract() {
    const COMMIT: [u8; 20] = hex!("4e97449a48c05600af00027d652519de61190b53");
    const CODEBOC: [u8; 41] = hex!(
        // <{ SETCP0 ACCEPT PROGRAM{ main PROC:<{ }> }END>c PUSHREF SETCODE }>c
        "B5EE9C7241010301001A00010EFF00F80088FB04010114FF00F4A413F4BCF2C80B020002D393E0BA78"
    );

    let code = Cell::construct_from_bytes(&CODEBOC).unwrap();
    let meta = Metadata::new(
        Version::new(COMMIT, "v0.1.1"),
        Version::new(COMMIT, "v0.2.2"),
        1676912859,
        SmallStr::new("WalletV3"),
        "Simple wallet v3 contract with seqno",
    );

    let tvc = TVMContractE0::new(code, Some(meta));
    let boc = tvc.write_to_bytes().unwrap();
    let deserialized = TVMContractE0::construct_from_bytes(boc.as_slice()).unwrap();

    assert_eq!(tvc, deserialized);
}
