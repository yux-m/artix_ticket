mod api;
mod model;
mod repo;

use api::ticket::{
    get_ticket,
    send_ticket,
    start_ticket,
    complete_ticket,
    pause_ticket,
};

use actix_web::{HttpServer, App, web::Data, middleware::Logger};
use repo::ddb::DDBRepository;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config = aws_config::load_from_env().await;
    HttpServer::new(move || {
        let ddb_repo: DDBRepository = DDBRepository::init(
            String::from("ticket"),
            config.clone()
        );
        let ddb_data = Data::new(
            ddb_repo
        );
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(ddb_data)
            .service(get_ticket)
            .service(send_ticket)
            .service(start_ticket)
            .service(complete_ticket)
            .service(pause_ticket)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
