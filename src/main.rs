use std::time::Instant;
use std::str::FromStr;

use fuels::accounts::{provider::Provider, wallet::WalletUnlocked};
use fuels::client::{PageDirection, PaginationRequest};
use fuels::types::ContractId;
use spark_market_sdk::SparkMarketContract;
use thiserror::Error;
use dotenv::dotenv;
use log::info;


#[derive(Debug, Error)]
pub enum BenchmarkError {
    #[error("Failed to connect to provider: {0}")]
    ProviderConnectionError(String),

    #[error("Failed to fetch latest gas price: {0}")]
    GasPriceFetchError(String),

    #[error("Failed to fetch latest block height: {0}")]
    BlockHeightFetchError(String),

    #[error("Failed to fetch latest transaction: {0}")]
    TransactionFetchError(String),

    #[error("Failed to create wallet: {0}")]
    WalletCreationError(String),

    #[error("Failed to interact with contract: {0}")]
    ContractInteractionError(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init(); 

    
    let provider_urls = vec![
        "mainnet.fuel.network",
        "fuel.liquify.com/v1/graphql",
    ];

    
    let mnemonic = std::env::var("MNEMONIC").expect("MNEMONIC not found in .env");

    
    let contract_id = "contract_id";

    info!("Starting benchmarks...");

    for url in &provider_urls {
        info!("\nBenchmarking provider: {}", url);

        
        match benchmark_node(url).await {
            Ok(duration) => info!("Node Request: Response Time: {:.2?}", duration),
            Err(e) => info!("Node Request: Error: {}", e),
        }

        
        /*
        match benchmark_contract(url, &mnemonic, contract_id).await {
            Ok(duration) => info!("Contract Request: Response Time: {:.2?}", duration),
            Err(e) => info!("Contract Request: Error: {}", e),
        }*/
    }

    Ok(())
}


async fn benchmark_node(url: &str) -> Result<std::time::Duration, BenchmarkError> {
    let start_time = Instant::now();

    
    info!("Connecting to node {:?}", url);
    let provider = Provider::connect(url)
        .await
        .map_err(|e| BenchmarkError::ProviderConnectionError(e.to_string()))?;
    info!("Connected");

    
    info!("Trying to get last block height...");
    let lbh = provider
        .latest_block_height()
        .await
        .map_err(|e| BenchmarkError::BlockHeightFetchError(e.to_string()))?;
    info!("Block height: {:?}", lbh);

    
    info!("Trying to get latest gas price...");
    let gas_price = provider
        .latest_gas_price()
        .await
        .map_err(|e| BenchmarkError::GasPriceFetchError(e.to_string()))?;
    info!("Latest gas price: {:?}", gas_price);

    
    info!("Trying to fetch the latest transaction...");
    let p_r = PaginationRequest {
        cursor: None,
        results: 10,
        direction: PageDirection::Backward
    };
    let transactions = provider.get_transactions(p_r)
        .await
        .map_err(|e| BenchmarkError::TransactionFetchError(e.to_string()))?;
    
    if let Some(latest_tx) = transactions.results.first() {
        info!("Latest transaction: {:?}", latest_tx.status);
    } else {
        info!("No transactions found in the latest block.");
    }

    Ok(start_time.elapsed())
}

async fn benchmark_contract(
    url: &str,
    mnemonic: &str,
    contract_id: &str,
) -> Result<std::time::Duration, BenchmarkError> {
    let start_time = Instant::now();

    
    let provider = Provider::connect(url)
        .await
        .map_err(|e| BenchmarkError::ProviderConnectionError(e.to_string()))?;

    
    let path = format!("m/44'/1179993420'/{}'/0/0", 0);
    let wallet = WalletUnlocked::new_from_mnemonic_phrase_with_path(
        mnemonic,
        Some(provider.clone()),
        &path,
    )
    .map_err(|e| BenchmarkError::WalletCreationError(e.to_string()))?;

    
    let market = SparkMarketContract::new(ContractId::from_str(contract_id).unwrap(), wallet)
        .await;
    
    market
        .matcher_fee()
        .await
        .map_err(|e| BenchmarkError::ContractInteractionError(e.to_string()))?;

    Ok(start_time.elapsed())
}
