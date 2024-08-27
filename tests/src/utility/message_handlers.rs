use casper_engine_test_support::LmdbWasmTestBuilder;
use casper_types::{
    contract_messages::{MessageChecksum, MessageTopicSummary, TopicNameHash},
    AddressableEntity, AddressableEntityHash, Digest, EntityAddr, Key, StoredValue,
};

pub fn entity(
    builder: &LmdbWasmTestBuilder,
    contract_hash: &AddressableEntityHash,
) -> AddressableEntity {
    let query_result = builder
        .query(None, Key::contract_entity_key(*contract_hash), &[])
        .expect("should query");

    if let StoredValue::AddressableEntity(entity) = query_result {
        entity
    } else {
        panic!(
            "Stored value is not an addressable entity: {:?}",
            query_result
        );
    }
}

pub fn message_topic(
    builder: &LmdbWasmTestBuilder,
    contract_hash: &AddressableEntityHash,
    topic_name_hash: TopicNameHash,
) -> MessageTopicSummary {
    let query_result = builder
        .query(
            None,
            Key::message_topic(
                EntityAddr::new_smart_contract(contract_hash.value()),
                topic_name_hash,
            ),
            &[],
        )
        .expect("should query");

    match query_result {
        StoredValue::MessageTopic(summary) => summary,
        _ => {
            panic!(
                "Stored value is not a message topic summary: {:?}",
                query_result
            );
        }
    }
}

pub fn message_summary(
    builder: &LmdbWasmTestBuilder,
    contract_hash: &AddressableEntityHash,
    topic_name_hash: &TopicNameHash,
    message_index: u32,
    state_hash: Option<Digest>,
) -> Result<MessageChecksum, String> {
    let query_result = builder.query(
        state_hash,
        Key::message(
            EntityAddr::new_smart_contract(contract_hash.value()),
            *topic_name_hash,
            message_index,
        ),
        &[],
    )?;

    match query_result {
        StoredValue::Message(summary) => Ok(summary),
        _ => panic!("Stored value is not a message summary: {:?}", query_result),
    }
}
