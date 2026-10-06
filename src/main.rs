use std::{collections::HashMap, sync::Arc};
use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post},
};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;
use time::{Date, macros::date};

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "apelido")]
    pub apelido: String,
    #[serde(rename = "nome")]
    pub nome: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub nascimento: Date,
    pub stack: Option<Vec<String>>,
}

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "apelido")]
    pub apelido: Nick,
    #[serde(rename = "nome")]
    pub nome: PersonName,
    #[serde(rename = "nascimento", with = "date_format")]
    pub nascimento: Date,
    pub stack: Option<Vec<Tech>>,
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PersonName(String);

impl TryFrom<String> for PersonName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <=100 {
            Ok(PersonName(value))
        } else {
            Err("Nome muito grande")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Nick(String);

impl TryFrom<String> for Nick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <=32 {
            Ok(Nick(value))
        } else {
            Err("Apelido muito grande")
        }
    }
}

#[derive(Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct Tech(String);

impl TryFrom<String> for Tech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() <=32 {
            Ok(Self(value))
        } else {
            Err("Tecnologia muito grande")
        }
    }
}

impl From<Tech> for String {
    fn from(value: Tech) -> Self {
        value.0
    }
}





type AppState = Arc<Mutex<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let mut people: HashMap<Uuid, Person> = HashMap::new();

    let person1 = Person {
        id: Uuid::now_v7(),
        apelido: "Lívius".to_string(),
        nome: "livinho".to_string(),
        nascimento: date!(1990 - 01 - 01),
        stack: Some(vec!["Rust".to_string(), "JavaScript".to_string()]),
    };

    println!("Person 1: {}", person1.id);

    people.insert(person1.id, person1);

    let app_state: AppState = Arc::new(Mutex::new(people));

    let app = Router::new()
        .route("/pessoas", get(search_people))
        .route("/pessoas/{id}", get(find_person))
        .route("/pessoas", post(create_person))
        .route("/contagem-pessoas", get(count_people))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people() -> impl IntoResponse{
    (StatusCode::OK, "Buscar pessoas")
}

async fn find_person(
    State(people): State<AppState>, 
    Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
    let my_people = people.lock().await;
    match my_people.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>
) -> impl IntoResponse {

    let id = Uuid::now_v7();
    let person = Person {
        id,
        apelido: new_person.apelido.0,
        nome: new_person.nome.0,
        nascimento: new_person.nascimento,
        stack: new_person
            .stack
            .map(|stack| stack.into_iter().map(String::from).collect()),
    };

    people.lock().await.insert(id, person.clone());
    (StatusCode::OK, Json(person))
}
    
async fn count_people(State(people): State<AppState>) -> impl IntoResponse {
    let count = people.lock().await.len();
    (StatusCode::OK, Json(count))
}