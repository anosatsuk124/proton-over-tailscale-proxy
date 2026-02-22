use proton_vpn_api::App;

#[tokio::main]
async fn main() {
    let app = App::new().await.expect("Failed to create application");
    
    if let Err(e) = app.run().await {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }
}
