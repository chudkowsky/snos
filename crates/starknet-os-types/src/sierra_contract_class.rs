use std::cell::OnceCell;
use std::rc::Rc;

use cairo_vm::Felt252;
use pathfinder_gateway_types::class_hash::compute_class_hash;
use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starknet_core::types::contract::ComputeClassHashError;

use crate::casm_contract_class::{CairoLangCasmClass, GenericCasmContractClass};
use crate::error::ContractClassError;
use crate::hash::GenericClassHash;
use crate::starknet_core_addons::starknet_core_sierra_class_from_flattened;

pub type CairoLangSierraContractClass = cairo_lang_starknet_classes::contract_class::ContractClass;
pub type FlattenedStarknetCoreSierraContractClass = starknet_core::types::FlattenedSierraClass;
pub type StarknetCoreSierraContractClass = starknet_core::types::contract::SierraClass;

/// A generic Sierra contract class that supports conversion to/from the most commonly used
/// contract class types in Starknet and provides utility methods.
/// Operations are implemented as lazily as possible, i.e. we only convert
/// between different types if strictly necessary.
/// Fields are boxed in an RC for cheap cloning.
#[derive(Debug, Clone)]
pub struct GenericSierraContractClass {
    cairo_lang_contract_class: OnceCell<Rc<CairoLangSierraContractClass>>,
    starknet_core_contract_class: OnceCell<Rc<StarknetCoreSierraContractClass>>,
    serialized_class: OnceCell<Vec<u8>>,
    class_hash: OnceCell<GenericClassHash>,
}

impl GenericSierraContractClass {
    pub fn from_bytes(serialized_class: Vec<u8>) -> Self {
        Self {
            cairo_lang_contract_class: Default::default(),
            starknet_core_contract_class: Default::default(),
            serialized_class: OnceCell::from(serialized_class),
            class_hash: OnceCell::new(),
        }
    }

    fn build_cairo_lang_class(&self) -> Result<CairoLangSierraContractClass, ContractClassError> {
        if let Ok(serialized_class) = self.get_serialized_contract_class() {
            let contract_class = serde_json::from_slice(serialized_class)?;
            return Ok(contract_class);
        }

        Err(ContractClassError::NoPossibleConversion)
    }

    pub fn get_serialized_contract_class(&self) -> Result<&Vec<u8>, ContractClassError> {
        self.serialized_class.get_or_try_init(|| serde_json::to_vec(self)).map_err(Into::into)
    }

    fn build_starknet_core_class(&self) -> Result<StarknetCoreSierraContractClass, ContractClassError> {
        let serialized_class = self.get_serialized_contract_class()?;
        serde_json::from_slice(serialized_class).map_err(Into::into)
    }
    pub fn get_cairo_lang_contract_class(&self) -> Result<&CairoLangSierraContractClass, ContractClassError> {
        self.cairo_lang_contract_class
            .get_or_try_init(|| self.build_cairo_lang_class().map(Rc::new))
            .map(|boxed| boxed.as_ref())
    }

    pub fn get_starknet_core_contract_class(&self) -> Result<&StarknetCoreSierraContractClass, ContractClassError> {
        self.starknet_core_contract_class
            .get_or_try_init(|| self.build_starknet_core_class().map(Rc::new))
            .map(|boxed| boxed.as_ref())
    }

    pub fn to_cairo_lang_contract_class(self) -> Result<CairoLangSierraContractClass, ContractClassError> {
        let cairo_lang_class = self.get_cairo_lang_contract_class()?;
        Ok(cairo_lang_class.clone())
    }

    pub fn to_starknet_core_contract_class(self) -> Result<StarknetCoreSierraContractClass, ContractClassError> {
        let starknet_core_class = self.get_starknet_core_contract_class()?;
        Ok(starknet_core_class.clone())
    }

    pub fn to_flattened_starknet_core_contract_class(
        self,
    ) -> Result<FlattenedStarknetCoreSierraContractClass, ContractClassError> {
        let starknet_core_class = self.to_starknet_core_contract_class()?;
        starknet_core_class.flatten().map_err(|e| ContractClassError::SerdeError(serde::ser::Error::custom(e)))
    }

    fn compute_class_hash(&self) -> Result<GenericClassHash, ContractClassError> {
        // if we have a starknet_core type, we can ask it for a class_hash without any type conversion
        if let Some(sn_core_cc) = self.starknet_core_contract_class.get() {
            let class_hash = sn_core_cc.as_ref().class_hash().map_err(|e| match e {
                ComputeClassHashError::Json(json_error) => {
                    ContractClassError::SerdeError(serde_json::Error::custom(json_error))
                }
                _ => panic!("Unexpected class hash computation error: {e}"),
            })?;
            Ok(GenericClassHash::new(class_hash.into()))
        } else {
            // otherwise, we have a cairo_lang contract_class which we can serialize and then
            // deserialize via ContractClassForPathfinderCompat
            // TODO: improve resilience and performance
            let contract_class = self.get_cairo_lang_contract_class()?;
            let contract_class_compat = ContractClassForPathfinderCompat::from(contract_class.clone());

            let contract_dump =
                serde_json::to_vec(&contract_class_compat).expect("JSON serialization failed unexpectedly.");
            let computed_class_hash = compute_class_hash(&contract_dump)
                .map_err(|e| ContractClassError::HashError(format!("Failed to compute class hash: {}", e)))?;

            Ok(GenericClassHash::from_bytes_be(computed_class_hash.hash().0.to_be_bytes()))
        }
    }

    pub fn class_hash(&self) -> Result<GenericClassHash, ContractClassError> {
        self.class_hash.get_or_try_init(|| self.compute_class_hash()).copied()
    }

    pub fn compile(&self) -> Result<GenericCasmContractClass, ContractClassError> {
        let cairo_lang_class = self.get_cairo_lang_contract_class()?.clone();
        // Values taken from the defaults of `starknet-sierra-compile`, see here:
        // https://github.com/starkware-libs/cairo/blob/main/crates/bin/starknet-sierra-compile/src/main.rs
        let add_pythonic_hints = false;
        let max_bytecode_size = 180000;
        let casm_contract_class =
            CairoLangCasmClass::from_contract_class(cairo_lang_class, add_pythonic_hints, max_bytecode_size)?;

        Ok(GenericCasmContractClass::from(casm_contract_class))
    }
}

#[derive(Debug, Serialize)]
struct ContractClassForPathfinderCompat {
    pub sierra_program: Vec<Felt252>,
    pub contract_class_version: String,
    pub entry_points_by_type: cairo_lang_starknet_classes::contract_class::ContractEntryPoints,
    pub abi: String,
}

impl From<cairo_lang_starknet_classes::contract_class::ContractClass> for ContractClassForPathfinderCompat {
    fn from(value: cairo_lang_starknet_classes::contract_class::ContractClass) -> Self {
        Self {
            sierra_program: value.sierra_program.into_iter().map(|x| Felt252::from(x.value)).collect(),
            contract_class_version: value.contract_class_version,
            entry_points_by_type: value.entry_points_by_type,
            abi: value.abi.map(|abi| abi.json()).unwrap_or_default(),
        }
    }
}

impl Serialize for GenericSierraContractClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(cairo_lang_class) = self.cairo_lang_contract_class.get() {
            cairo_lang_class.serialize(serializer)
        } else if let Some(starknet_core_class) = self.starknet_core_contract_class.get() {
            starknet_core_class.serialize(serializer)
        } else {
            Err(S::Error::custom("No possible serialization"))
        }
    }
}

impl<'de> Deserialize<'de> for GenericSierraContractClass {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let cairo_lang_class = CairoLangSierraContractClass::deserialize(deserializer)?;
        Ok(Self::from(cairo_lang_class))
    }
}

impl From<CairoLangSierraContractClass> for GenericSierraContractClass {
    fn from(cairo_lang_class: CairoLangSierraContractClass) -> Self {
        Self {
            cairo_lang_contract_class: OnceCell::from(Rc::new(cairo_lang_class)),
            starknet_core_contract_class: Default::default(),
            serialized_class: Default::default(),
            class_hash: Default::default(),
        }
    }
}

impl From<StarknetCoreSierraContractClass> for GenericSierraContractClass {
    fn from(starknet_core_class: StarknetCoreSierraContractClass) -> Self {
        Self {
            cairo_lang_contract_class: Default::default(),
            starknet_core_contract_class: OnceCell::from(Rc::new(starknet_core_class)),
            serialized_class: Default::default(),
            class_hash: Default::default(),
        }
    }
}

impl From<FlattenedStarknetCoreSierraContractClass> for GenericSierraContractClass {
    fn from(flattened_contract_class: FlattenedStarknetCoreSierraContractClass) -> Self {
        let sierra_class = starknet_core_sierra_class_from_flattened(flattened_contract_class);
        Self::from(sierra_class)
    }
}

impl TryFrom<GenericSierraContractClass> for StarknetCoreSierraContractClass {
    type Error = ContractClassError;

    fn try_from(contract_class: GenericSierraContractClass) -> Result<Self, Self::Error> {
        contract_class.to_starknet_core_contract_class()
    }
}

#[cfg(test)]
mod tests {
    use starknet_core::types::contract::SierraClass;

    use super::*;

    const SIERRA_CLASS: &[u8] = include_bytes!(
        "../../../tests/integration/contracts/blockifier_contracts/feature_contracts/cairo1/compiled/test_contract.\
         sierra"
    );

    /// Tests that generating a Starknet Core class from a generic class created from a cairo-lang
    /// class works.
    #[test]
    fn test_convert_cairo_lang_class_to_starknet_core_class() {
        let cairo_lang_class: CairoLangSierraContractClass = serde_json::from_slice(SIERRA_CLASS).unwrap();
        let generic_class = GenericSierraContractClass::from(cairo_lang_class);

        let _starknet_core_class = generic_class.to_starknet_core_contract_class().unwrap();
    }

    #[test]
    fn test_try_from_flattened_sierra_class() {
        let starknet_core_class: SierraClass = serde_json::from_slice(SIERRA_CLASS).unwrap();
        let flattened_class = starknet_core_class.flatten().unwrap();

        let generic_class = GenericSierraContractClass::from(flattened_class);
        let _generated_starknet_core_class = generic_class.to_starknet_core_contract_class();
    }

    #[test]
    fn test_compile_from_starknet_core_class() {
        let starknet_core_class: SierraClass = serde_json::from_slice(SIERRA_CLASS).unwrap();
        let generic_class = GenericSierraContractClass::from(starknet_core_class);

        let starknet_core_class = generic_class.to_starknet_core_contract_class().unwrap();
        let generic_class = GenericSierraContractClass::from(starknet_core_class);

        let _casm_class = generic_class.compile().unwrap_or_else(|e| panic!("failed to compile class: {e}"));
    }
}
