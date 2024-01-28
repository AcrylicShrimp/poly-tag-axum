// @generated automatically by Diesel CLI.

diesel::table! {
    collection_file_pairs (collection_id, file_id) {
        collection_id -> Int4,
        file_id -> Int4,
    }
}

diesel::table! {
    collections (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Text,
        description -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    files (id) {
        id -> Int4,
        uuid -> Uuid,
        name -> Text,
        mime -> Nullable<Text>,
        size -> Nullable<Int8>,
        hash -> Nullable<Int8>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    tags (id) {
        id -> Int8,
        file_id -> Int4,
        title -> Text,
        value -> Nullable<Text>,
    }
}

diesel::joinable!(collection_file_pairs -> collections (collection_id));
diesel::joinable!(collection_file_pairs -> files (file_id));
diesel::joinable!(tags -> files (file_id));

diesel::allow_tables_to_appear_in_same_query!(
    collection_file_pairs,
    collections,
    files,
    tags,
);
