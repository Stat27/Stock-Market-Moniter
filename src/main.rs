use chrono::NaiveDateTime;
use yahoo_finance_api as yahoo;
use time::{OffsetDateTime, Duration};
use std::io::{self, Write};
use std::error::Error;
use plotters::prelude::*;
use plotters::style::full_palette::GREY;
use ta::Next;
use ta::indicators::RelativeStrengthIndex;
use ta::indicators::MovingAverageConvergenceDivergence;

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
    plot_rsi(ticker, &quotes, &dates)?;
    plot_macd(ticker, &quotes, &dates)?;

    Ok(())
}

// plot RSI values of the stock
fn plot_rsi(ticker: &str, quotes: &[yahoo::Quote], dates: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Define the picture path
    let path = format!("{}-stock-RSI-chart.png", ticker);
    // Create the backend picture
    let root_area = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    // Fill with all white
    root_area.fill(&WHITE)?;

    let closing_prices: Vec<_> = quotes.iter().map(|quote| quote.close).collect();
    let mut rsi = RelativeStrengthIndex::new(14).unwrap();

    // calculate the rsi_values
    let rsis: Vec<f64> = closing_prices.iter().map(|&price| {
        rsi.next(price)
    }).collect();

    // create the initial chart and set up x/y axis
    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("{} Stock RSI", ticker), ("sans-serif", 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..dates.len(), 0.0..100.0)?;

    // Set x-axis labels to be the dates
    chart.configure_mesh()
        .x_labels(dates.len()/4)
        .x_label_formatter(&|idx| {
            let labels_count = dates.len();
            let labels_to_display = 6;
            let step = labels_count / labels_to_display;
            if idx%step == 0 {
                if let Some(date) = dates.get(*idx) {
                    println!("{}",date);

                    return date.to_string();
                }
            }

            String::new()
        }).y_labels(dates.len()/4)
        .draw()?;

    // draw RSI line
    chart.draw_series(LineSeries::new(
        rsis.iter().enumerate().map(|(i, &rsi)| (i, rsi)),
        &RED,
    ))?.label(ticker.to_owned() + " RSI")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));;

    // overbought line
    chart.draw_series(std::iter::once(PathElement::new(
        [(0, 70.0), (dates.len(), 70.0)],
        GREEN.stroke_width(2),
    )))?.label("Overbought")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &GREEN));;

    // oversold line
    chart.draw_series(std::iter::once(PathElement::new(
        [(0, 30.0), (dates.len(), 30.0)],
        BLUE.stroke_width(2),
    )))?.label("Oversold")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));;

    // Add legend on the top-left corner
    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    Ok(())
}

fn plot_macd(ticker: &str, quotes: &[yahoo::Quote], dates: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Define the picture path
    let path = format!("{}-stock-MACD-chart.png", ticker);
    // Create the backend picture
    let root_area = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    // Fill with all white
    root_area.fill(&WHITE)?;

    let closing_prices: Vec<_> = quotes.iter().map(|quote| quote.close).collect();
    let mut macd = MovingAverageConvergenceDivergence::new(12, 26, 9).unwrap();
    let macds: Vec<(f64, f64, f64)> = closing_prices.iter().map(|&price| {
        let macd_next = macd.next(price);
        (macd_next.macd, macd_next.signal, macd_next.histogram)
    }).collect();

    let macd_min = macds.iter().map(|x| x.2).fold(f64::INFINITY, f64::min);
    let macd_max = macds.iter().map(|x| x.2).fold(f64::NEG_INFINITY, f64::max);
    let space = macd_max - macd_min;

    // create the initial chart and set up x/y axis
    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("{} Stock MACD", ticker), ("sans-serif", 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..dates.len(), macd_min - space..macd_max + space)?;

    // Set x-axis labels to be the dates
    chart.configure_mesh()
        .x_labels(dates.len()/4)
        .x_label_formatter(&|idx| {
            let labels_count = dates.len();
            let labels_to_display = 6;
            let step = labels_count / labels_to_display;
            if idx%step == 0 {
                if let Some(date) = dates.get(*idx) {
                    println!("{}",date);

                    return date.to_string();
                }
            }

            String::new()
        }).y_labels(dates.len()/4)
        .draw()?;

    chart.draw_series(LineSeries::new(
        macds.iter().enumerate().map(|(i, &(macd, ..))| (i, macd)),
        &RED,
    ))?.label("MACD")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    chart.draw_series(LineSeries::new(
        macds.iter().enumerate().map(|(i, &(_, signal, ..))| (i, signal)),
        &BLUE,
    ))?.label("MACD Signal")
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

    chart.configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .draw()?;

    Ok(())

}

fn plot_quotes(ticker: &str, quotes: &[yahoo::Quote], dates: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Define the picture path
    let path = format!("{}-stock-chart.png", ticker);
    // Create the backend picture
    let root_area = BitMapBackend::new(&path, (800, 600)).into_drawing_area();
    // Fill with all white
    root_area.fill(&WHITE)?;

    let closing_prices: Vec<_> = quotes.iter().map(|quote| quote.close).collect();
    let min_close_price = quotes.iter().map(|x| x.close).fold(f64::INFINITY, f64::min);
    let max_close_price = quotes.iter().map(|x| x.close).fold(f64::NEG_INFINITY, f64::max);

    // Determine volatile days
    let volatile_days: Vec<(usize, &yahoo::Quote)> = quotes.iter().enumerate().filter(|(_i, quote)| {
        let price_range = quote.high - quote.low;
        let price_change_percent = (price_range / quote.close) * 100.0;
        price_change_percent > 2.0
    }).collect();

    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("{} Stock Price", ticker), ("sans-serif", 40))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0..closing_prices.len()+closing_prices.len()/20, min_close_price-10.0..max_close_price+10.0)?;

    // Draw the closing prices as a line series
    chart.draw_series(LineSeries::new(
        dates.iter().zip(closing_prices.iter()).enumerate().map(|(i, (_date, price))| (i, *price)),
        &RED,
    ))?
    .label(ticker)
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    let bar_width = 1;

    // draw special error lines for volatile_days
    for (i, quote) in quotes.iter().enumerate() {
        let price_range = quote.high - quote.low;
        let price_change_percent = (price_range / quote.close) * 100.0;
        if price_change_percent > 2.0 {
            // Error bar for high-low
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(i, quote.low), (i, quote.high)],
                BLUE.stroke_width(1),
            )))?;

            chart.draw_series(volatile_days.iter().map(|(i, quote)| {
                // Directly use a color that implements `Color` trait for the circle fill
                Circle::new((*i, quote.close), 3, BLUE.mix(0.5))
            }))?;

            let start_pos = if i > bar_width { i - bar_width } else { 0 };

            chart.draw_series(std::iter::once(PathElement::new(
                vec![(start_pos, quote.high), (i + bar_width, quote.high)],
                BLUE.stroke_width(1),
            )))?;
            
            chart.draw_series(std::iter::once(PathElement::new(
                vec![(start_pos, quote.low), (i + bar_width, quote.low)],
                BLUE.stroke_width(1),
            )))?;
        
        }
    }

    // Set x-axis labels to be the dates
    chart.configure_mesh()
        .x_labels(dates.len()/4)
        .y_labels(dates.len()/4)
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