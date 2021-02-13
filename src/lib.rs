
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize,Serialize};

#[derive(Debug, Deserialize)]
pub struct AllocationRecord {
    #[serde(rename = "DATE")]
    pub date: NaiveDate,
    #[serde(rename = "NCBEILQ027S_BCNSDODNS_CMDEBT_FGSDODNS_SLGSDODNS_FBCELLQ027S_DODFFSWCMI")]
    pub allocation: f64,
}

pub async fn fetch_stock_allocation() -> Result<Vec<AllocationRecord>, anyhow::Error> {
    let fut = reqwest::get("https://fred.stlouisfed.org/graph/fredgraph.csv?id=NCBEILQ027S_BCNSDODNS_CMDEBT_FGSDODNS_SLGSDODNS_FBCELLQ027S_DODFFSWCMI&cosd=1951-10-01&coed=2100-01-01&fml=%28%28a%2Bf%29%2F1000%29%2F%28%28%28a%2Bf%29%2F1000%29%2Bb%2Bc%2Bd%2Be%2Bg%29");
    let result = fut.await?;

    let csv = result.text().await?;
    //println!("{}", csv);

    let mut xx = csv::Reader::from_reader(csv.as_bytes());

    let result = xx
        .deserialize()
        .collect::<Result<Vec<AllocationRecord>, _>>()?;

    Ok(result)
}

#[derive(Debug, Deserialize)]
pub struct Snp500Record {
    #[serde(rename = "Date")]
    pub date: NaiveDate,
    #[serde(rename = "Adj Close")]
    pub closing_price: f64,
}

/*
static yahooUrl = 'https://finance.yahoo.com/quote/%5ESP500TR/history?p=%5ESP500TR';
static dailyCsvUrl = 'https://query1.finance.yahoo.com/v7/finance/download/%5ESP500TR?period1=${lastYear}&period2=2500000000&interval=1d&events=history&crumb=';
*/

#[derive(Clone,Copy)]
pub enum Snp500Symbol {
    SNP500,
    SNP500TR
}

impl Snp500Symbol {
    pub fn as_str(self) -> &'static str {
        match self {
            Snp500Symbol::SNP500 => "%5EGSPC",
            Snp500Symbol::SNP500TR => "%5ESP500TR",
        }
    }
}

pub async fn fetch_snp500_data(symbol: Snp500Symbol) -> Result<Vec<Snp500Record>, anyhow::Error> {

    #[cfg(any())] 
    {
        let pageurl = format!("https://finance.yahoo.com/quote/{}/history", symbol.as_str());
        let page = reqwest::get(pageurl)
            .await?
            .text()
            .await?;
    
        let re = regex::Regex::new(r#""CrumbStore":\{"crumb":"([^"]*)"\}"#)?;
    
        let crumb = re
            .captures(&page)
            .ok_or_else(|| anyhow!("crumbled cookie: no crumb"))?
            .get(1)
            .ok_or_else(|| anyhow!("crumbled cookie: no crumb, either"))?
            .as_str()
            .replace(r"\u002F", "/");
    
        dbg!(&crumb);
        let csvurl = format!("https://query1.finance.yahoo.com/v7/finance/download/{0}?period1=${lastYear}&period2=2500000000&interval=1d&events=history&crumb={1}", &symbol.as_str(), &crumb);
    }

    let csvurl = format!("https://query1.finance.yahoo.com/v7/finance/download/{0}?period1=-576032400&period2=4102441200&interval=1d&events=history&includeAdjustedClose=true", symbol.as_str());


    let csvtext = reqwest::get(&csvurl)
        .await?
        .text()
        .await?;

    //dbg!(&csvtext);

    let mut csvrdr = csv::Reader::from_reader(csvtext.as_bytes());

    let result = csvrdr
        .deserialize()
        .collect::<Result<Vec<Snp500Record>, _>>()?;
    
    
    Ok(result)
}

#[derive(Debug, Deserialize)]
pub struct CombinedRecord {
    #[serde(rename = "Date")]
    pub date: NaiveDate,
    #[serde(rename = "StockAllocation")]
    pub stock_allocation: f64,
    #[serde(rename = "Snp500Price")]
    pub snp500_price: f64,
    #[serde(rename = "Snp500Return")]
    pub snp500_return: Option<f64>,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct PredictionRecord {
    #[serde(rename = "Date")]
    pub date: NaiveDate,
    #[serde(rename = "Snp500Price")]
    pub snp500_price: f64,
    #[serde(rename = "Snp500ReturnPredicted")]
    pub snp500_return_predicted: f64,
    #[serde(rename = "Snp500PricePredicted")]
    pub snp500_price_predicted: f64,
}


pub fn allocation_to_return(allocation: f64) -> f64 {
    // taken from the article
    // allocation is 15% - 55%
    // return is -6% - 25%

    let f = (allocation - 0.15) / (0.55 - 0.15);
    let result = 0.25 - f * (0.25 - -0.06);
    result
}

pub fn price_now_to_price_in_10y(price_now: f64, predicted_return: f64) -> f64 {
    price_now * (1.0+predicted_return).powi(10)
}

pub fn combined_records(allocations: &[AllocationRecord], snp500: &[Snp500Record]) -> Vec<CombinedRecord> {
    let allocation_iter = allocations.iter();
    let mut snp_iter = snp500.iter().peekable();
    if let Some(snpfirst) = snp_iter.peek() {
        let allocations_with_snp = allocation_iter.skip_while(|x| x.date < snpfirst.date);
    
        let combined_records = allocations_with_snp.filter_map(|x| {
            let mut snp_iter = snp500.iter().peekable();
            loop {
                if let Some(snp_current) = snp_iter.peek() {
                    if snp_current.date < x.date {
                        let _ = snp_iter.next();
                    } else {

                        if let Some(date_new) = x.date.with_year(x.date.year() + 10) {
                            let prediction = CombinedRecord{ 
                                date: date_new,
                                stock_allocation: x.allocation,
                                snp500_price: snp_current.closing_price,
                                snp500_return: None,
                            };
                            break Some(prediction);
                        } else {
                            break None
                        }

                    }
                } else {
                    break None;
                }
            }
        });
        combined_records.collect()
    } else {
        Vec::new()
    }

}

pub fn prediction_records(allocations: &[AllocationRecord], snp500: &[Snp500Record]) -> Vec<PredictionRecord> {
    let allocation_iter = allocations.iter();
    let mut snp_iter = snp500.iter().peekable();
    if let Some(snpfirst) = snp_iter.peek() {
        let allocations_with_snp = allocation_iter.skip_while(|x| x.date < snpfirst.date);

        let mut snp_iter = snp500.iter().peekable();    
        let prediction_records = allocations_with_snp.filter_map(|x| {
            loop {
                if let Some(snp_current) = snp_iter.peek() {
                    if snp_current.date < x.date {
                        let _ = snp_iter.next();
                    } else {
                        let snp500_return_predicted = allocation_to_return(x.allocation);
                        let snp500_price_predicted = price_now_to_price_in_10y(snp_current.closing_price, snp500_return_predicted);

                        if let Some(date_new) = x.date.with_year(x.date.year() + 10) {
                            let prediction = PredictionRecord{ 
                                date: date_new,
                                snp500_return_predicted,
                                snp500_price_predicted,
                                snp500_price: snp_current.closing_price
                            };
                            break Some(prediction);
                        } else {
                            break None
                        }

                    }
                } else {
                    break None;
                }
            }
        });
        prediction_records.collect()
    } else {
        Vec::new()
    }
}
