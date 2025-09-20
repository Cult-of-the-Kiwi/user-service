use async_std::stream::StreamExt;
use dotenvy::var;
use fluvio::{Fluvio, Offset, consumer::ConsumerConfigExtBuilder};
use serde_json::from_slice;
use topic_structs::UserCreated;

use crate::{api_utils::structs::User, sql_utils::calls::UserRepository};

pub async fn run<T: UserRepository>(fluvio: Fluvio, db: T) -> anyhow::Result<()> {
    //TODO! do a proper fix on this
    let auth_registered_consumer_topic = var("AUTH_REGISTER_TOPIC")
        .unwrap_or("auth-register".to_owned())
        .trim()
        .to_string();

    let consumer_config = ConsumerConfigExtBuilder::default()
        .topic(auth_registered_consumer_topic)
        .offset_start(Offset::beginning())
        .build()
        .expect("Failed to build consumer config");

    let mut consumer_stream = fluvio.consumer_with_config(consumer_config).await?;

    while let Some(Ok(record)) = consumer_stream.next().await {
        let parse_result = from_slice::<UserCreated>(record.value());

        if let Ok(user_created) = &parse_result {
            let user = User {
                id: user_created.id.clone(),
                username: user_created.username.clone(),
                created_at: None,
            };
            if db.get_user(&user.id).await.is_some() {
                //FIXME! User already exists, big time error
                continue;
            }

            if db.insert_user(&user).await.is_err() {
                //TODO! IDK, panic I guess
            }
        }
    }

    Ok(())
}
