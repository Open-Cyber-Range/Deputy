// @generated automatically by Diesel CLI.

diesel::table! {
    packages (id) {
        id -> Binary,
        version -> Nullable<Tinytext>,
        license -> Nullable<Text>,
        readme_path -> Nullable<Mediumtext>,
        readme_html -> Nullable<Longtext>,
        checksum -> Nullable<Text>,
        name -> Tinytext,
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

diesel::allow_tables_to_appear_in_same_query!(packages, versions,);
