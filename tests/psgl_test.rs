// extern crate we're testing, same as any other code will do.
//extern crate gmacro;

// use sqlx::PgQuery;

use async_trait::async_trait;

// #[derive(Default, Debug, sqlx::FromRow)]
#[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
struct Car {
    pub id: i32,
    pub name: String,
}

//for<'c> sqlx::FromRow<'c, sqlx::postgres::PgRow> +
#[async_trait]
trait DBOps: Send +  Unpin + Sized {
    const TABLE_NAME: &'static str;

    async fn insert<'e, E>(&self, pool: E) -> eyre::Result<Self>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;

    async fn update<'e, E>(&self, pool: E) -> eyre::Result<Self>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>;
}

#[derive(Default, Debug, sqlx::FromRow, sqlxinsert::PgInsert)]
#[sqlxinsert(table="car", update="name")]
struct CreateCar {
    pub id: Option<i32>,
    pub name: String,
    pub color: Option<String>,
}
impl CreateCar {
    pub fn new<T: Into<String>>(name: T) -> Self {
        format!("{}", 1);
        CreateCar {
            id: None,
            name: name.into(),
            color: None,
        }
    }
}

#[tokio::test]
async fn test_macro_insert() {
    let car_skoda = CreateCar::new("Skoda");
    let car_tesla = CreateCar::new("Tesla");

    let url = "postgres://user:pass@localhost:5444/test_db";

    let pool = sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(url)
        .await
        .expect("Not possible to create pool");

    // Reset database
    let drop_table = "DROP TABLE IF EXISTS cars";
    sqlx::query(drop_table).execute(&pool).await.unwrap();

    let create_table = "create table cars (
        id SERIAL PRIMARY KEY,
        name TEXT NOT NULL,
        color TEXT
    )";

    sqlx::query(create_table).execute(&pool).await.unwrap();

    // Fill data
    let car_skoda_res = car_skoda
        .insert(&pool)
        .await
        .expect("Not possible to insert into dabase");
    assert_eq!(car_skoda_res.name, car_skoda.name);

    let car_tesla_res = car_tesla
        .insert(&pool)
        .await
        .expect("Not possible to insert into dabase");
    assert_eq!(car_tesla_res.name, car_tesla.name);

    let cars = sqlx::query_as::<_, Car>("SELECT * FROM cars")
        .fetch_all(&pool)
        .await
        .expect("Not possible to fetch");

    assert_eq!(cars.len(), 2);
    assert_eq!(cars[0].name, "Skoda");
    assert_eq!(cars[0].id, 1);
    assert_eq!(cars[1].name, "Tesla");
    assert_eq!(cars[1].id, 2);
}
