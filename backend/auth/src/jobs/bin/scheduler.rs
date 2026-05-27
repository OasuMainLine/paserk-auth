use std::{str::FromStr, sync::Arc};

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Monitor, Storage, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronStream, Schedule};
use apalis_redis::RedisStorage;
use auth::{
    config::Config,
    jobs::{
        generate_key_pair_job::{GenerateKeyPairJob, generate_key_pair},
        schedule_state::InnerScheduleState,
    },
};
use log::info;
use redis::Client;
use shared::env::load_env;

#[tokio::main]
async fn main() {
    load_env().expect("Error loading env variables for scheduler");
    let config = envy::from_env::<Config>().expect("Error loading config for scheduler");
    let conn = apalis_redis::connect(config.auth_redis_url.clone())
        .await
        .expect("Error loading redis backend for scheduler");
    let redis_client =
        Client::open(config.auth_redis_url.clone()).expect("Unable to reach redis_client");
    let schedule_state = Arc::new(InnerScheduleState {
        redis: redis_client,
        config: config.clone(),
    });
    let mut storage = RedisStorage::new(conn);

    let schedule = Schedule::from_str("0 0 0 * * *").unwrap();
    let worker = WorkerBuilder::new("generate_key_pair_worker")
        .data(schedule_state)
        .concurrency(2)
        .backend(CronStream::new(schedule).pipe_to_storage(storage.clone()))
        .build_fn(generate_key_pair);

    // Push a GenerateKeyPairJob so it executes immediately when starting the scheduler
    storage
        .push(GenerateKeyPairJob)
        .await
        .expect("Error scheduling GenerateKeyPairJob");

    let monitor = Monitor::new()
        .register(worker)
        .on_event(|e| info!("Started worker: {}", e.id()));

    println!("Scheduler started");
    monitor.run().await.expect("Scheduler failed");
}
