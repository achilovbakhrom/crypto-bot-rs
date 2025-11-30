use crypto_bot::{ Config, Result };
use axum::{ Router, routing::{ get, post } };
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt };

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber
        ::registry()
        .with(
            tracing_subscriber::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| "crypto_bot=debug,tower_http=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().map_err(|e| crypto_bot::AppError::Config(e.to_string()))?;

    tracing::info!("Starting crypto-bot with network mode: {:?}", config.network_mode);

    // Initialize database connection
    let db = sea_orm::Database
        ::connect(&config.database_url).await
        .map_err(|e| crypto_bot::AppError::Database(e))?;

    tracing::info!("Database connected successfully");

    // Run migrations
    migration::Migrator::up(&db, None).await.map_err(|e| crypto_bot::AppError::Database(e))?;

    tracing::info!("Migrations completed successfully");

    // Initialize encryptor
    let encryptor = Arc::new(crypto_bot::crypto::Encryptor::new(&config.encryption_key)?);

    // Initialize RPC manager
    let rpc_manager = Arc::new(crypto_bot::rpc::RpcManager::new(&config)?);
    tracing::info!("RPC manager initialized");

    // Initialize repository
    let repository = Arc::new(crypto_bot::db::WalletRepository::new(db));

    // Initialize services
    let wallet_service = Arc::new(
        crypto_bot::services::WalletService::new(
            repository.clone(),
            rpc_manager.clone(),
            encryptor.clone()
        )
    );

    let balance_service = Arc::new(
        crypto_bot::services::BalanceService::new(
            repository.clone(),
            rpc_manager.clone(),
            encryptor.clone()
        )
    );

    let transfer_service = Arc::new(
        crypto_bot::services::TransferService::new(
            repository.clone(),
            rpc_manager.clone(),
            encryptor.clone()
        )
    );

    // Create app state
    let app_state = crypto_bot::api::AppState::new(
        wallet_service,
        balance_service,
        transfer_service
    );

    let config = Arc::new(config);

    // Build application router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/wallets/generate", post(crypto_bot::api::wallet::generate_wallet))
        .route("/api/wallets/restore", post(crypto_bot::api::wallet::restore_wallet))
        .route("/api/wallets/:id", get(crypto_bot::api::wallet::get_wallet))
        .route("/api/wallets/:id/balance", get(crypto_bot::api::balance::get_balance))
        .route("/api/wallets/:id/transfer", post(crypto_bot::api::transfer::send_transaction))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener
        ::bind(&addr).await
        .map_err(|e| crypto_bot::AppError::Internal(e.to_string()))?;

    axum::serve(listener, app).await.map_err(|e| crypto_bot::AppError::Internal(e.to_string()))?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
