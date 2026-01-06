use crypto_bot::{ Config, Result };
use axum::{ Router, routing::{ get, post } };
use std::sync::Arc;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{ layer::SubscriberExt, util::SubscriberInitExt };

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber
        ::registry()
        .with(
            tracing_subscriber::EnvFilter
                ::try_from_default_env()
                .unwrap_or_else(|_| "crypto_bot=debug,tower_http=debug".into())
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env().map_err(|e| crypto_bot::AppError::Config(e.to_string()))?;
    tracing::info!("Starting crypto-bot with network mode: {:?}", config.network_mode);

    let db = sea_orm::Database
        ::connect(&config.database_url).await
        .map_err(|e| crypto_bot::AppError::Database(e))?;
    tracing::info!("Database connected successfully");

    let encryptor = Arc::new(crypto_bot::crypto::Encryptor::new(&config.encryption_key)?);
    let rpc_manager = Arc::new(crypto_bot::rpc::RpcManager::new(&config)?);
    tracing::info!("RPC manager initialized");

    let repository = Arc::new(crypto_bot::db::WalletRepository::new(db.clone()));
    let transaction_repo = Arc::new(crypto_bot::db::TransactionRepository::new(db.clone()));

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
            transaction_repo.clone(),
            rpc_manager.clone(),
            encryptor.clone()
        )
    );

    let transaction_service = Arc::new(
        crypto_bot::services::TransactionService::new(transaction_repo.clone(), repository.clone())
    );

    let price_service = Arc::new(crypto_bot::services::PriceService::new());

    let portfolio_service = Arc::new(
        crypto_bot::services::PortfolioService::new(
            repository.clone(),
            rpc_manager.clone(),
            price_service.clone()
        )
    );

    let address_book_service = Arc::new(
        crypto_bot::services::AddressBookService::new(Arc::new(db.clone()))
    );

    let gas_estimation_service = Arc::new(
        crypto_bot::services::GasEstimationService::new(
            repository.clone(),
            rpc_manager.clone(),
            price_service.clone()
        )
    );

    let scheduling_service = Arc::new(
        crypto_bot::services::scheduling_service::SchedulingService::new(db.clone())
    );

    let price_alert_service = Arc::new(
        crypto_bot::services::price_alert_service::PriceAlertService::new(db.clone())
    );

    let security_service = Arc::new(
        crypto_bot::services::security_service::SecurityService::new(db.clone())
    );

    let swap_service = Arc::new(
        crypto_bot::services::swap_service::SwapService::new(db.clone(), wallet_service.clone())
    );

    let config_clone = config.clone();

    // Background task: scheduled transaction executor
    let scheduler_db = db.clone();
    let scheduler_transfer_service = transfer_service.clone();
    tokio::spawn(async move {
        let scheduler = crypto_bot::scheduler::Scheduler::new(
            scheduler_db,
            scheduler_transfer_service
        );
        scheduler.start().await;
    });

    // Background task: Telegram bot
    let bot_wallet_service = wallet_service.clone();
    let bot_balance_service = balance_service.clone();
    let bot_transfer_service = transfer_service.clone();
    let bot_transaction_service = transaction_service.clone();
    let bot_portfolio_service = portfolio_service.clone();
    let bot_price_service = price_service.clone();
    let bot_address_book_service = address_book_service.clone();
    let bot_gas_estimation_service = gas_estimation_service.clone();
    let bot_scheduling_service = scheduling_service.clone();
    let bot_price_alert_service = price_alert_service.clone();
    let bot_security_service = security_service.clone();
    let bot_swap_service = swap_service.clone();
    let bot_encryptor = encryptor.clone();
    let bot_config = Arc::new(config.clone());
    let bot_token = config.telegram_bot_token.clone();

    tokio::spawn(async move {
        crypto_bot::bot::run_bot(
            bot_token,
            bot_wallet_service,
            bot_balance_service,
            bot_transfer_service,
            bot_transaction_service,
            bot_portfolio_service,
            bot_price_service,
            bot_address_book_service,
            bot_gas_estimation_service,
            bot_scheduling_service,
            bot_price_alert_service,
            bot_security_service,
            bot_swap_service,
            bot_encryptor,
            bot_config,
        ).await;
    });

    // Background task: price alert checker
    let alert_db = db.clone();
    let alert_price_service = price_service.clone();
    let alert_bot_token = config.telegram_bot_token.clone();

    tokio::spawn(async move {
        let bot = teloxide::Bot::new(alert_bot_token);
        let alert_checker = crypto_bot::alert_checker::AlertChecker::new(
            alert_db,
            alert_price_service,
            bot
        );
        alert_checker.start().await;
    });

    let app_state = crypto_bot::api::AppState::new(
        wallet_service,
        balance_service,
        transfer_service,
        transaction_service
    );

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/wallets/generate", post(crypto_bot::api::wallet::generate_wallet))
        .route("/api/wallets/restore", post(crypto_bot::api::wallet::restore_wallet))
        .route("/api/wallets/{id}", get(crypto_bot::api::wallet::get_wallet))
        .route("/api/wallets/{id}/balance", get(crypto_bot::api::balance::get_balance))
        .route("/api/wallets/{id}/transfer", post(crypto_bot::api::transfer::send_transaction))
        .route("/api/wallets/{id}/transactions", get(crypto_bot::api::transaction::get_wallet_transactions))
        .route("/api/transactions", get(crypto_bot::api::transaction::get_user_transactions))
        .route("/api/transactions/{tx_hash}", get(crypto_bot::api::transaction::get_transaction))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config_clone.server_host, config_clone.server_port);
    tracing::info!("REST API listening on {}", addr);
    tracing::info!("Telegram bot running...");

    let listener = tokio::net::TcpListener
        ::bind(&addr).await
        .map_err(|e| crypto_bot::AppError::Internal(e.to_string()))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| crypto_bot::AppError::Internal(e.to_string()))?;

    tracing::info!("Shutting down...");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn health_check() -> &'static str {
    "OK"
}
