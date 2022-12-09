extern crate proc_macro;
use self::proc_macro::TokenStream;
use darling::{FromDeriveInput, FromMeta};
use heck::ToSnakeCase;
use proc_macro2::{Ident, Span};
//use syn::{Meta, NestedMeta, Lit};
use std::collections::HashSet;

use quote::quote;

use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Fields, Lit, Meta, MetaNameValue, NestedMeta,
};

/// 2 -> ( $1,$2 )
fn dollar_values(max: usize) -> Vec<String> {
    let itr = 1..max + 1;
    itr.into_iter()
        .map(|s| format!("${}", s))
        .collect::<Vec<String>>()
}

/// Create method for inserting struts into Sqlite database
///
/// ```rust
/// # #[tokio::main]
/// # async fn main() -> eyre::Result<()>{
/// #[derive(Default, Debug, sqlx::FromRow, sqlxinsert::SqliteInsert)]
/// struct Car {
///     pub car_id: i32,
///     pub car_name: String,
/// }
///
/// let car = Car {
///     car_id: 33,
///     car_name: "Skoda".to_string(),
/// };
///
/// let url = "sqlite::memory:";
/// let pool = sqlx::sqlite::SqlitePoolOptions::new().connect(url).await.unwrap();
///
/// let create_table = "create table cars ( car_id INTEGER PRIMARY KEY, car_name TEXT NOT NULL )";
/// sqlx::query(create_table).execute(&pool).await.expect("Not possible to execute");
///
/// let res = car.insert_raw(&pool, "cars").await.unwrap(); // returning id
/// # Ok(())
/// # }
/// ```
///
#[proc_macro_derive(SqliteInsert)]
pub fn derive_from_struct_sqlite(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };

    // Attributes -> field names
    let field_name = fields.iter().map(|field| &field.ident);
    let field_name2 = fields.iter().map(|field| &field.ident);

    let struct_name = &input.ident;

    let field_length = field_name.len();
    // ( $1, $2)
    let values = dollar_values(field_length).join(",");

    let fields_list = quote! {
        #( #field_name ),*
    };
    let columns = format!("{}", fields_list);

    TokenStream::from(quote! {

        impl #struct_name {
            pub fn insert_query(&self, table: &str) -> String
            {
                let sqlquery = format!("insert into {} ( {} ) values ( {} )", table, #columns, #values); //self.values );
                println!("{}", sqlquery);
                sqlquery
            }

            pub async fn insert_raw(&self, pool: &sqlx::SqlitePool, table: &str) -> eyre::Result<sqlx::sqlite::SqliteQueryResult>
            {
                let sql = self.insert_query(table);
                Ok(sqlx::query(&sql)
                #(
                    .bind(&self.#field_name2)//         let #field_name: #field_type = Default::default();
                )*
                    .execute(pool)// (&mut conn)
                    .await?
                )
            }
        }
    })
}

/// Create method for inserting struts into Postgres database
///
/// ```rust,ignore
/// # #[tokio::main]
/// # async fn main() -> eyre::Result<()>{
///
/// #[derive(Default, Debug, std::cmp::PartialEq, sqlx::FromRow)]
/// struct Car {
///     pub id: i32,
///     pub name: String,
/// }
///
/// #[derive(Default, Debug, sqlx::FromRow, sqlxinsert::PgInsert)]
/// struct CreateCar {
///     pub name: String,
///     pub color: Option<String>,
/// }
/// impl CreateCar {
///     pub fn new<T: Into<String>>(name: T) -> Self {
///         CreateCar {
///             name: name.into(),
///             color: None,
///         }
///     }
/// }
/// let url = "postgres://user:pass@localhost:5432/test_db";
/// let pool = sqlx::postgres::PgPoolOptions::new().connect(&url).await.unwrap();
///
/// let car_skoda = CreateCar::new("Skoda");
/// let res: Car = car_skoda.insert::<Car>(pool, "cars").await?;
/// # Ok(())
/// # }
/// ```
///

//#[derive(Debug, darling::Meta)]
//pub struct Args {
//    #[darling(default)]
//    table: Option<String>,
//}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(sqlxinsert), supports(struct_any))]
struct Attrs {
    /// The struct ident.
    ident: syn::Ident,
    /// The type's generics. You'll need these any time your trait is expected
    /// to work with types that declare generics.
    generics: syn::Generics,
    table: Option<String>,
    update: Option<String>,
    skip: Option<String>,
}

#[proc_macro_derive(PgInsert, attributes(sqlxinsert))]
pub fn derive_from_struct_psql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let attrs: Attrs = Attrs::from_derive_input(&input).unwrap();

    /*let table_name = input
        .attrs
        .iter()
        .find_map(|a| {
            if a.path.is_ident("sqlxinsert") {
                if let Meta::List(l) = a.parse_meta().unwrap() {
                    let x = l.nested.iter().filter_map(|val| {
                        if let NestedMeta::Meta(m) = val {
                            match m {
                                Meta::NameValue(MetaNameValue {
                                    path,
                                    lit: Lit::Str(val),
                                    ..
                                }) if path.is_ident("table") => Some(("table", val.value())),
                                Meta::NameValue(MetaNameValue {
                                    path,
                                    lit: Lit::Str(val),
                                    ..
                                }) if path.is_ident("update") => Some(("update", val.value())),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    }).collect::<Vec<(String, String)>>();
                    l.nested.iter().find_map(|val| {
                        if let NestedMeta::Meta(m) = val {
                            match m {
                                Meta::NameValue(MetaNameValue {
                                    path,
                                    lit: Lit::Str(val),
                                    ..
                                }) if path.is_ident("table") => Some(val.value()),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap_or(input.ident.to_string().to_snake_case());*/

    let skip = match attrs.skip {
        None => {
            HashSet::from(["id".into()])
        }
        Some(s) => {
            //let x = s.split(",");
            s.split(",").map(|s| s.to_string()).collect::<HashSet<String>>()
        }
    };

    let table_name = attrs.table
        .unwrap_or(input.ident.to_string().to_snake_case());

    let fields = match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields"),
    };
    let field_name = fields
        .iter()
        .filter_map(|field| {
            if !skip.contains(field.ident.as_ref().unwrap().to_string().as_str()) {
                Some(&field.ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let field_name_values = fields
        .iter()
        .filter_map(|field| {
            if !skip.contains(field.ident.as_ref().unwrap().to_string().as_str()) {
                Some(&field.ident)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let (update_col_str, is_opt) = match attrs.update {
        None => ( "id".to_string(),  false),
        Some(col) => {
             let mut parts = col.splitn(2, ",").collect::<Vec<_>>();
             if parts.len() == 2 {
                 (parts.first().unwrap().to_string(), true)
             } else {
                 (parts.first().unwrap().to_string(), false)
             }
        }
    };
    let update_col_ident = Ident::new(&update_col_str, Span::call_site());
    let update_col = if is_opt {
        quote! { #update_col_ident.unwrap() }
    } else {
        quote! { #update_col_ident }
    };
    //attrs.update.map_or(quote! { id.unwrap() }, |f| quote! { #f });

    let update_field_name = fields
        .iter()
        .filter_map(|field| {
            if field.ident.as_ref().unwrap().to_string() != update_col_str &&
                !skip.contains(field.ident.as_ref().unwrap().to_string().as_str()) {
                Some(field.ident.clone().unwrap())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let update_field_name_len = update_field_name.len();
    let update_dollars = dollar_values(update_field_name_len);
    let update_vals = update_dollars.join(",");

    let field_length = field_name.len();
    // struct Car { id: i32, name: String }
    // -> ( $1,$2 )
    let vals = dollar_values(field_length);
    let values = vals.join(",");
    /*let vals1 = vals
    .iter()
    .map(|s| Ident::new(s, Span::call_site()))
    .collect::<Vec<_>>();*/

    let update_field_values = update_field_name
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}=${}", s.to_string(), i + 1))
        .collect::<Vec<_>>()
        .join(",");
    //panic!(x);

    // struct Car ...
    // -> Car
    let struct_name = &input.ident;

    // struct { id: i32, name: String }
    // -> ( id, name )
    let columns = format!(
        "{}",
        quote! {
            #( #field_name ),*
        }
    );

    /*let x = quote! {
        #( #field_name = #vals1 ),*
    };

    println!("{}", x);*/
    //let col_val = format!("{}", );

    //println!("{}", col_val);
    //println!("{}", x);

    let update_col_str = update_col.to_string();

    let out = quote! {
        #[async_trait::async_trait]
        impl DBOps for #struct_name {

            const TABLE_NAME: &'static str = #table_name;

            async fn insert<'e, E>(&self, pool: E) -> eyre::Result<Self>
            where
                E: sqlx::Executor<'e, Database = sqlx::Postgres>
            {
                let sql = format!("insert into {} ( {} ) values ( {} ) returning *", Self::TABLE_NAME, #columns, #values); // self.value_list()); //self.values );
                let res: Self = sqlx::query_as::<_,Self>(&sql)
                #(
                    .bind(&self.#field_name_values)
                )*
                    .fetch_one(pool)
                    .await?;

                Ok(res)
            }

            async fn update<'e, E>(&self, pool: E) -> eyre::Result<Self>
            where
                E: sqlx::Executor<'e, Database = sqlx::Postgres>
            {
                let sql = format!("update {} set {} where {} = ${} returning *", Self::TABLE_NAME, #update_field_values, #update_col_str, #update_field_name_len + 1); // self.value_list()); //self.values );
                let res: Self = sqlx::query_as::<_,Self>(&sql)
                #(
                    .bind(&self.#update_field_name)
                )*
                    .bind(&self.#update_col)
                    .fetch_one(pool)
                    .await?;

                Ok(res)
            }
        }
    };

    //panic!("{}", out);

    TokenStream::from(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_test() {
        let itr = 1..4;
        let res = itr
            .into_iter()
            .map(|s| format!("${}", s))
            .collect::<Vec<String>>()
            .join(",");

        assert_eq!(res, "$1,$2,$3");
    }

    #[test]
    fn dollar_value_tes() {
        let res = dollar_values(3).join(",");
        assert_eq!(res, "$1,$2,$3");
    }
}
