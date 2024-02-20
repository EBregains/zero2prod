use actix_web::{web ,HttpResponse };
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
  email: String,
  name: String
}

pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> HttpResponse {
  let request_id = Uuid::new_v4();
  tracing::info!("
    [RequestID {}] Adding '{}' <{}> as a new subscriber.",
    request_id,
    form.name,
    form.email
  );
  tracing::info!("
    [RequestID {}] Saving new subscriber details in the databse.",
    request_id,
  );
  
  match sqlx::query!(
    r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
    Uuid::new_v4(),
    form.email,
    form.name,
    Utc::now()
  )
  // We use `get_ref` to get an immutable reference to the `PgPool`
	// wrapped by `web::Data`.
  .execute(pool.get_ref())
  .await
  {
    Ok(_) =>  {
      tracing::info!("[RequestID {}] New subscriber details have been saved", request_id);
      HttpResponse::Ok().finish()
    },
    Err(e) => {
      tracing::error!("[RequestID {}]Failed to execute query: {:?}", request_id, e);
      HttpResponse::InternalServerError().finish()
    }
  }
}