use serde::ser::SerializeStruct;
use serde::{Deserializer, Serializer};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

///Snowflakes algorithm
#[derive(Debug)]
pub struct SnowflakeGenerator {
    pub epoch: i64,
    pub worker_id: i64,
    pub datacenter_id: i64,
    pub sequence: AtomicI64,
}

impl serde::Serialize for SnowflakeGenerator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Snowflake", 5)?;
        s.serialize_field("epoch", &self.epoch)?;
        s.serialize_field("worker_id", &self.worker_id)?;
        s.serialize_field("datacenter_id", &self.datacenter_id)?;
        s.serialize_field("sequence", &self.sequence.load(Ordering::Relaxed))?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for SnowflakeGenerator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Snowflake {
            pub epoch: i64,
            pub worker_id: i64,
            pub datacenter_id: i64,
            pub sequence: i64,
        }
        let proxy = Snowflake::deserialize(deserializer)?;
        Ok(self::SnowflakeGenerator {
            epoch: proxy.epoch,
            worker_id: proxy.worker_id,
            datacenter_id: proxy.datacenter_id,
            sequence: AtomicI64::from(proxy.sequence),
        })
    }
}

impl Clone for SnowflakeGenerator {
    fn clone(&self) -> Self {
        let sequence = self.sequence.load(Ordering::Relaxed);
        Self {
            epoch: self.epoch,
            worker_id: self.worker_id,
            datacenter_id: self.datacenter_id,
            sequence: AtomicI64::new(sequence),
        }
    }
}

impl Default for SnowflakeGenerator {
    fn default() -> Self {
        SnowflakeGenerator {
            epoch: 1_564_790_400_000,
            worker_id: 1,
            datacenter_id: 1,
            sequence: AtomicI64::new(0),
        }
    }
}

impl SnowflakeGenerator {
    pub fn new(epoch: i64, worker_id: i64, datacenter_id: i64) -> SnowflakeGenerator {
        SnowflakeGenerator {
            epoch,
            worker_id,
            datacenter_id,
            sequence: AtomicI64::new(0),
        }
    }

    pub fn set_epoch(&mut self, epoch: i64) -> &mut Self {
        self.epoch = epoch;
        self
    }

    pub fn set_worker_id(&mut self, worker_id: i64) -> &mut Self {
        self.worker_id = worker_id;
        self
    }

    pub fn set_datacenter_id(&mut self, datacenter_id: i64) -> &mut Self {
        self.datacenter_id = datacenter_id;
        self
    }

    pub fn generate(&self) -> i64 {
        let timestamp = self.get_time();
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        (timestamp << 22) | (self.worker_id << 17) | (self.datacenter_id << 12) | sequence
    }

    fn get_time(&self) -> i64 {
        let since_the_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis() as i64 - self.epoch
    }
}

#[async_trait::async_trait]
impl crate::IdentifierGenerator for SnowflakeGenerator {
    async fn next_id(&self) -> crate::Value {
        let id = self.generate();
        crate::Value::Bigint(Some(id))
    }
}
