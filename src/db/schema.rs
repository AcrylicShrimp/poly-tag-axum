// @generated automatically by Diesel CLI.

diesel::table! {
    files (uuid) {
        uuid -> Uuid,
        file_name -> Text,
        file_type -> Text,
        file_hash -> Bpchar,
        file_size -> Int8,
        created_at -> Timestamp,
    }
}

diesel::table! {
    stagings (uuid) {
        uuid -> Uuid,
        staged_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    files,
    stagings,
);
