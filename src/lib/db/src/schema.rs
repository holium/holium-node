// @generated automatically by Diesel CLI.

diesel::table! {
    passports (ship) {
        ship -> Text,
        is_public -> Bool,
        nickname -> Text,
        color -> Text,
        twitter -> Nullable<Text>,
        bio -> Nullable<Text>,
        avatar -> Nullable<Text>,
        cover -> Nullable<Text>,
        featured_url -> Nullable<Text>,
        phone_number -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}
