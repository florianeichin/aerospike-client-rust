// Copyright 2015-2017 Aerospike, Inc.
//
// Portions may be licensed to Aerospike, Inc. under one or more contributor
// license agreements.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not
// use this file except in compliance with the License. You may obtain a copy of
// the License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations under
// the License.

use std::env;
use std::sync::Arc;

use rand;
use rand::Rng;

use aerospike::{Client, ClientPolicy};

lazy_static! {
    pub static ref AEROSPIKE_HOSTS: String = env::var("AEROSPIKE_HOSTS").unwrap_or(String::from("127.0.0.1"));

    pub static ref AEROSPIKE_NAMESPACE: String = env::var("AEROSPIKE_NAMESPACE").unwrap_or(String::from("test"));

    pub static ref GLOBAL_CLIENT_POLICY: ClientPolicy = {
        let mut policy = ClientPolicy::default();
        if let Ok(user) = env::var("AEROSPIKE_USER") {
            let password = env::var("AEROSPIKE_PASSWORD").unwrap_or(String::new());
            policy.set_user_password(user, password).unwrap();
        }
        policy
    };

    pub static ref GLOBAL_CLIENT: Arc<Client> = {
        Arc::new(Client::new(&GLOBAL_CLIENT_POLICY, &*AEROSPIKE_HOSTS).unwrap())
    };
}

pub fn rand_str(sz: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(sz).collect()
}