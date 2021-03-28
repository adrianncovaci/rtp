use super::schema::users;
use crate::actor::actor::*;
use diesel::{Insertable, Queryable};
#[derive(Queryable, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub user_id: String,
    pub username: &'a str,
}

impl Message for User {
    type Result = ();
}
