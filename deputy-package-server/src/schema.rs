// @generated automatically by Diesel CLI.

diesel::table! {
    categories (id) {
        id -> Binary,
        name -> Tinytext,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    package_categories (package_id, category_id) {
        package_id -> Binary,
        category_id -> Binary,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    packages (id) {
        id -> Binary,
        name -> Tinytext,
        package_type -> Nullable<Tinytext>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    versions (id) {
        id -> Binary,
        package_id -> Binary,
        version -> Tinytext,
        description -> Longtext,
        license -> Text,
        readme_html -> Longtext,
        checksum -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(versions -> packages (package_id));

diesel::allow_tables_to_appear_in_same_query!(
    categories,
    package_categories,
    packages,
    versions,
);
