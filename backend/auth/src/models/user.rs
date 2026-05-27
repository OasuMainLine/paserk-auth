use chrono::NaiveDateTime;
use diesel::{Selectable, deserialize::Queryable, prelude::Insertable};
use serde::{Deserialize, Serialize};

use crate::utils::paseto::UserClaims;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: Vec<u8>,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewUser {
    pub id: Option<i32>,
    pub username: String,
    pub email: String,
    pub password_hash: Vec<u8>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::schema::users)]
pub struct PartialUser {
    pub id: i32,
    pub username: String,
    pub email: String,
}

impl From<User> for PartialUser {
    fn from(value: User) -> Self {
        return Self {
            email: value.email,
            id: value.id,
            username: value.username,
        };
    }
}

impl Into<UserClaims> for PartialUser {
    fn into(self) -> UserClaims {
        UserClaims {
            email: self.email,
            id: self.id,
            username: self.username,
        }
    }
}
