use greatest_predictor::{Snp500Symbol, fetch_snp500_data, fetch_stock_allocation, prediction_records};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let allocations = fetch_stock_allocation().await?;
    let snp500data = fetch_snp500_data(Snp500Symbol::SNP500TR).await?;

    let prediction = prediction_records(&allocations, &snp500data);
    dbg!(&prediction);

    let mut writer = csv::Writer::from_writer(std::io::stdout());

    prediction.iter().try_for_each(|x| writer.serialize(x))?;
    writer.flush()?;

    //dbg!(allocations, snp500data);
    //dbg!(snp500data);
    Ok(())
}
