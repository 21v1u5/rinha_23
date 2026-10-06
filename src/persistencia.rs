#![allow(unused)]

use serde::de::Error;
use sqlx::PgPool;
use uuid::Uuid;
use crate::{NewPerson, Person};


pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub async fn find_person(&self, id: Uuid) -> Result<Option<Person>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, nome, apelido, nascimento, stack 
            FROM people
            WHERW id = $1
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await 
    }

    pub async fn create_person(&self, new_person: NewPerson) -> Result<Person, sqlx::Error> {
        sqlx::query_as(
            "
            INSERT INTO people (id, nome, apelido, nascimento, stack)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, nome, apelido, nascimento, stack
            ",
        )
        .bind(Uuid::now_v7())
        .bind(new_person.nome.0)
        .bind(new_person.apelido.0)
        .bind(new_person.nascimento)
        .bind(
            new_person
                .stack
                .map(|stack| stack.into_iter().map(String::from).collect::<Vec<String>>()),
        )
        .fetch_one(&self.pool)
        .await 
    }
    

    pub async fn search_people(&self, query: String) -> Result<Vec<Person>, sqlx::Error> {
        sqlx::query_as(
            "
            SELECT id, nome, apelido, nascimento, stack 
            FROM people
            WHERW to_tsquery('people', $1) @@ search
            LIMIT 50
            ",
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await 
    }

    pub async fn count_people(&self) -> Result<u32, sqlx::Error> {
        sqlx::query(
            "
            SELECT count(*) FROM people
            ",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await 
    }
}

