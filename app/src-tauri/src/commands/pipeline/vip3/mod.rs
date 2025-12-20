//! VIP3 Browser Commands - Multi-filter file search, favorites, saved searches, collections
//!
//! This module provides a comprehensive browser interface for MIDI files with:
//! - Multi-filter search with dynamic WHERE clause building
//! - Favorites management (toggle, set, list)
//! - Saved searches with use tracking
//! - Collections with file ordering
//! - Category management (timbres, styles, articulations)
//! - Lookup endpoints for filter dropdowns
//! - Bulk retag for high-performance category assignment
//! - Dynamic filter counts that update based on current selections

pub mod bulk_retag;
pub mod categories;
pub mod collections;
pub mod favorites;
pub mod filter_counts;
pub mod lookups;
pub mod saved_searches;
pub mod search;
pub mod types;

// Re-export all public types
pub use types::{
    CollectionResponse, CreateCollectionRequest, CreateSavedSearchRequest, FileCategoriesResponse,
    FilterOption, SavedSearchResponse, Vip3FileResult, Vip3FilterCounts, Vip3Filters,
    Vip3SearchResults, Vip3Sort,
};

// Re-export all commands
pub use bulk_retag::{bulk_retag_vip3, update_vip3_counts, BulkRetagResponse};
pub use categories::{
    add_articulation_to_file, add_style_to_file, add_timbre_to_file, get_file_categories,
    get_vip3_articulations, get_vip3_folders, get_vip3_instruments, get_vip3_manufacturers,
    get_vip3_styles, get_vip3_timbres, remove_articulation_from_file, remove_style_from_file,
    remove_timbre_from_file, get_all_vip3_categories, get_vip3_analytics_counts, AllVip3Categories
};
pub use collections::{
    add_file_to_collection, batch_add_files_to_collection, batch_remove_files_from_collection,
    clear_collection, create_collection, delete_collection, get_collection, get_collection_files,
    get_collections, get_smart_collection_files, remove_file_from_collection, reorder_collection_files,
    update_collection,
};
pub use favorites::{get_favorite_count, get_favorites, set_favorite, toggle_favorite};
pub use filter_counts::get_vip3_dynamic_filter_counts;
pub use lookups::{
    get_all_articulations, get_all_bpm_ranges, get_all_musical_keys, get_all_styles,
    get_all_timbres,
};
pub use saved_searches::{
    delete_saved_search, get_saved_searches, load_saved_search, save_search,
    toggle_saved_search_pin,
};
pub use search::{get_vip3_filter_counts, search_files_vip3};
