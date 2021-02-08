use greatest_predictor::{Snp500Symbol, fetch_snp500_data, fetch_stock_allocation};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //let allocations = fetch_stock_allocation().await?;
    let snp500data = fetch_snp500_data(Snp500Symbol::SNP500TR).await?;
    //dbg!(allocations, snp500data);
    dbg!(snp500data);
    Ok(())
}
