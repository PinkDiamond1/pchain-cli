/*
 Copyright (c) 2022 ParallelChain Lab

 This program is free software: you can redistribute it and/or modify
 it under the terms of the GNU General Public License as published by
 the Free Software Foundation, either version 3 of the License, or
 (at your option) any later version.

 This program is distributed in the hope that it will be useful,
 but WITHOUT ANY WARRANTY; without even the implied warranty of
 MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::path::Path;

use protocol_types::{CallData, Serializable};
use serde_json::{Value};
use borsh::{BorshSerialize, BorshDeserialize};
use regex::Regex;

pub struct Builder {
    pub args :Vec<Vec<u8>>
}

impl Builder {
    pub fn new() -> Self {
        Self { args: vec![] }
    }
    pub fn add<T: BorshSerialize>(mut self,  arg :T) -> Self{
        self.insert(arg);
        self
    }

    pub fn insert<T: BorshSerialize>(&mut self, arg :T) {
        let mut args_bs:Vec<u8> = vec![]; 
        arg.serialize(&mut args_bs).unwrap();
        self.args.push(args_bs.clone());
    }

    pub fn make_data(&self, entry_name :&str) -> (Vec<u8>, Vec<u8>) {
        let version_bs = (0 as u32).to_le_bytes().to_vec();

        let mut args_bs :Vec<u8> = vec![];
        BorshSerialize::serialize(&self.args, &mut args_bs).unwrap();

        let ctx = CallData{
            method_name: entry_name.to_string(),
            arguments: args_bs.clone()
        };
        let call_data_bs = CallData::serialize(&ctx);
        (
            [version_bs, call_data_bs].concat(),
            args_bs
        )
    }

    fn check_vals(value :&str) -> bool {
        let re = Regex::new(r"\[[\s*\-*\d\s*,*\s*]*\]").unwrap();
        !re.is_match(value)
    }

    fn vals_to_vec(value :&str) -> Result<Vec<String>, ()>{
        let mut val_str = value.to_string();
        val_str.remove(0);
        val_str.remove(val_str.len()-1);
        let val_strs :Vec<&str> = val_str.split(",").collect();
        Ok(val_strs.iter().map(|v|{
            v.trim().to_string()
        }).collect())
    }


    pub fn insert_from_str(&mut self, data_type :&str, value :&str) -> std::result::Result<(), String> {
        macro_rules! insert_primitives {
            ($d:expr, $v:expr, $($t:ty, )*) => {
                $(
                    if $d == stringify!($t) {
                        match $v.parse::<$t>() {
                            Ok(typed_value) => self.insert(typed_value),
                            Err(_) => return Err(format!("cannot parse {}", $d))
                        }
                        return Ok(());
                    }
                )*
            };
        }
        macro_rules! insert_vecs {
            ($d:expr, $v:expr, $($t:ty, )*) => {
                $(
                    if $d == concat!("Vec<",stringify!($t),">") {
                        if Self::check_vals($v) { return Err(format!("cannot parse {}", $d))}
                        let mut ret_vs = Vec::<$t>::new();
                        match Self::vals_to_vec($v) {
                            Ok(val_strs) => {
                                for v in val_strs {
                                    match v.parse::<$t>() {
                                        Ok(typed_value) => ret_vs.push(typed_value),
                                        Err(_) => return Err(format!("cannot parse {}", $d))
                                    };
                                }
                            },
                            Err(_) => return Err(format!("cannot parse {}", $d))
                        };
                        self.insert(ret_vs);
                        return Ok(());
                    }
                )*
            };
        }

        macro_rules! insert_vecs_unchecked {
            ($d:expr, $v:expr, $($t:ty, )*) => {
                $(
                    if $d == concat!("Vec<",stringify!($t),">") {
                        let mut ret_vs = Vec::<$t>::new();
                        match Self::vals_to_vec($v) {
                            Ok(val_strs) => {
                                for v in val_strs {
                                    match v.parse::<$t>() {
                                        Ok(typed_value) => ret_vs.push(typed_value),
                                        Err(_) => return Err(format!("cannot parse {}", $d))
                                    };
                                }
                            },
                            Err(_) => return Err(format!("cannot parse {}", $d))
                        };
                        self.insert(ret_vs);
                        return Ok(());
                    }
                )*
            };
        }

        macro_rules! insert_slice {
            ($d:expr, $v:expr, $($s:expr, )*) => {
                $(
                    if $d == concat!("[",stringify!($s),"]") {
                        if Self::check_vals(value) { return Err(format!("cannot parse {}", $d))}
                        let mut ret_vs = Vec::<u8>::new();
                        match Self::vals_to_vec($v) {
                            Ok(val_strs) => {
                                for v in val_strs {
                                    match v.parse::<u8>() {
                                        Ok(typed_value) => ret_vs.push(typed_value),
                                        Err(_) => return Err(format!("cannot parse {}", $d))
                                    };
                                }
                            },
                            Err(_) => return Err(format!("cannot parse {}", $d))
                        };
                        if ret_vs.len() != $s {return Err(format!("cannot parse {}", data_type)) }
                        let mut ret_bs = [0u8; $s];
                        ret_bs.copy_from_slice(ret_vs.as_slice());
                        self.insert(ret_bs);
                        return Ok(());
                    }
                )*
            };
        }

        insert_primitives!(data_type, value, 
            i8, i16, i32, i64, i128,
            u8, u16, u32, u64, u128,
            bool, String,
        );

        insert_vecs!(data_type, value, 
            i8, i16, i32, i64, i128,
            u8, u16, u32, u64, u128,
        );

        insert_vecs_unchecked!(data_type, value,
            bool, String,
        );

        insert_slice!(data_type, value,
            32, 64,
        );

        if data_type == "address" {
            match protocol_types::Base64URL::decode(value) {
                Ok(bs) => {
                    if bs.len() != 32 { 
                        return Err(format!("cannot parse {}", data_type))
                    }
                    let mut arg_bs: [u8; 32] = [0u8; 32];
                    arg_bs.copy_from_slice(bs.as_slice());
                    self.insert(arg_bs);
                    return Ok(());
                },
                Err(_) => return Err(format!("cannot parse {}", data_type))
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct CLICallData{
    method_name: String,
    arguments: Vec<(String, String)> // type-value pair
}

impl CLICallData {
    fn from_json(json_data: &str) -> core::result::Result<CLICallData, ()> {
        let json_val: Value = match serde_json::from_str(json_data) {
            Ok(val) => { val },
            Err(_) => return Err(())
        };

        // parse method name
        let method_name = match &json_val["method_name"].as_str() {
            Some(method_name) => { method_name.to_string()},
            None => return Err(())
        };

        let json_args: Vec<Value> = match &json_val["arguments"].as_array() {
            Some(args) => { args.to_vec() },
            None => return Err(()),
        };

        // parse arguments
        let arguments: Vec<(String, String)> = json_args.iter().filter_map(|jarg|{
            if let Some(j_type) = jarg["type"].as_str() {
                if let Some(j_val) = jarg["value"].as_str() {
                    Some((j_type.to_string(), j_val.to_string()))
                } else {None}
            } else {None}
        }).collect();
        
        Ok(CLICallData{
            method_name,
            arguments
        })
    }
}

/// Parse to tx-data and viewargs
pub fn parse(path_to_json: String) -> (String, String) {
    let json_string = if Path::new(&path_to_json).is_file(){
        match std::fs::read(&path_to_json) {
            Ok(data) => match String::from_utf8(data) {
                Ok(call_data_json) => {  call_data_json },
                Err(e) => {
                    println!("Error: : Fail to parse file although is file found {:?}", e);
                    std::process::exit(1);  
                }
            },
            Err(e) => {
                println!("Error: : Fail to read file although is file found {:?}", e);
                std::process::exit(1);  
            }
        }
    } else {
        println!("Error: : Invalid path. Cannot retrieve designated keypair file from the designated path.");
        std::process::exit(1);
    };

    parse_call_data(json_string)
}

fn parse_call_data(json_string: String) -> (String, String) {
    let call_data_from_json =  CLICallData::from_json(&json_string).unwrap();
    let mut arg_builder = Builder::new();
    for (data_type, value) in call_data_from_json.arguments {
        arg_builder.insert_from_str(data_type.as_str(), value.as_str()).unwrap();
    }

    let (data, arguments) = arg_builder.make_data(call_data_from_json.method_name.as_str());
    let output_data_str = protocol_types::Base64URL::encode(data).to_string();
    let output_arguments_str = protocol_types::Base64URL::encode(arguments).to_string();
    (output_data_str, output_arguments_str)
}

#[derive(BorshSerialize, BorshDeserialize)]
struct CallBack {
    return_value: Vec<u8>
}

impl CallBack {
    fn to_data_type(&self, data_type: String) -> String {
        macro_rules! convert_to_data_type {
            ($d:expr, $($t:ty,)*) => {
                $(
                    if data_type == stringify!($t) {
                        match <$t>::deserialize(&mut self.return_value.as_slice()) {
                            Ok(data) => {
                                return format!("{:?}", data);
                            },
                            Err(e) => {
                                println!("Error: : Fail to convert to target data tyoe. {:?}", e);
                                std::process::exit(1);  
                            }
                        }
                    }
                )*
            };
        }

        macro_rules! convert_to_vecs {
            ($d:expr, $($t:ty,)*) => {
                $(
                    if data_type == concat!("Vec<", stringify!($t), ">") {
                        match Vec::<$t>::deserialize(&mut self.return_value.as_slice()) {
                            Ok(data) => {
                                return format!("{:?}", data);
                            },
                            Err(e) => {
                                println!("Error: : Fail to convert to target data tyoe. {:?}", e);
                                std::process::exit(1);  
                            }
                        }
                    }
                )*
            };
        }

        macro_rules! convert_to_slice {
            ($d:expr, $($s:expr,)*) => {
                $(
                    if data_type == concat!("[", stringify!($s), "]") {
                        match <[u8; $s]>::deserialize(&mut self.return_value.as_slice()) {
                            Ok(data) => {
                                return format!("{:?}", data);
                            },
                            Err(e) => {
                                println!("Error: : Fail to convert to target data tyoe. {:?}", e);
                                std::process::exit(1);  
                            }
                        }
                    }
                )*
            };
        }

        convert_to_data_type!(data_type,
            u8, u16, u32, u64, u128,
            i8, i16, i32, i64, i128,
            bool, String,
        );

        convert_to_vecs!(data_type,
            u8, u16, u32, u64, u128,
            i8, i16, i32, i64, i128,
            bool, String,
        );

        convert_to_slice!(data_type,
            32, 64,
        );

        "".to_string()
    }
}

/// return data representation (require Debug trait)
pub fn from_callback(value :String, data_type: String) -> String {
    let borsh_serialized = match protocol_types::Base64URL::decode(&value){
        Ok(data) => data,
        Err(e) => {
            println!("Error: : Fail to decode base64 string {:?}", e);
            std::process::exit(1);  
        }
    };

    let call_back: CallBack = match BorshDeserialize::deserialize(&mut borsh_serialized.as_slice()) {
        Ok(cb) => cb,
        Err(e) => {
            println!("Error: : Fail to decode data as it is not with expected data format. {:?}", e);
            std::process::exit(1);  
        }
    };

    call_back.to_data_type(data_type)
}

#[cfg(test)]
mod test {
    use borsh::BorshSerialize;

    use super::CallBack;


    #[test]
    fn test_parse() {
        let json_string = r#" {
            "method_name": "hello_world",
            "arguments": [
                {"type": "i8", "value":"-1"},
                {"type": "Vec<i8>", "value":"[-1]"},
                {"type": "i16", "value":"-30000"},
                {"type": "Vec<i16>", "value":"[-1,0]"},
                {"type": "i32", "value":"-1094967295"},
                {"type": "Vec<i32>", "value":"[-1,0,1]"},
                {"type": "i64", "value":"-9046744073709551615"},
                {"type": "Vec<i64>", "value":"[-1,0,1,65656565]"},
                {"type": "i128", "value":"-9046744073709551615"},
                {"type": "Vec<i128>", "value":"[-1,0,1,-65656565,0]"},
                {"type": "u8", "value":"255"},
                {"type": "Vec<u8>", "value":"[0]"},
                {"type": "u16", "value":"65535"},
                {"type": "Vec<u16>", "value":"[65535,6535]"},
                {"type": "u32", "value":"4294967295"},
                {"type": "Vec<u32>", "value":"[65535,6535,1919]"},
                {"type": "u64", "value":"18446744073709551615"},
                {"type": "Vec<u64>", "value":"[65535,6535,1919112123223]"},
                {"type": "u128", "value":"18446744073709551616"},
                {"type": "Vec<u128>", "value":"[65535,6535,1919112123223,123123,124124,125152]"},
                {"type": "bool", "value": "true"},
                {"type": "Vec<bool>", "value": "[true,false,true]"},
                {"type": "String", "value": "string data"},
                {"type": "Vec<String>", "value": "[string data,asdaf,1d1 as2]"},
                {"type": "[32]", "value": "[1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2]"},
                {"type": "[64]", "value": "[1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2]"}
            ]
        }"#;

        let (d, p) = super::parse_call_data(json_string.to_string());
        assert_eq!(d, "AAAAAAsAAAB6AgAAaGVsbG9fd29ybGQaAAAAAQAAAP8FAAAAAQAAAP8CAAAA0IoIAAAAAgAAAP__AAAEAAAAASC8vhAAAAADAAAA_____wAAAAABAAAACAAAAAEArFgygnOCJAAAAAQAAAD__________wAAAAAAAAAAAQAAAAAAAAD11ukDAAAAABAAAAABAKxYMoJzgv__________VAAAAAUAAAD_____________________AAAAAAAAAAAAAAAAAAAAAAEAAAAAAAAAAAAAAAAAAAALKRb8________________AAAAAAAAAAAAAAAAAAAAAAEAAAD_BQAAAAEAAAAAAgAAAP__CAAAAAIAAAD__4cZBAAAAP____8QAAAAAwAAAP__AACHGQAAfwcAAAgAAAD__________xwAAAADAAAA__8AAAAAAACHGQAAAAAAAFcT_9O-AQAAEAAAAAAAAAAAAAAAAQAAAAAAAABkAAAABgAAAP__AAAAAAAAAAAAAAAAAACHGQAAAAAAAAAAAAAAAAAAVxP_074BAAAAAAAAAAAAAPPgAQAAAAAAAAAAAAAAAADc5AEAAAAAAAAAAAAAAAAA4OgBAAAAAAAAAAAAAAAAAAEAAAABBwAAAAMAAAABAAEPAAAACwAAAHN0cmluZyBkYXRhJwAAAAMAAAALAAAAc3RyaW5nIGRhdGEFAAAAYXNkYWYHAAAAMWQxIGFzMiAAAAABAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAkAAAAABAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAgECAwQFBgcICQABAgMEBQYHCAkAAQIDBAUGBwgJAAEC");
        assert_eq!(p, "GgAAAAEAAAD_BQAAAAEAAAD_AgAAANCKCAAAAAIAAAD__wAABAAAAAEgvL4QAAAAAwAAAP____8AAAAAAQAAAAgAAAABAKxYMoJzgiQAAAAEAAAA__________8AAAAAAAAAAAEAAAAAAAAA9dbpAwAAAAAQAAAAAQCsWDKCc4L__________1QAAAAFAAAA_____________________wAAAAAAAAAAAAAAAAAAAAABAAAAAAAAAAAAAAAAAAAACykW_P_______________wAAAAAAAAAAAAAAAAAAAAABAAAA_wUAAAABAAAAAAIAAAD__wgAAAACAAAA__-HGQQAAAD_____EAAAAAMAAAD__wAAhxkAAH8HAAAIAAAA__________8cAAAAAwAAAP__AAAAAAAAhxkAAAAAAABXE__TvgEAABAAAAAAAAAAAAAAAAEAAAAAAAAAZAAAAAYAAAD__wAAAAAAAAAAAAAAAAAAhxkAAAAAAAAAAAAAAAAAAFcT_9O-AQAAAAAAAAAAAADz4AEAAAAAAAAAAAAAAAAA3OQBAAAAAAAAAAAAAAAAAODoAQAAAAAAAAAAAAAAAAABAAAAAQcAAAADAAAAAQABDwAAAAsAAABzdHJpbmcgZGF0YScAAAADAAAACwAAAHN0cmluZyBkYXRhBQAAAGFzZGFmBwAAADFkMSBhczIgAAAAAQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQJAAAAAAQIDBAUGBwgJAAECAwQFBgcICQABAgMEBQYHCAkAAQIBAgMEBQYHCAkAAQIDBAUGBwgJAAECAwQFBgcICQABAg");
    }

    #[test]
    fn test_callback(){
        macro_rules! assert_data_types {
            ($($t:expr, $v:expr, $e:expr,)*) => {
                $(
                    let value = {
                        let mut buf = Vec::<u8>::new();
                        $v.serialize(&mut buf).unwrap();
                        let cb = CallBack{ return_value: buf };
                        let mut ret = Vec::<u8>::new();
                        cb.serialize(&mut ret).unwrap();
                        protocol_types::Base64URL::encode(ret).to_string()
                    };
                    assert_eq!(
                        super::from_callback(value, $t.to_string()),
                        $e
                    );
                )*
            }
        }


        let test_data_32: [u8; 32] = [1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2];
        let test_data_64: [u8; 64] = [
            1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,
            1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2
        ];

        assert_data_types!(
            "u8", 1u8, "1".to_string(),
            "u16", 123u16, "123".to_string(),
            "u32", 9898u32, "9898".to_string(),
            "u64", 9999999999999u64, "9999999999999".to_string(),
            "u128", 111199999999999u128, "111199999999999".to_string(),
            "i8", -1i8, "-1".to_string(),
            "i16", -123i16, "-123".to_string(),
            "i32", -9898i32, "-9898".to_string(),
            "i64", -9999999999999i64, "-9999999999999".to_string(),
            "i128", -111199999999999i128, "-111199999999999".to_string(),
            "bool", false, "false".to_string(),
            "String", "asdas".to_string(), "\"asdas\"".to_string(),
            "Vec<u8>", [0u8, 1u8, 2u8].to_vec(), "[0, 1, 2]".to_string(),
            "Vec<u16>", [99u16].to_vec(), "[99]".to_string(),
            "Vec<u32>", [0u32, 6u32].to_vec(), "[0, 6]".to_string(),
            "Vec<u64>", [0u64, 6123123123u64].to_vec(), "[0, 6123123123]".to_string(),
            "Vec<i8>", [0i8, 1i8, -2i8].to_vec(), "[0, 1, -2]".to_string(),
            "Vec<i16>", [-99i16].to_vec(), "[-99]".to_string(),
            "Vec<i32>", [0i32, 6i32].to_vec(), "[0, 6]".to_string(),
            "Vec<i64>", [-1i64, -6123123123i64].to_vec(), "[-1, -6123123123]".to_string(),
            "Vec<i128>", [-1i128, -1i128, -1i128, -1i128, -6123123123i128].to_vec(), "[-1, -1, -1, -1, -6123123123]".to_string(),
            "Vec<bool>", [true, false, false].to_vec(), "[true, false, false]".to_string(),
            "Vec<String>", ["true", "false", "false"].to_vec(), "[\"true\", \"false\", \"false\"]".to_string(),
            "[32]", test_data_32, format!("{:?}", test_data_32),
            "[64]", test_data_64, format!("{:?}", test_data_64),
        );


    }
    
}