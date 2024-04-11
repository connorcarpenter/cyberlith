use std::{any::TypeId, collections::HashMap};

use crate::{
    table::{Table, TableImpl},
    DbTableKey,
};

pub struct DatabaseWrapper {
    tables: HashMap<TypeId, Box<dyn Table>>,
}

impl DatabaseWrapper {
    pub fn init() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn table_open<K: DbTableKey>(&mut self) {
        let table_impl = TableImpl::<K>::init();
        self.tables.insert(TypeId::of::<K>(), Box::new(table_impl));
    }

    pub fn table<K: DbTableKey>(&self) -> &TableImpl<K> {
        let dyn_ref = self.tables.get(&TypeId::of::<K>()).unwrap();
        let any_ref = dyn_ref.to_any_ref();
        any_ref.downcast_ref::<TableImpl<K>>().unwrap()
    }

    pub fn table_mut<K: DbTableKey>(&mut self) -> &mut TableImpl<K> {
        let dyn_mut = self.tables.get_mut(&TypeId::of::<K>()).unwrap();
        let any_mut = dyn_mut.to_any_mut();
        any_mut.downcast_mut::<TableImpl<K>>().unwrap()
    }
}
