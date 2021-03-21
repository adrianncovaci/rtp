use super::schema::users;
use diesel::{Insertable, Queryable};
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub username: &'a str,
}