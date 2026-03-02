use sled::{Db, transaction};
use std::collections::HashSet;
use std::path::Path;

use crate::error::{DatabaseError, TransactionError};

/// 数据库封装
pub struct Database {
    db: Db,
}

impl Database {
    /// 初始化数据库
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    /// 创建命名空间（模块专用）
    pub fn with_namespace(&self, namespace: &str) -> NamespacedDatabase {
        let prefix = format!("{}:", namespace).into_bytes();
        NamespacedDatabase {
            db: self.db.clone(),
            prefix,
        }
    }

    /// 基本键值操作
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let value = self.db.get(key)?;
        Ok(value.map(|v| v.to_vec()))
    }

    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        self.db.insert(key, value)?;
        Ok(())
    }

    pub fn remove(&self, key: &[u8]) -> Result<(), DatabaseError> {
        self.db.remove(key)?;
        Ok(())
    }

    /// 批量操作
    pub fn batch(&self) -> sled::Batch {
        sled::Batch::default()
    }

    /// 遍历键值对
    pub fn iter(&self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> {
        self.db
            .iter()
            .filter_map(|item| item.ok().map(|(k, v)| (k.to_vec(), v.to_vec())))
    }

    /// 集合操作（用于列表/标签）
    pub fn add_to_set(&self, set_key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let key = [set_key, b":", value].concat();
        self.db.insert(key, b"1")?;
        Ok(())
    }

    pub fn remove_from_set(&self, set_key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let key = [set_key, b":", value].concat();
        self.db.remove(key)?;
        Ok(())
    }

    pub fn get_set(&self, set_key: &[u8]) -> Result<HashSet<Vec<u8>>, DatabaseError> {
        let prefix = [set_key, b":"].concat();
        let mut set = HashSet::new();
        let prefix_slice = prefix.as_slice();
        for item in self.db.scan_prefix(prefix.clone()) {
            let (key, _) = item?;
            let value = key.as_ref().strip_prefix(prefix_slice).map(|v| v.to_vec());
            if let Some(v) = value {
                set.insert(v);
            }
        }
        Ok(set)
    }

    /// 事务支持
    pub fn transaction<F, A, E>(&self, f: F) -> Result<A, TransactionError>
    where
        F: for<'a> Fn(&'a sled::transaction::TransactionalTree) -> transaction::ConflictableTransactionResult<A, E>,
        E: ToString + From<sled::transaction::TransactionError> + std::fmt::Debug,
    {
        self.db
            .transaction(f)
            .map_err(|e| TransactionError::Abort(format!("{:?}", e)))
    }

}
/// 带命名空间的数据库（模块使用）
pub struct NamespacedDatabase {
    db: Db,
    prefix: Vec<u8>,
}

impl NamespacedDatabase {
    /// 带前缀的键
    fn prefixed_key(&self, key: &[u8]) -> Vec<u8> {
        [self.prefix.clone(), key.to_vec()].concat()
    }

    /// 基本键值操作（带命名空间）
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, DatabaseError> {
        let prefixed = self.prefixed_key(key);
        let value = self.db.get(prefixed)?;
        Ok(value.map(|v| v.to_vec()))
    }

    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), DatabaseError> {
        let prefixed = self.prefixed_key(key);
        self.db.insert(prefixed, value)?;
        Ok(())
    }

    pub fn remove(&self, key: &[u8]) -> Result<(), DatabaseError> {
        let prefixed = self.prefixed_key(key);
        self.db.remove(prefixed)?;
        Ok(())
    }

    /// 序列化插入
    pub fn insert_json<T: serde::Serialize>(
        &self,
        key: &[u8],
        value: &T,
    ) -> Result<(), DatabaseError> {
        let json = serde_json::to_vec(value)?;
        self.insert(key, &json)
    }

    /// 序列化获取
    pub fn get_json<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &[u8],
    ) -> Result<Option<T>, DatabaseError> {
        if let Some(data) = self.get(key)? {
            let value = serde_json::from_slice(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// 遍历键值对（带命名空间）
    pub fn iter(&self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> {
        let prefix = self.prefix.clone();
        self.db.scan_prefix(prefix.as_slice()).filter_map(move |item| {
            item.ok().and_then(|(k, v)| {
                k.as_ref().strip_prefix(prefix.as_slice())
                    .map(|sk| (sk.to_vec(), v.to_vec()))
            })
        })
    }
}
