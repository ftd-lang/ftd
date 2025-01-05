#[tokio::main]
async fn main() {
    let command = fastn::commands::parse();
    let mut section_provider = fastn::SectionProvider::default();
    let mut package = section_provider
        .read(fastn_package::Package::reader())
        .await;
    let router = section_provider.read(fastn_router::Router::reader()).await;
    // read config here and pass to everyone?
    // do common build stuff here
    match command {
        fastn::commands::Cli::Serve(input) => input.run(package, router).await,
        fastn::commands::Cli::Render(input) => input.run(&mut package, router).await,
        fastn::commands::Cli::Build(input) => input.run(package).await,
        fastn::commands::Cli::Static { .. } => {}
        fastn::commands::Cli::Test { .. } => {}
        fastn::commands::Cli::Fmt(_) => {}
        fastn::commands::Cli::Upload { .. } => {}
        fastn::commands::Cli::Clone(_) => {}
    };
}
