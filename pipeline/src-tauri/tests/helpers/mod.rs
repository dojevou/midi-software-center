   /// Test helpers module
   ///
   /// Provides database helpers, macros, and utilities for repository testing.

pub mod db;
pub mod macros;

// Re-export commonly used items
pub use db::{
    assert_file_count, assert_file_exists, assert_file_not_exists, assert_metadata_exists,
    assert_tag_count, assert_tag_exists, cleanup_database, cleanup_table, count_file_tags,
    count_files, count_files_where, count_musical_metadata, count_tags, create_transaction,
    file_exists, file_tag_exists, get_file_by_id, get_metadata_by_file_id, get_tag_by_id,
    get_tags_for_file, metadata_exists, setup_test_pool, setup_test_pool_with_config, tag_exists,
    FileRow, MetadataRow, TagRow,
};
