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

use crate::scheme::*;

#[test]
fn test_small_str() {
    const CASE: &str = "hello cute kitty!";

    let dat = SmallStr::new(CASE.to_string());
    let boc = dat.write_to_bytes().unwrap();

    let deserialized = SmallStr::construct_from_bytes(boc.as_slice()).unwrap();
    assert_eq!(dat, deserialized);
}

#[test]
fn test_small_str_too_long() {
    let case = &str::repeat("1", MAX_UINT7 + 1);

    let bocerr = SmallStr::new(case.to_string())
        .write_to_bytes()
        .unwrap_err();

    assert_eq!(
        bocerr.downcast::<SmallStrError>().unwrap(),
        SmallStrError::TooLarge
    );
}

#[test]
fn test_version() {
    const COMMIT: [u8; 20] = hex!("4e97449a48c05600af00027d652519de61190b53");
    const SEMANTIC: &str = "v0.18.4";

    let ver = Version::new(COMMIT, SEMANTIC.to_string());

    let boc = ver.write_to_bytes().unwrap();
    let deserialized = Version::construct_from_bytes(boc.as_slice()).unwrap();

    assert_eq!(ver, deserialized);
}

#[test]
fn test_metadata() {
    const COMMIT: [u8; 20] = hex!("4e97449a48c05600af00027d652519de61190b53");
    const COMPILED_AT: u64 = 1676912859;
    const DESC: &str = "Simple wallet v3 contract with seqno";

    let sold_version = Version::new(COMMIT, "v0.1.1".to_string());
    let linker_version = Version::new(COMMIT, "v0.2.2".to_string());
    let name = SmallStr::new("WalletV3".to_string());

    let meta = Metadata::new(
        sold_version,
        linker_version,
        COMPILED_AT,
        name,
        DESC.to_string(),
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
        Version::new(COMMIT, "v0.1.1".to_string()),
        Version::new(COMMIT, "v0.2.2".to_string()),
        1676912859,
        SmallStr::new("WalletV3".to_string()),
        "Simple wallet v3 contract with seqno".to_string(),
    );

    let smc = TvmSmc::TvcFrst(TvcFrst::new(code, Some(meta)));
    let tvc = TVC::new(smc);

    let boc = tvc.write_to_bytes().unwrap();
    let deserialized = TVC::construct_from_bytes(boc.as_slice()).unwrap();

    assert_eq!(tvc, deserialized);
}
