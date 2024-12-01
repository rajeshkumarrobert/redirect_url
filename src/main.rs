use std::{collections::HashMap, sync::{Arc, Mutex}};

use anyhow::Result;
use axum::{ extract::{Path, State}, response::{IntoResponse, Redirect}, routing::{get, put}, Json, Router};
use serde::Deserialize;
use sqlx::{prelude::FromRow, PgPool};

#[tokio::main]

async fn main() -> Result<()>{
    let url_hashmap= Arc::new(Mutex::new(HashMap::<String,String>::new()));

    let db = dotenv::var("DATABASE_URL")?;
    let db_pool = sqlx::postgres::PgPoolOptions::new()
    .max_connections(25)
    .connect(&db)
    .await?;
    
    let app_state = AppState{
        db:db_pool.clone(),
        memory_store: url_hashmap,
    };
    
    let db_value:Vec<UrlTableValue> = sqlx::query_as(r#"SELECT name,value, is_active FROM urls"#)
    .fetch_all(&db_pool)
    .await?;


    let mut store = app_state.memory_store.lock().unwrap();

    for val in db_value.iter() {
        if val.is_active {
            store.insert(val.name.clone(), val.value.clone());
        }
    }
    drop(store);


    let v1_routes = Router::new()
    .route("/add_url", put(add_url))
    .route("/get_url/:target_url", get(get_url))
    .route("/list", get(get_url_list))
    .with_state(app_state);

    // let app = Router::new()
    // .nest("/api/link", v1_routes);

    println!("test");

    let port = std::env::var("PORT").unwrap_or("3000".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    axum::serve(listener, v1_routes).await?;

    Ok(())

}
#[derive(Debug,Deserialize)]
struct ActualUrl{
    name:String,
    url: String,
    secret: String
}
#[derive(Clone)]
struct AppState{
   db: PgPool,
   memory_store: Arc<Mutex<HashMap<String,String>>>,
}
#[derive(FromRow,Debug)]
struct UrlTableValue{
    name:String,
    value:String,
    is_active:bool,
}

 async fn add_url(
    State(app_state):State<AppState>,
    Json(params):Json<ActualUrl>,
) -> impl IntoResponse{
        let secret_from_env = dotenv::var("secret");
        if secret_from_env.is_err() {
            return "Err"
        }
        let secret_from_env = secret_from_env.unwrap();
        if secret_from_env != params.secret {
            return "Err";
        }
        let query = sqlx::query(r#"INSERT into urls (name,value) VALUES ($1,$2) 
        ON CONFLICT (name) DO UPDATE SET value=$2"#)
        .bind(&params.name)
        .bind(&params.url)
        .execute(&app_state.db)
        .await;
    if query.is_err() {
        return "Error"
    //     let resp = Builder::new()
    //     .status(StatusCode::INTERNAL_SERVER_ERROR)
    //     .body("Error updating db")
    //     .unwrap();

    //     return resp;
    }

   let mut map = app_state.memory_store.lock().unwrap();
    // println!("{:#?}",params);
 map.insert(params.name,params.url);
   println!("{:?}",app_state.memory_store);
   "oK"
    // Html("Hello <strong>World!!!</strong>") 
}

async  fn get_url(
    Path(target_url):Path<String>,
    State(app_state):State<AppState>) -> impl IntoResponse{
        let set_map: std::sync::MutexGuard<'_, HashMap<String, String>>= app_state.memory_store.lock().unwrap();
        println!("{:?}",set_map);
      let name = set_map.get(&target_url).unwrap();
      println!("name {name}");
      Redirect::to(name)
    //   Redirect::to("https://hmagchurch.in")
    //   return set_map.get(&target_url).unwrap().to_string();
}

async fn get_url_list(
    State(app_state):State<AppState>,
    ) -> impl IntoResponse {
        let db_pool = app_state.db;
        let db_value:Vec<UrlTableValue> = sqlx::query_as(r#"SELECT name,value, is_active FROM urls"#)
    .fetch_all(&db_pool)
    .await.unwrap();
        println!("{:?}",db_value);
       //db_value
    }