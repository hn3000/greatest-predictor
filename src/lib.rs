use anyhow::anyhow;
use chrono::NaiveDate;
use serde::Deserialize;

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
