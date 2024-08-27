use casper_engine_test_support::LmdbWasmTestBuilder;
use casper_types::{bytesrepr::FromBytes, CLTyped, EntityAddr, Key};

pub(crate) fn get_dictionary_value_from_key<T: CLTyped + FromBytes>(
    builder: &LmdbWasmTestBuilder,
    contract_key: &Key,
    dictionary_name: &str,
    dictionary_key: &str,
) -> T {
    let named_key = match contract_key.into_entity_hash() {
        Some(hash) => {
            let entity_with_named_keys = builder
                .get_entity_with_named_keys_by_entity_hash(hash)
                .expect("should be named key from entity hash");
            let named_keys = entity_with_named_keys.named_keys();
            named_keys
                .get(dictionary_name)
                .expect("must have key")
                .to_owned()
        }
        None => match contract_key.into_hash_addr() {
            Some(contract_key) => {
                let named_keys = builder.get_named_keys(EntityAddr::SmartContract(contract_key));
                named_keys
                    .get(dictionary_name)
                    .expect("must have key")
                    .to_owned()
            }
            None => {
                panic!("unsupported dictionary location")
            }
        },
    };

    let seed_uref = named_key.as_uref().expect("must convert to seed uref");

    builder
        .query_dictionary_item(None, *seed_uref, dictionary_key)
        .expect("should have dictionary value")
        .as_cl_value()
        .expect("T should be CLValue")
        .to_owned()
        .into_t()
        .unwrap()
}
