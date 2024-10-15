use actix_web::{web, App, HttpServer};
use polytorus::app::global::{initialize_blockchain, initialize_server, start_p2p};
use polytorus::app::mine::mine;
use polytorus::app::miner_transactions::miner_transactions;
use polytorus::app::p2p::P2p;
use polytorus::app::public_key::public_key;
use polytorus::app::route::index;
use polytorus::app::show_block::block;
use polytorus::app::transact::transact;
use polytorus::app::transaction::transactions;
use std::clone::Clone;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let blockchain = initialize_blockchain().await;
    let server = initialize_server().await;

    let p2p_port: String = std::env::var("P2P_PORT").unwrap_or_else(|_| "5001".to_string());
    let http_port: String = std::env::var("HTTP_PORT").unwrap_or_else(|_| "3001".to_string());

    // start_p2p().await;

    let server_clone = server.clone();
    tokio::spawn(async move {
        server_clone.connect_peers().await;
    });

    let server_clone = server.clone();
    tokio::spawn(async move {
        if let Err(e) = server_clone.listen().await {
            eprintln!("Error in P2P server: {}", e);
        }
    });

    println!("Start http server: http://localhost:{}", http_port);
    println!("Start p2p server: ws://localhost:{}", p2p_port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(blockchain.clone()))
            .app_data(web::Data::new(server.clone()))
            .service(index)
            .service(block)
            .service(mine)
            .service(transactions)
            .service(transact)
            .service(public_key)
            .service(miner_transactions)
            .service(web::redirect("/mine", "/block"))
            .service(web::redirect("/trasact", "/transactions"))
            .service(web::redirect("/miner-transactions", "/block"))
    })
    .bind(format!("0.0.0.0:{}", http_port))?
    .run()
    .await
}
