use super::global::PostPoolJson;
use crate::{
    app::global::{CHAIN, POOL, SERVER, WALLET},
    wallet::wallets::Wallet,
};
use actix_web::{post, web, HttpResponse, Responder};
use tokio::sync::MutexGuard;

#[post("/transact")]
async fn transact(data: web::Json<PostPoolJson>) -> impl Responder {
    let (recipient, amount) = (data.recipient.clone(), data.amount);

    let transaction = {
        let chain = CHAIN.lock().await;
        let mut pool = POOL.lock().await;
        let mut wallet = WALLET.lock().await;

        match wallet.create_transaction(recipient, amount, &chain, &mut pool) {
            Ok(tx) => tx,
            Err(e) => return HttpResponse::BadRequest().json(format!("Transaction Error: {}", e)),
        }
    };

    {
        let mut server = SERVER.lock().await;
        if let Some(server_instance) = server.as_mut() {
            server_instance
                .broadcast_transaction(transaction.clone())
                .await;
        }
    }

    let valid_transactions = {
        let pool = POOL.lock().await;
        pool.valid_transactions()
            .iter()
            .map(|t| t.to_json())
            .collect::<Vec<_>>()
    };

    HttpResponse::Ok().json(valid_transactions)
}
