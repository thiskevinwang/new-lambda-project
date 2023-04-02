use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use tokio_postgres::NoTls;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let queryparams = event.query_string_parameters();
    let env = queryparams.first("env").unwrap_or("dev");

    // dev
    let mut host = "localhost";
    let mut password = "mysecretpassword";

    if env == "rds" {
        host = todo!();
        password = todo!();
    } else if env == "aurora" {
        host = todo!();
        password = todo!();
    }

    // Connect to the database.
    let config_str = format!(
        "host={h} user=postgres password={p}",
        h = host,
        p = password
    );
    let (mut client, connection) = tokio_postgres::connect(&config_str, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    match event.method() {
        &lambda_http::http::Method::GET => {
            println!("GET");
            let transaction = client.transaction().await?;

            // https://github.com/viascom/nanoid-postgres/blob/main/nanoid.sql
            transaction
                .execute("CREATE EXTENSION IF NOT EXISTS pgcrypto", &[])
                .await?;
            transaction.execute("
          CREATE OR REPLACE FUNCTION nanoid(size int DEFAULT 21, alphabet text DEFAULT '_-0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ')
              RETURNS text
              LANGUAGE plpgsql
              volatile
          AS
          $$
          DECLARE
              idBuilder     text := '';
              i             int  := 0;
              bytes         bytea;
              alphabetIndex int;
              mask          int;
              step          int;
          BEGIN
              mask := (2 << cast(floor(log(length(alphabet) - 1) / log(2)) as int)) - 1;
              step := cast(ceil(1.6 * mask * size / length(alphabet)) AS int);
          
              while true
                  loop
                      bytes := gen_random_bytes(size);
                      while i < size
                          loop
                              alphabetIndex := (get_byte(bytes, i) & mask) + 1;
                              if alphabetIndex <= length(alphabet) then
                                  idBuilder := idBuilder || substr(alphabet, alphabetIndex, 1);
                                  if length(idBuilder) = size then
                                      return idBuilder;
                                  end if;
                              end if;
                              i = i + 1;
                          end loop;
          
                      i := 0;
                  end loop;
          END
          $$", &[]).await?;

            transaction
                .execute(
                    "CREATE TABLE IF NOT EXISTS mytable (
              id char(21) DEFAULT nanoid() PRIMARY KEY
            )",
                    &[],
                )
                .await?;

            let rows = transaction
                .query("SELECT * FROM mytable WHERE id IS NOT NULL FOR UPDATE", &[])
                .await?;
            transaction.execute("SELECT pg_sleep(5)", &[]).await?;
            transaction.commit().await?;

            // And then check that we got back the same string we sent over.
            let mut result: &str;
            if rows.len() == 0 {
                result = "no rows";
            } else {
                result = rows[0].get(0);
            }
            println!("result: {:?}", result);

            // Extract some useful information from the request

            // Return something that implements IntoResponse.
            // It will be serialized to the right response event automatically by the runtime
            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(result.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        &lambda_http::http::Method::POST => {
            println!("POST");

            let row = client
                .query_one("INSERT INTO mytable DEFAULT VALUES RETURNING id", &[])
                .await?;

            // id
            let result: &str = row.get(0);
            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(result.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        _ => {
            let message = format!("Unsupported method {}", event.method());
            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/html")
                .body(message.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
    }
}
