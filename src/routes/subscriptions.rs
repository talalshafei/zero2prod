use chrono::Utc;
use uuid::Uuid;
use sqlx::PgPool;
use actix_web::{web, HttpResponse};
use crate::email_client::EmailClient;
use crate::domain::{
    NewSubscriber, 
    SubscriberEmail, 
    SubscriberName,
};



#[derive(serde::Deserialize)]
pub struct FormData{
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>
) -> HttpResponse {

    // `web::Form` is a wrapper around `FormData`
    // `form.0` gives us access to the underlying `FormData`
    
    // when implementing try_from, 
    // the opposite try_into get implemented for free!
    // NewSubscriber::try_from(form.0) == form.0.try_into()

    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    
    if insert_subscriber(&pool, &new_subscriber).await.is_err(){
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {

    let confirmation_link = 
        "https://there-is-no-such-domain.com/subscriptions/confirm";

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link,
    );

    let html_body = format!(
        "Welcome to our newsletter! <br/>\
        Click <a href=\"{}\">here</a> to confirm your subscription. ",
        confirmation_link,
    );

    // Send a (useless) email to the new subscriber
    email_client.send_email(
        new_subscriber.email, 
        "Welcome!", 
        &plain_body, 
        &html_body, 
    ).await
    
}

#[tracing::instrument(
    name = "Saving a new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

