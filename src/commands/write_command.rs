// Copyright 2015-2018 Aerospike, Inc.
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
use std::time::Duration;

use errors::*;
use Bin;
use Key;
use ResultCode;
use cluster::{Cluster, Node};
use commands::buffer;
use commands::{Command, SingleCommand};
use net::Connection;
use operations::OperationType;
use policy::WritePolicy;

pub struct WriteCommand<'a, A: 'a> {
    single_command: SingleCommand<'a>,
    policy: &'a WritePolicy,
    bins: &'a [A],
    operation: OperationType,
}

impl<'a, 'b, A: AsRef<Bin<'b>>> WriteCommand<'a, A> {
    pub fn new(
        policy: &'a WritePolicy,
        cluster: Arc<Cluster>,
        key: &'a Key,
        bins: &'a [A],
        operation: OperationType,
    ) -> Self {
        WriteCommand {
            single_command: SingleCommand::new(cluster, key),
            bins: bins,
            policy: policy,
            operation: operation,
        }
    }

    pub fn execute(&mut self) -> Result<()> {
        SingleCommand::execute(self.policy, self)
    }
}

impl<'a, 'b, A: AsRef<Bin<'b>>> Command for WriteCommand<'a, A> {
    fn write_timeout(&mut self, conn: &mut Connection, timeout: Option<Duration>) -> Result<()> {
        conn.buffer.write_timeout(timeout);
        Ok(())
    }

    fn write_buffer(&mut self, conn: &mut Connection) -> Result<()> {
        conn.flush()
    }

    fn prepare_buffer(&mut self, conn: &mut Connection) -> Result<()> {
        conn.buffer.set_write(
            self.policy,
            self.operation,
            self.single_command.key,
            self.bins,
        )
    }

    fn get_node(&self) -> Result<Arc<Node>> {
        self.single_command.get_node()
    }

    fn parse_result(&mut self, conn: &mut Connection) -> Result<()> {
        // Read header.
        if let Err(err) = conn.read_buffer(buffer::MSG_TOTAL_HEADER_SIZE as usize) {
            warn!("Parse result error: {}", err);
            return Err(err);
        }

        try!(conn.buffer.reset_offset());

        let result_code = ResultCode::from(try!(conn.buffer.read_u8(Some(13))) & 0xFF);
        if result_code != ResultCode::Ok {
            bail!(ErrorKind::ServerError(result_code));
        }

        SingleCommand::empty_socket(conn)
    }
}
