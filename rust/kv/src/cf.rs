//use paste::paste;

#[macro_export]
macro_rules! count {
  ($name:ident) => { 1 };
  ($first:ident, $($rest:ident),*) => {
    1 + count!($($rest),*)
  }
}

pub struct ColumnFamily(pub *mut librocksdb_sys::rocksdb_column_family_handle_t);

impl rocksdb::AsColumnFamilyRef for ColumnFamily {
  fn inner(&self) -> *mut librocksdb_sys::rocksdb_column_family_handle_t {
    self.0
  }
}

unsafe impl Send for ColumnFamily {}
unsafe impl Sync for ColumnFamily {}

#[macro_export]
macro_rules! column_family {

  ($($name:ident),*) => {
    pub const CF_LI:[&str;count!($($name),+)] = [$($name),*];
    $(
      #[allow(non_upper_case_globals)]
      pub const $name: &str = stringify!($name);
    )*

      pub struct Cf {
        $( pub $name:cf::ColumnFamily ),*
      }

    pub fn cf_all(db:&rocksdb::OptimisticTransactionDB) -> Cf {
      use rocksdb::AsColumnFamilyRef;
      Cf {
        $(
          $name:cf::ColumnFamily(db.cf_handle($name).unwrap().inner())
        ),*
      }
    }
  }

}
