use c8y_sdk::inventory::Inventory;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::platform::Platform;

pub struct MicroserviceSubscription {
    pub platforms: Arc<DashMap<String, Platform>>,
    pub subscription_listeners: Arc<Mutex<Vec<fn(plaform: Platform)>>>,
    pub unsubscription_listeners: Arc<Mutex<Vec<fn(plaform: Platform)>>>,
    c8y_bootstrap_tenant: String,
    c8y_bootstrap_user: String,
    c8y_bootstrap_password: String,
    c8y_baseurl: String,
}

impl MicroserviceSubscription {
    pub fn new() -> Self {
        MicroserviceSubscription {
            platforms: Arc::new(DashMap::new()),
            subscription_listeners: Arc::new(Mutex::new(Vec::new())),
            unsubscription_listeners: Arc::new(Mutex::new(Vec::new())),
            c8y_bootstrap_tenant: env::var("C8Y_BOOTSTRAP_TENANT").unwrap(),
            c8y_bootstrap_user: env::var("C8Y_BOOTSTRAP_USER").unwrap(),
            c8y_bootstrap_password: env::var("C8Y_BOOTSTRAP_PASSWORD").unwrap(),
            c8y_baseurl: env::var("C8Y_BASEURL").unwrap(),
        }
    }

    pub async fn add_subscription_listener(&self, f: fn(platform: Platform)) {
        self.subscription_listeners.lock().await.push(f);
    }

    pub async fn add_unsubscription_listener(&self, f: fn(platform: Platform)) {
        self.unsubscription_listeners.lock().await.push(f);
    }

    pub async fn send_new_subcription_event(&self, platform: Platform) {
        match self.subscription_listeners.lock().await {
            iter => {
                let i = iter.clone();
                for listener in i {
                    let p = platform.clone();
                    tokio::spawn(async move {
                        listener(p);
                    });
                }
            }
        }
    }

    pub async fn send_new_unsubcription_event(&self, platform: Platform) {
        match self.unsubscription_listeners.lock().await {
            iter => {
                let i = iter.clone();
                for listener in i {
                    let p = platform.clone();
                    tokio::spawn(async move {
                        listener(p);
                    });
                }
            }
        }
    }

    pub async fn start_subscription_listener(&self) {
        let sched = match JobScheduler::new().await {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };
        match sched
            .add(
                match Job::new_async("1/10 * * * * *", move |_uuid, _l| Box::pin(get_users())) {
                    Ok(j) => j,
                    Err(e) => panic!("{}", e),
                },
            )
            .await
        {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
        sched.shutdown_on_ctrl_c();
        match sched.start().await {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
    }
}

pub static SERVICE: Lazy<MicroserviceSubscription> = Lazy::new(|| MicroserviceSubscription::new());

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct User {
    name: String,
    password: String,
    tenant: String,
}

#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Users {
    pub users: Vec<User>,
}

async fn get_users() {
    println!("In get_users function");
    let response = Client::new()
        .get(format!(
            "{}/application/currentApplication/subscriptions",
            SERVICE.c8y_baseurl
        ))
        .basic_auth(
            format!(
                "{}/{}",
                SERVICE.c8y_bootstrap_tenant, SERVICE.c8y_bootstrap_user
            ),
            Some(SERVICE.c8y_bootstrap_password.clone()),
        )
        .header("accept", "application/json")
        .send()
        .await;
    match response {
        Ok(r) => {
            let result = r.json::<Users>().await;
            match result {
                Ok(users) => {
                    let mut subscribed = HashMap::new();
                    for p in SERVICE.platforms.iter() {
                        subscribed.insert(p.tenant.clone(), false);
                    }
                    for user in users.users.iter() {
                        let platform: Platform = Platform {
                            tenant: user.tenant.clone(),
                            username: user.name.clone(),
                            password: user.password.clone(),
                            base_url: SERVICE.c8y_baseurl.clone(),
                            inventory_api: Inventory {
                                tenant: user.tenant.clone(),
                                username: user.name.clone(),
                                password: user.password.clone(),
                                base_url: SERVICE.c8y_baseurl.clone(),
                            },
                        };
                        println!("Got user {}", user.name);
                        subscribed.insert(user.tenant.clone(), true);
                        let p = platform.clone();
                        match SERVICE.platforms.insert(user.tenant.clone(), platform) {
                            Some(_) => {
                                println!("Updating already subscribed tenant {}", user.tenant);
                            }
                            None => {
                                println!("New subscription from tenant {}", user.tenant);
                                SERVICE.send_new_subcription_event(p).await;
                            }
                        }
                    }
                    for s in subscribed.iter() {
                        if !*s.1 {
                            println!("Subscription removed: {}", s.0);
                            let p = SERVICE.platforms.remove(s.0).unwrap().1;
                            SERVICE.send_new_unsubcription_event(p).await;
                        }
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }
        Err(e) => panic!("{}", e),
    };
}
