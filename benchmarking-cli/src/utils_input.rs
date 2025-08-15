use miden_vm::{
    crypto::{MerkleStore, MerkleTree, SimpleSmt},
    math::Felt,
    utils::collections::BTreeMap,
    AdviceInputs, MemAdviceProvider, StackInputs, StackOutputs, Word,
};

/// The Outputs struct is used to serialize the output of the program.
/// Via Rust WASM we cannot return arbitrary structs, so we need to serialize it to JSON.
/// Here we need the Outputs because they can be inputs for the verifier.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Outputs {
    pub stack_output: Vec<u64>,
    pub trace_len: Option<usize>,
    pub proof: Option<Vec<u8>>,
}

// MERKLE DATA
// ================================================================================================

/// Struct used to deserialize merkle data from input file. Merkle data can be represented as a
/// merkle tree or a sparse merkle tree.
#[derive(serde::Deserialize)]
pub enum MerkleData {
    /// String representation of a merkle tree.  The merkle tree is represented as a vector of
    /// 32 byte hex strings where each string represents a leaf in the tree.
    #[serde(rename = "merkle_tree")]
    MerkleTree(Vec<String>),
    /// String representation of a sparse merkle tree. The sparse merkle tree is represented as a
    /// vector of tuples where each tuple consists of a u64 node index and a 32 byte hex string
    /// representing the value of the node.
    #[serde(rename = "sparse_merkle_tree")]
    SparseMerkleTree(Vec<(u64, String)>),
}

// INPUT FILE
// ================================================================================================

// TODO consider using final types instead of string representations.
/// Input file struct that is used to deserialize input data from file. It consists of four
/// components:
/// - operand_stack
/// - advice_stack
/// - advice_map
/// - merkle_store
#[derive(serde::Deserialize)]
pub struct InputFile {
    /// String representation of the initial operand stack, composed of chained field elements.
    pub operand_stack: Option<Vec<String>>,
    /// Optional string representation of the initial advice stack, composed of chained field
    /// elements.
    pub advice_stack: Option<Vec<String>>,
    /// Optional map of 32 byte hex strings to vectors of u64s representing the initial advice map.
    pub advice_map: Option<BTreeMap<String, Vec<u64>>>,
    /// Optional vector of merkle data which will be loaded into the initial merkle store. Merkle
    /// data is represented as 32 byte hex strings and node indexes are represented as u64s.
    pub merkle_store: Option<Vec<MerkleData>>,
}

/// Helper methods to interact with the input file
impl InputFile {
    /// Parse advice provider data from the input file.
    pub fn parse_advice_provider(&self) -> Result<MemAdviceProvider, String> {
        let mut advice_inputs = AdviceInputs::default();

        let stack = self
            .parse_advice_stack()
            .map_err(|e| format!("failed to parse advice provider: {e}"))?;
        advice_inputs = advice_inputs
            .with_stack_values(stack)
            .map_err(|e| e.to_string())?;

        if let Some(map) = self
            .parse_advice_map()
            .map_err(|e| format!("failed to parse advice provider: {e}"))?
        {
            advice_inputs = advice_inputs.with_map(map);
        }

        if let Some(merkle_store) = self
            .parse_merkle_store()
            .map_err(|e| format!("failed to parse advice provider: {e}"))?
        {
            advice_inputs = advice_inputs.with_merkle_store(merkle_store);
        }

        Ok(MemAdviceProvider::from(advice_inputs))
    }

    /// Parse advice stack data from the input file.
    fn parse_advice_stack(&self) -> Result<Vec<u64>, String> {
        self.advice_stack
            .as_ref()
            .map(Vec::as_slice)
            .unwrap_or(&[])
            .iter()
            .map(|v| {
                v.parse::<u64>()
                    .map_err(|e| format!("failed to parse advice stack value `{v}` - {e}"))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// Parse advice map data from the input file.
    fn parse_advice_map(&self) -> Result<Option<BTreeMap<[u8; 32], Vec<Felt>>>, String> {
        let advice_map = match &self.advice_map {
            Some(advice_map) => advice_map,
            None => return Ok(None),
        };

        let map = advice_map
            .iter()
            .map(|(k, v)| {
                // decode hex key
                let mut key = [0u8; 32];
                hex::decode_to_slice(k, &mut key)
                    .map_err(|e| format!("failed to decode advice map key `{k}` - {e}"))?;

                // convert values to Felt
                let values = v
                    .iter()
                    .map(|v| {
                        Felt::try_from(*v).map_err(|e| {
                            format!("failed to convert advice map value `{v}` to Felt - {e}")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((key, values))
            })
            .collect::<Result<BTreeMap<[u8; 32], Vec<Felt>>, String>>()?;

        Ok(Some(map))
    }

    /// Parse merkle store data from the input file.
    fn parse_merkle_store(&self) -> Result<Option<MerkleStore>, String> {
        let merkle_data = match &self.merkle_store {
            Some(merkle_data) => merkle_data,
            None => return Ok(None),
        };

        let mut merkle_store = MerkleStore::default();
        for data in merkle_data {
            match data {
                MerkleData::MerkleTree(data) => {
                    let leaves = parse_merkle_tree(data)?;
                    let merkle_tree = MerkleTree::new(leaves)
                        .map_err(|e| format!("failed to add merkle tree to merkle store - {e}"))?;
                    merkle_store.extend(merkle_tree.inner_nodes());
                }
                MerkleData::SparseMerkleTree(data) => {
                    let entries = parse_sparse_merkle_tree(data)?;
                    // TODO: Support variable depth
                    let smt =
                        SimpleSmt::with_leaves(SimpleSmt::MAX_DEPTH, entries).map_err(|e| {
                            format!("failed to add sparse merkle tree to merkle store - {e}")
                        })?;
                    merkle_store.extend(smt.inner_nodes());
                }
            }
        }

        Ok(Some(merkle_store))
    }

    /// Parse and return the stack inputs for the program.
    pub fn parse_stack_inputs(&self) -> Result<StackInputs, String> {
        let operand_stack = match self.operand_stack.as_ref() {
            Some(operand_stack) => operand_stack,
            None => return Ok(StackInputs::default()),
        };

        let stack_inputs = operand_stack
            .iter()
            .map(|v| v.parse::<u64>().map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;

        StackInputs::try_from_values(stack_inputs).map_err(|e| e.to_string())
    }
}

/// Miden Inputs plus Outputs that are used as inputs for the verifier.
pub struct Inputs {
    pub stack_inputs: StackInputs,
    pub advice_provider: MemAdviceProvider,
    #[allow(dead_code)]
    pub stack_outputs: StackOutputs,
}

/// We need to implement the default trait for the Inputs struct.
impl Inputs {
    pub fn new() -> Self {
        Self {
            stack_inputs: StackInputs::new(vec![]),
            advice_provider: MemAdviceProvider::default(),
            stack_outputs: StackOutputs::new(vec![], vec![]).unwrap(),
        }
    }

    pub fn deserialize_inputs(&mut self, inputs: &str) -> Result<(), String> {
        if !inputs.trim().is_empty() {
            let inputs_des: InputFile = serde_json::from_str(inputs).map_err(|e| e.to_string())?;

            self.stack_inputs = inputs_des.parse_stack_inputs().unwrap();
            self.advice_provider = inputs_des.parse_advice_provider().unwrap();
        }
        Ok(())
    }
}

/// Parse and return merkle tree leaves.
fn parse_merkle_tree(tree: &[String]) -> Result<Vec<Word>, String> {
    tree.iter()
        .map(|v| {
            let leaf = parse_word(v)?;
            Ok(leaf)
        })
        .collect()
}

/// Parse and return sparse merkle tree entries.
fn parse_sparse_merkle_tree(tree: &[(u64, String)]) -> Result<Vec<(u64, Word)>, String> {
    tree.iter()
        .map(|(index, v)| {
            let leaf = parse_word(v)?;
            Ok((*index, leaf))
        })
        .collect()
}

/// Parse a `Word` from a hex string.
pub fn parse_word(word_hex: &str) -> Result<Word, String> {
    let mut word_data = [0u8; 32];
    hex::decode_to_slice(word_hex, &mut word_data)
        .map_err(|e| format!("failed to decode `Word` from hex {word_hex} - {e}"))?;
    let mut word = Word::default();
    for (i, value) in word_data.chunks(8).enumerate() {
        word[i] = Felt::try_from(value).map_err(|e| {
            format!("failed to convert `Word` data {word_hex} (element {i}) to Felt - {e}")
        })?;
    }
    Ok(word)
}

#[test]
fn test_parse_output() {
    let output_str: &str = r#"
    {
        "stack_output": [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        "trace_len": 1024
    }"#;

    let mut inputs: Inputs = Inputs::new();
    inputs.deserialize_inputs(output_str).unwrap();

    let output: StackOutputs = inputs.stack_outputs;

    assert_eq!(
        output.stack(),
        vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
}
