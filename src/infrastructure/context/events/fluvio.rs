use std::env::var;

use fluvio::{Fluvio, FluvioConfig, metadata::topic::TopicSpec};

pub async fn new_fluvio(addr: &str) -> anyhow::Result<Fluvio> {
    let mut fluvio_config = FluvioConfig::new(addr);
    fluvio_config.use_spu_local_address = true;

    let fluvio = fluvio::Fluvio::connect_with_config(&fluvio_config).await?;

    let auth_registered_consumer_topic = var("AUTH_REGISTER_TOPIC")
        .unwrap_or("auth-register".to_owned())
        .trim()
        .to_string();

    let request_producer_topic = var("USER_RESQUEST_TOPIC")
        .unwrap_or("friendships-request".to_owned())
        .trim()
        .to_string();

    let answered_producer_topic = var("USER_ANSWER_TOPIC")
        .unwrap_or("friendships-answer".to_owned())
        .trim()
        .to_string();

    let admin = fluvio.admin().await;

    let topics = admin
        .all::<TopicSpec>()
        .await
        .expect("Failed to list topics");
    let topic_names = topics
        .iter()
        .map(|topic| topic.name.clone())
        .collect::<Vec<String>>();

    //Creates topic if they dont exist

    if !topic_names.contains(&request_producer_topic) {
        let topic_spec = TopicSpec::new_computed(1, 1, None);
        admin
            .create(request_producer_topic.clone(), false, topic_spec)
            .await?;
    }

    if !topic_names.contains(&answered_producer_topic) {
        let topic_spec = TopicSpec::new_computed(1, 1, None);
        admin
            .create(answered_producer_topic.clone(), false, topic_spec)
            .await?;
    }

    if !topic_names.contains(&auth_registered_consumer_topic) {
        let topic_spec = TopicSpec::new_computed(1, 1, None);
        admin
            .create(auth_registered_consumer_topic.clone(), false, topic_spec)
            .await?;
    }

    Ok(fluvio)
}
