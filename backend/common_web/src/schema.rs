table! {
    feeditems (id) {
        id -> Int4,
        created_by -> Varchar,
        image_id -> Varchar,
        caption -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (email) {
        id -> Int4,
        email -> Varchar,
        password_hash -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(
    feeditems,
    users,
);
