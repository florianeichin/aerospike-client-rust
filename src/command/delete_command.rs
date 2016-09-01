// Copyright 2013-2016 Aerospike, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::Arc;
use std::io::Write;
use std::collections::{HashMap};
use std::time::{Instant, Duration};
use std::str;

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt, ByteOrder};

use net::Connection;
use error::{AerospikeError, ResultCode, AerospikeResult};
use value::{Value};

use net::Host;
use cluster::node_validator::NodeValidator;
use cluster::partition_tokenizer::PartitionTokenizer;
use cluster::partition::Partition;
use cluster::{Node, Cluster};
use common::{Key, Record, OperationType, FieldType, ParticleType, Bin};
use policy::{ClientPolicy, WritePolicy, Policy, ConsistencyLevel};
use common::operation;
use command::command::Command;
use command::single_command::SingleCommand;
use command::buffer;
use command::buffer::{Buffer};
use value::value;

pub struct DeleteCommand<'a> {
    single_command: SingleCommand<'a>,

    policy: &'a WritePolicy,
    pub existed: bool,
}

impl<'a> DeleteCommand<'a> {

    pub fn new(policy: &'a WritePolicy, cluster: Arc<Cluster>, key: &'a Key<'a>) -> AerospikeResult<Self> {
        Ok(DeleteCommand {
            single_command: try!(SingleCommand::new(cluster, key)),

            policy: policy,
            existed: false,
        })
    }

    pub fn execute(&mut self) -> AerospikeResult<()> {
        SingleCommand::execute(self.policy, self)
    }

}

impl<'a> Command for DeleteCommand<'a> {

    fn write_timeout(&mut self, conn: &mut Connection, timeout: Option<Duration>) -> AerospikeResult<()> {
        conn.buffer.write_timeout(timeout);
        Ok(())
    }

    fn write_buffer(&mut self, conn: &mut Connection) -> AerospikeResult<()> {
        conn.flush()
    }

    fn prepare_buffer(&mut self, conn: &mut Connection) -> AerospikeResult<()> {
        conn.buffer.set_delete(self.policy, self.single_command.key)
    }

    fn get_node(&self) -> AerospikeResult<Arc<Node>> {
        self.single_command.get_node()
    }

    fn parse_result(&mut self, conn: &mut Connection) -> AerospikeResult<()> {
        // Read header.
        if let Err(err) = conn.read_buffer(buffer::MSG_TOTAL_HEADER_SIZE as usize) {
            warn!("Parse result error: {}", err);
            return Err(err);
        }

        try!(conn.buffer.reset_offset());

        // A number of these are commented out because we just don't care enough to read
        // that section of the header. If we do care, uncomment and check!
        let result_code = (try!(conn.buffer.read_u8(Some(13))) & 0xFF) as isize;

        if result_code != 0 && result_code != ResultCode::KEY_NOT_FOUND_ERROR {
            return Err(AerospikeError::new(result_code, None));
        }

        self.existed = result_code == 0;

        SingleCommand::empty_socket(conn)
    }

 }