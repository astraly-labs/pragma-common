// Taken from:
// https://github.com/dojoengine/dojo/blob/34b13caa785c1149558d28f1a9d9fbd700c4aa2d/crates/torii/libp2p/src/typed_data.rs

use std::str::FromStr;

use cainome::cairo_serde::ByteArray;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use starknet::core::types::Felt;
use starknet::core::utils::{cairo_short_string_to_felt, get_selector_from_name};
use starknet_crypto::poseidon_hash_many;

use crate::starknet::errors::SignerError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimpleField {
    pub name: String,
    pub r#type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParentField {
    pub name: String,
    pub r#type: String,
    pub contains: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Field {
    ParentType(ParentField),
    SimpleType(SimpleField),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveType {
    // All of object types. Including preset types
    Object(IndexMap<String, PrimitiveType>),
    Array(Vec<PrimitiveType>),
    Bool(bool),
    // comprehensive representation of
    // String, ShortString, Selector and Felt
    String(String),
    // For JSON numbers. Formed into a Felt
    Number(Number),
}

fn get_preset_types() -> IndexMap<String, Vec<Field>> {
    let mut types = IndexMap::new();

    types.insert(
        "TokenAmount".to_string(),
        vec![
            Field::SimpleType(SimpleField {
                name: "token_address".to_string(),
                r#type: "ContractAddress".to_string(),
            }),
            Field::SimpleType(SimpleField {
                name: "amount".to_string(),
                r#type: "u256".to_string(),
            }),
        ],
    );

    types.insert(
        "NftId".to_string(),
        vec![
            Field::SimpleType(SimpleField {
                name: "collection_address".to_string(),
                r#type: "ContractAddress".to_string(),
            }),
            Field::SimpleType(SimpleField {
                name: "token_id".to_string(),
                r#type: "u256".to_string(),
            }),
        ],
    );

    types.insert(
        "u256".to_string(),
        vec![
            Field::SimpleType(SimpleField {
                name: "low".to_string(),
                r#type: "u128".to_string(),
            }),
            Field::SimpleType(SimpleField {
                name: "high".to_string(),
                r#type: "u128".to_string(),
            }),
        ],
    );

    types
}

// Get the fields of a specific type
// Looks up both the types hashmap as well as the preset types
// Returns the fields and the hashmap of types
fn get_fields(name: &str, types: &IndexMap<String, Vec<Field>>) -> Result<Vec<Field>, SignerError> {
    if let Some(fields) = types.get(name) {
        return Ok(fields.clone());
    }

    Err(SignerError::InvalidMessage(format!(
        "Type {name} not found",
    )))
}

fn get_dependencies(
    name: &str,
    types: &IndexMap<String, Vec<Field>>,
    dependencies: &mut Vec<String>,
) -> Result<(), SignerError> {
    if dependencies.contains(&name.to_string()) {
        return Ok(());
    }

    dependencies.push(name.to_string());

    for field in get_fields(name, types)? {
        let mut field_type = match field {
            Field::SimpleType(simple_field) => simple_field.r#type.clone(),
            Field::ParentType(parent_field) => parent_field.contains.clone(),
        };

        field_type = field_type.trim_end_matches('*').to_string();

        if types.contains_key(&field_type) && !dependencies.contains(&field_type) {
            get_dependencies(&field_type, types, dependencies)?;
        }
    }

    Ok(())
}

pub fn encode_type(
    name: &str,
    types: &IndexMap<String, Vec<Field>>,
) -> Result<String, SignerError> {
    let mut type_hash = String::new();

    // get dependencies
    let mut dependencies: Vec<String> = Vec::new();
    get_dependencies(name, types, &mut dependencies)?;

    let (_, rest) = dependencies
        .split_first_mut()
        .ok_or_else(|| SignerError::InvalidMessage("No dependencies found".to_string()))?;
    rest.sort_by_key(|dep| dep.to_lowercase());

    for dep in dependencies {
        type_hash += &format!("\"{dep}\"");

        type_hash += "(";

        let fields = get_fields(&dep, types)?;
        for (idx, field) in fields.iter().enumerate() {
            match field {
                Field::SimpleType(simple_field) => {
                    // if ( at start and ) at end
                    if simple_field.r#type.starts_with('(') && simple_field.r#type.ends_with(')') {
                        let inner_types = &simple_field.r#type[1..simple_field.r#type.len() - 1]
                            .split(',')
                            .map(|t| {
                                if t.is_empty() {
                                    t.to_string()
                                } else {
                                    format!("\"{t}\"")
                                }
                            })
                            .collect::<Vec<String>>()
                            .join(",");
                        type_hash += &format!("\"{}\":({})", simple_field.name, inner_types);
                    } else {
                        type_hash +=
                            &format!("\"{}\":\"{}\"", simple_field.name, simple_field.r#type);
                    }
                }
                Field::ParentType(parent_field) => {
                    type_hash +=
                        &format!("\"{}\":\"{}\"", parent_field.name, parent_field.contains);
                }
            }

            if idx < fields.len() - 1 {
                type_hash += ",";
            }
        }

        type_hash += ")";
    }

    Ok(type_hash)
}

#[derive(Debug, Default)]
pub struct Ctx {
    pub base_type: String,
    pub parent_type: String,
    pub is_preset: bool,
}

pub(crate) struct FieldInfo {
    _name: String,
    r#type: String,
    base_type: String,
    index: usize,
}

pub(crate) fn get_value_type(
    name: &str,
    types: &IndexMap<String, Vec<Field>>,
) -> Result<FieldInfo, SignerError> {
    // iter both "types" and "preset_types" to find the field
    for (idx, (key, value)) in types.iter().enumerate() {
        if key == name {
            return Ok(FieldInfo {
                _name: name.to_string(),
                r#type: key.clone(),
                base_type: String::new(),
                index: idx,
            });
        }

        for (idx, field) in value.iter().enumerate() {
            match field {
                Field::SimpleType(simple_field) => {
                    if simple_field.name == name {
                        return Ok(FieldInfo {
                            _name: name.to_string(),
                            r#type: simple_field.r#type.clone(),
                            base_type: String::new(),
                            index: idx,
                        });
                    }
                }
                Field::ParentType(parent_field) => {
                    if parent_field.name == name {
                        return Ok(FieldInfo {
                            _name: name.to_string(),
                            r#type: parent_field.contains.clone(),
                            base_type: parent_field.r#type.clone(),
                            index: idx,
                        });
                    }
                }
            }
        }
    }

    Err(SignerError::InvalidMessage(format!(
        "Field {name} not found in types",
    )))
}

fn get_hex(value: &str) -> Result<Felt, SignerError> {
    Felt::from_str(value).map_or_else(
        |_| {
            cairo_short_string_to_felt(value).map_err(|e| {
                SignerError::InvalidMessage(format!("Invalid shortstring for felt: {e}"))
            })
        },
        Ok,
    )
}

impl PrimitiveType {
    pub fn encode(
        &self,
        r#type: &str,
        types: &IndexMap<String, Vec<Field>>,
        ctx: &mut Ctx,
    ) -> Result<Felt, SignerError> {
        match self {
            Self::Object(obj) => {
                ctx.is_preset = types.contains_key(r#type);

                let mut hashes = Vec::new();

                if ctx.base_type == "enum" {
                    let (variant_name, value) = obj.first().ok_or_else(|| {
                        SignerError::InvalidMessage("Enum value must be populated".to_string())
                    })?;
                    let variant_type = get_value_type(variant_name, types)?;

                    let arr: &Vec<Self> = match value {
                        Self::Array(arr) => arr,
                        _ => {
                            return Err(SignerError::InvalidMessage(
                                "Enum value must be an array".to_string(),
                            ));
                        }
                    };

                    // variant index
                    hashes.push(Felt::from(variant_type.index as u32));

                    // variant parameters
                    for (idx, param) in arr.iter().enumerate() {
                        let field_type = &variant_type
                            .r#type
                            .trim_start_matches('(')
                            .trim_end_matches(')')
                            .split(',')
                            .nth(idx)
                            .ok_or_else(|| {
                                SignerError::InvalidMessage("Invalid enum variant type".to_string())
                            })?;

                        let field_hash = param.encode(field_type, types, ctx)?;
                        hashes.push(field_hash);
                    }

                    return Ok(poseidon_hash_many(hashes.as_slice()));
                }

                let type_hash = encode_type(r#type, types)?;
                hashes.push(get_selector_from_name(&type_hash).map_err(|e| {
                    SignerError::InvalidMessage(format!("Invalid type {type} for selector: {e}",))
                })?);

                for (field_name, value) in obj {
                    // recheck if we're currently in a preset type
                    ctx.is_preset = types.contains_key(r#type);

                    // pass correct types - preset or types
                    let field_type = get_value_type(field_name, types)?;
                    ctx.base_type = field_type.base_type;
                    ctx.parent_type = r#type.to_string();
                    let field_hash = value.encode(field_type.r#type.as_str(), types, ctx)?;
                    hashes.push(field_hash);
                }

                Ok(poseidon_hash_many(hashes.as_slice()))
            }
            Self::Array(array) => Ok(poseidon_hash_many(
                array
                    .iter()
                    .map(|x| x.encode(r#type.trim_end_matches('*'), types, ctx))
                    .collect::<Result<Vec<_>, _>>()?
                    .as_slice(),
            )),
            Self::Bool(boolean) => {
                let v = if *boolean {
                    Felt::from(1_u32)
                } else {
                    Felt::from(0_u32)
                };
                Ok(v)
            }
            Self::String(string) => match r#type {
                "felt" | "shortstring" | "ContractAddress" | "ClassHash" | "timestamp" | "u128"
                | "i128" => get_hex(string),
                "string" => {
                    // split the string into short strings and encode
                    let byte_array = ByteArray::from_string(string).map_err(|e| {
                        SignerError::InvalidMessage(format!("Invalid string for bytearray: {e}"))
                    })?;

                    let mut hashes = vec![Felt::from(byte_array.data.len())];

                    for hash in byte_array.data {
                        hashes.push(hash.felt());
                    }

                    hashes.push(byte_array.pending_word);
                    hashes.push(Felt::from(byte_array.pending_word_len));

                    Ok(poseidon_hash_many(hashes.as_slice()))
                }
                "selector" => get_selector_from_name(string)
                    .map_err(|e| SignerError::InvalidMessage(format!("Invalid selector: {e}"))),
                _ => Err(SignerError::InvalidMessage(format!(
                    "Invalid type {type} for string",
                ))),
            },
            Self::Number(number) => {
                let felt = Felt::from_str(&number.to_string())
                    .map_err(|_| SignerError::InvalidMessage(format!("Invalid number {number}")))?;
                Ok(felt)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
    pub version: String,
    #[serde(rename = "chainId")]
    pub chain_id: String,
    pub revision: Option<String>,
}

impl Domain {
    pub fn new(name: &str, version: &str, chain_id: &str, revision: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            chain_id: chain_id.to_string(),
            revision: revision.map(ToString::to_string),
        }
    }

    pub fn encode(&self, types: &IndexMap<String, Vec<Field>>) -> Result<Felt, SignerError> {
        let mut object = IndexMap::new();

        object.insert("name".to_string(), PrimitiveType::String(self.name.clone()));
        object.insert(
            "version".to_string(),
            PrimitiveType::String(self.version.clone()),
        );
        object.insert(
            "chainId".to_string(),
            PrimitiveType::String(self.chain_id.clone()),
        );
        if let Some(revision) = &self.revision {
            object.insert(
                "revision".to_string(),
                PrimitiveType::String(revision.clone()),
            );
        }

        // we dont need to pass our preset types here. domain should never use a preset type
        PrimitiveType::Object(object).encode("StarknetDomain", types, &mut Default::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypedData {
    pub types: IndexMap<String, Vec<Field>>,
    #[serde(rename = "primaryType")]
    pub primary_type: String,
    pub domain: Domain,
    pub message: IndexMap<String, PrimitiveType>,
}

/// Breakdown of components that make up a typed message hash.
#[derive(Debug, Clone)]
pub struct TypedDataHash {
    /// The final hash of the entire message.
    pub hash: Felt,
    /// Hash of the `domain_separator` component.
    #[allow(dead_code)]
    pub domain_separator_hash: Felt,
    /// Hash of the `message` component.
    #[allow(dead_code)]
    pub message_hash: Felt,
}

impl TypedData {
    pub fn new(
        types: IndexMap<String, Vec<Field>>,
        primary_type: &str,
        domain: Domain,
        message: IndexMap<String, PrimitiveType>,
    ) -> Self {
        Self {
            types,
            primary_type: primary_type.to_string(),
            domain,
            message,
        }
    }

    pub fn encode(&self, account: Felt) -> Result<TypedDataHash, SignerError> {
        let preset_types = get_preset_types();

        // Combine types and preset_types
        let mut all_types = preset_types;
        all_types.extend(self.types.clone());

        if self.domain.revision.clone().unwrap_or("1".to_string()) != "1" {
            return Err(SignerError::InvalidMessage(
                "Legacy revision 0 is not supported".to_string(),
            ));
        }

        let prefix_message = cairo_short_string_to_felt("StarkNet Message").unwrap();

        // encode domain separator
        let domain_hash = self.domain.encode(&self.types)?;

        // encode message
        let message_hash = PrimitiveType::Object(self.message.clone()).encode(
            &self.primary_type,
            &all_types,
            &mut Default::default(),
        )?;

        // return full hash
        Ok(TypedDataHash {
            hash: poseidon_hash_many(
                vec![prefix_message, domain_hash, account, message_hash].as_slice(),
            ),
            domain_separator_hash: domain_hash,
            message_hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use starknet::core::types::Felt;
    use starknet::core::utils::starknet_keccak;

    use super::*;

    #[rstest]
    #[case(EXAMPLE_BASE_TYPES)]
    #[case(EXAMPLE_ENUM)]
    #[case(EXAMPLE_PRESET_TYPES)]
    #[case(MAIL_STRUCT_ARRAY)]
    fn test_read_json(#[case] json_data: &str) {
        let reader = std::io::BufReader::new(json_data.as_bytes());

        let typed_data: TypedData = serde_json::from_reader(reader).unwrap();

        // Optionally, you can assert certain properties of `typed_data`
        // For now, we'll just ensure that deserialization succeeds
        assert!(!typed_data.types.is_empty());
        assert!(!typed_data.primary_type.is_empty());
    }

    #[rstest]
    #[case(
        EXAMPLE_BASE_TYPES,
        "\"Example\"(\"n0\":\"felt\",\"n1\":\"bool\",\"n2\":\"string\",\"n3\":\"selector\",\"n4\":\"u128\",\"n5\":\"i128\",\"n6\":\"ContractAddress\",\"n7\":\"ClassHash\",\"n8\":\"timestamp\",\"n9\":\"shortstring\")"
    )]
    #[case(
        MAIL_STRUCT_ARRAY,
        "\"Mail\"(\"from\":\"Person\",\"to\":\"Person\",\"posts_len\":\"felt\",\"posts\":\"Post*\")\"Person\"(\"name\":\"felt\",\"wallet\":\"felt\")\"Post\"(\"title\":\"felt\",\"content\":\"felt\")"
    )]
    #[case(
        EXAMPLE_ENUM,
        "\"Example\"(\"someEnum\":\"MyEnum\")\"MyEnum\"(\"Variant 1\":(),\"Variant 2\":(\"u128\",\"u128*\"),\"Variant 3\":(\"u128\"))"
    )]
    #[case(
        EXAMPLE_PRESET_TYPES,
        "\"Example\"(\"n0\":\"TokenAmount\",\"n1\":\"NftId\")"
    )]
    fn test_type_encode(#[case] json_data: &str, #[case] expected_encoded: &str) {
        let reader = std::io::BufReader::new(json_data.as_bytes());

        let typed_data: TypedData = serde_json::from_reader(reader).unwrap();

        let encoded = encode_type(&typed_data.primary_type, &typed_data.types).unwrap();

        assert_eq!(encoded, expected_encoded);
    }

    #[test]
    fn test_selector_encode() {
        let selector = PrimitiveType::String("transfer".to_string());
        let selector_hash = PrimitiveType::String(starknet_keccak(b"transfer").to_string());

        let preset_types = get_preset_types();

        let encoded_selector = selector
            .encode("selector", &preset_types, &mut Default::default())
            .unwrap();
        let raw_encoded_selector = selector_hash
            .encode("felt", &preset_types, &mut Default::default())
            .unwrap();

        assert_eq!(encoded_selector, raw_encoded_selector);
        assert_eq!(encoded_selector, starknet_keccak(b"transfer"));
    }

    #[test]
    fn test_domain_hash() {
        let reader = std::io::BufReader::new(EXAMPLE_BASE_TYPES.as_bytes());

        let typed_data: TypedData = serde_json::from_reader(reader).unwrap();

        let domain_hash = typed_data.domain.encode(&typed_data.types).unwrap();

        assert_eq!(
            domain_hash,
            Felt::from_hex("0x555f72e550b308e50c1a4f8611483a174026c982a9893a05c185eeb85399657")
                .unwrap()
        );
    }

    #[rstest]
    #[case(
        EXAMPLE_BASE_TYPES,
        "0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826",
        "0x2d80b87b8bc32068247c779b2ef0f15f65c9c449325e44a9df480fb01eb43ec"
    )]
    #[case(
        EXAMPLE_ENUM,
        "0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826",
        "0x3df10475ad5a8f49db4345a04a5b09164d2e24b09f6e1e236bc1ccd87627cc"
    )]
    #[case(
        EXAMPLE_PRESET_TYPES,
        "0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826",
        "0x185b339d5c566a883561a88fb36da301051e2c0225deb325c91bb7aa2f3473a"
    )]
    #[case(
        MAIL_STRUCT_ARRAY,
        "0xcd2a3d9f938e13cd947ec05abc7fe734df8dd826",
        "0x1df06fd32d689b5431a784a33b02314cc7f395f3bda3ecedf97deafaa66ea31"
    )]
    fn test_message_hash(
        #[case] example_data: &str,
        #[case] address_hex: &str,
        #[case] expected_hash_hex: &str,
    ) {
        let address = Felt::from_str(address_hex).unwrap();

        let reader = std::io::BufReader::new(example_data.as_bytes());

        let typed_data: TypedData = serde_json::from_reader(reader).unwrap();

        let message_hash = typed_data.encode(address).unwrap().hash;

        assert_eq!(message_hash, Felt::from_str(expected_hash_hex).unwrap());
    }

    const EXAMPLE_BASE_TYPES: &str = r#"
{
  "types": {
    "StarknetDomain": [
      { "name": "name", "type": "shortstring" },
      { "name": "version", "type": "shortstring" },
      { "name": "chainId", "type": "shortstring" },
      { "name": "revision", "type": "shortstring" }
    ],
    "Example": [
      { "name": "n0", "type": "felt" },
      { "name": "n1", "type": "bool" },
      { "name": "n2", "type": "string" },
      { "name": "n3", "type": "selector" },
      { "name": "n4", "type": "u128" },
      { "name": "n5", "type": "i128" },
      { "name": "n6", "type": "ContractAddress" },
      { "name": "n7", "type": "ClassHash" },
      { "name": "n8", "type": "timestamp" },
      { "name": "n9", "type": "shortstring" }
    ]
  },
  "primaryType": "Example",
  "domain": {
    "name": "StarkNet Mail",
    "version": "1",
    "chainId": "1",
    "revision": "1"
  },
  "message": {
    "n0": "0x3e8",
    "n1": true,
    "n2": "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    "n3": "transfer",
    "n4": "0x3e8",
    "n5": "-170141183460469231731687303715884105727",
    "n6": "0x3e8",
    "n7": "0x3e8",
    "n8": 1000,
    "n9": "transfer"
  }
}"#;

    const EXAMPLE_ENUM: &str = r#"
{
  "types": {
    "StarknetDomain": [
      { "name": "name", "type": "shortstring" },
      { "name": "version", "type": "shortstring" },
      { "name": "chainId", "type": "shortstring" },
      { "name": "revision", "type": "shortstring" }
    ],
    "Example": [{ "name": "someEnum", "type": "enum", "contains": "MyEnum" }],
    "MyEnum": [
      { "name": "Variant 1", "type": "()" },
      { "name": "Variant 2", "type": "(u128,u128*)" },
      { "name": "Variant 3", "type": "(u128)" }
    ]
  },
  "primaryType": "Example",
  "domain": {
    "name": "StarkNet Mail",
    "version": "1",
    "chainId": "1",
    "revision": "1"
  },
  "message": {
    "someEnum": {
      "Variant 2": [2, [0, 1]]
    }
  }
}"#;

    const EXAMPLE_PRESET_TYPES: &str = r#"
{
  "types": {
    "StarknetDomain": [
      { "name": "name", "type": "shortstring" },
      { "name": "version", "type": "shortstring" },
      { "name": "chainId", "type": "shortstring" },
      { "name": "revision", "type": "shortstring" }
    ],
    "Example": [
      { "name": "n0", "type": "TokenAmount" },
      { "name": "n1", "type": "NftId" }
    ]
  },
  "primaryType": "Example",
  "domain": {
    "name": "StarkNet Mail",
    "version": "1",
    "chainId": "1",
    "revision": "1"
  },
  "message": {
    "n0": {
      "token_address": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
      "amount": {
        "low": "0x3e8",
        "high": "0x0"
      }
    },
    "n1": {
      "collection_address": "0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
      "token_id": {
        "low": "0x3e8",
        "high": "0x0"
      }
    }
  }
}"#;

    const MAIL_STRUCT_ARRAY: &str = r#"
{
  "types": {
    "StarknetDomain": [
      { "name": "name", "type": "shortstring" },
      { "name": "version", "type": "shortstring" },
      { "name": "chainId", "type": "shortstring" },
      { "name": "revision", "type": "shortstring" }
    ],
    "Person": [
      { "name": "name", "type": "felt" },
      { "name": "wallet", "type": "felt" }
    ],
    "Post": [
      { "name": "title", "type": "felt" },
      { "name": "content", "type": "felt" }
    ],
    "Mail": [
      { "name": "from", "type": "Person" },
      { "name": "to", "type": "Person" },
      { "name": "posts_len", "type": "felt" },
      { "name": "posts", "type": "Post*" }
    ]
  },
  "primaryType": "Mail",
  "domain": {
    "name": "StarkNet Mail",
    "version": "1",
    "chainId": "1",
    "revision": "1"
  },
  "message": {
    "from": {
      "name": "Cow",
      "wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
    },
    "to": {
      "name": "Bob",
      "wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
    },
    "posts_len": 2,
    "posts": [
      { "title": "Greeting", "content": "Hello, Bob!" },
      { "title": "Farewell", "content": "Goodbye, Bob!" }
    ]
  }
}"#;
}
