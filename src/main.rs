use tracing_subscriber::fmt::format::FmtSpan;

mod font;
mod highlight;
mod image;
mod text;
mod vendor;
mod web;

fn main() {
    let (env, filter) = crate::web::env::parse();
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(false)
        .init();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let addr = ([0, 0, 0, 0], env.port).into();

            let f = axum::Server::bind(&addr)
                .serve(crate::web::app::app(env))
                .with_graceful_shutdown(async {
                    tokio::signal::ctrl_c().await.ok();
                });
            tracing::info!("listening on {addr}");

            f.await
        })
        .unwrap();
}
