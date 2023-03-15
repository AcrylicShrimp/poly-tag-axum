// @generated automatically by Diesel CLI.

diesel::table! {
    uploads (uuid) {
        uuid -> Uuid,
        created_at -> Timestamp,
    }
}
