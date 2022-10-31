// @generated automatically by Diesel CLI.

diesel::table! {
    packages (id) {
        id -> Int4,
        name -> Varchar,
        version -> Varchar,
        readme -> Text,
        licence -> Varchar,
        created_at -> Timestamp,
    }
}
