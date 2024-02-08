use chrono::NaiveDateTime;
use yahoo_finance_api as yahoo;
use time::{OffsetDateTime, Duration};
use std::io::{self, Write};
use std::error::Error;
use plotters::prelude::*;

async fn fetch_stock_data(ticker: &str) -> Result<(), Box<dyn Error>> {
    let provider = yahoo::YahooConnector::new();

    // Use OffsetDateTime from the 'time' crate
    // set now and 6 month ago to get the time period
    let now = OffsetDateTime::now_utc();
    let six_months = Duration::days(30 * 6); // Approximately 6 months
    let start = now - six_months;
    let end = now;

    // println!("{}", start);
    // Attempt to fetch the quote history
    let resp = match provider.get_quote_history(ticker, start, end).await {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("Invalid Company Code, {} ",e);
            return Ok(());
        }
    };
    let quotes = resp.quotes()
        .expect("Failed to get quotes from response");

    // let dates = Vec::new();
    // println!("{}'s daily closing prices for the last six months:", ticker);
    // for quote in quotes.iter() {
    //     // Convert timestamp to human-readable date and save it into the vector
    //     let date = match NaiveDateTime::from_timestamp_opt(quote.timestamp as i64, 0) {
    //         Some(dt) => dt,
    //         None => {
    //             println!("Invalid timestamp: {}", quote.timestamp);
    //             continue; // Skip this quote if timestamp is invalid
    //         }
    //     };
    //     let formatted_date = date.format("%Y-%m-%d").to_string();
    //     dates.push(formatted_date);
    //     println!("Date: {}, Close: {}", date, quote.close);
    // }
    let dates: Vec<String> = quotes.iter().filter_map(|quote| {
        // Convert timestamp to human-readable date
        let date = match NaiveDateTime::from_timestamp_opt(quote.timestamp as i64, 0) {
            Some(dt) => dt,
            None => {
                println!("Invalid timestamp: {}", quote.timestamp);
                return None; // Skip this quote if timestamp is invalid
            }
        };
        Some(date.format("%Y-%m-%d").to_string()) // Return formatted date
    }).collect();
    plot_quotes(ticker, &quotes, &dates)?;

    Ok(())
}

fn plot_quotes(ticker: &str, quotes: &[yahoo::Quote], dates: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Create a drawing area to plot the chart
    let path = format!("{}-stock-chart.png", ticker);
    let root_area = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    root_area.fill(&WHITE)?;

    // Extract closing prices and convert them to f64
    let closing_prices: Vec<_> = quotes.iter().map(|quote| quote.close).collect();
    let min_close_price = quotes.iter().map(|x| x.close).fold(f64::INFINITY, f64::min);
    let max_close_price = quotes.iter().map(|x| x.close).fold(f64::NEG_INFINITY, f64::max);


    // Create a chart builder
    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("{} Stock Price", ticker), ("sans-serif", 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..closing_prices.len()+closing_prices.len()/20, min_close_price-10.0..max_close_price+10.0)?; // Adjust the y-axis range as needed

    // Draw the closing prices as a line series
    chart.draw_series(LineSeries::new(
        dates.iter().zip(closing_prices.iter()).enumerate().map(|(i, (_date, price))| (i, *price)),
        &RED,
    ))?
    .label(ticker)
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED)); // Add a legend

    // Set x-axis labels to be the dates
    chart.configure_mesh()
    .x_labels(dates.len())
    .x_label_formatter(&|idx| {
        let labels_count = dates.len();
        println!("{}c",labels_count);
        let labels_to_display = 6;
        let step = labels_count / labels_to_display;
        println!("{}",step);
        if idx%step == 0 {
            if let Some(date) = dates.get(*idx) {
                println!("{}",date);

                return date.to_string();
            }
        }
        
        String::new()
    }).draw()?;
     // Add legend on the top-left corner
    chart.configure_series_labels()
    .position(SeriesLabelPosition::UpperLeft)
    .draw()?;

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        print!("Enter the stock ticker (or 'q' to exit): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let ticker = input.trim();
        if ticker == "q" {
            break;
        }

        fetch_stock_data(ticker).await?;
    }

    Ok(())
}