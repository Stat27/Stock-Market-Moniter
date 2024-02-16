# Stock-Market-Moniter

## Crate used in this program:
#### 1. chrono
We use chrono::NaiveDateTime method to handle the dates and times from the timestamp
#### 2. yahoo_finance_api
We use yahoo_finance_api to fetch the historical stock data of the input company within a time period (6 monthes)
#### 3. time
We use time crate to get current time by ***OffsetDateTime*** and get six month by ***Duration***
#### 4. plotters
We use plotters to generate the stock data plot
#### 5. ta
We use ta to get the RSI and MACD data

## Financial analysis algorithm
Firstly, we retrieve the stock data for a specific period (6 months) by utilizing the get_quote_history method from the yahoo_finance_api. 
We then focus on the daily closing prices as the definitive price for each day and extract these values. 
These closing prices will later serve as the primary data for the chart. 
In addition, we identify the volatile_days with significant price fluctuations. To calculate the daily fluctuation percentage, we use the formula: (quote.high−quote.low)/quote.close. 
We then filter out the days when this percentage exceeds two percent to subsequently draw error lines for notable price movements.

## Charting setup
For the closing price chart, we determine the location for the image generation and set the base size for the drawing area. A white color is then filled in the drawing area to serve as the background for the chart. 
Next, by calculating the minimum and maximum values of all closing prices, we set the range for the Y-axis of the chart. 
Once we have all the data we need, we use ***ChartBuilder*** to construct the chart, setting the title, font, label area sizes for the X and Y axes, and the range of the axes. 
Then, using the closing prices as the base points, we draw a red line chart representing the data over six months, where each point on the image corresponds to the closing price for that day. 
For those volatile days, we use blue lines and dots to provide special annotations (error lines). Finally, we obtain the completed chart.

For the RSI and MACD chart, we determine the corresponding RSI and MACD values from the closing prices by using the ta crate. Then we simply repeat what we did for the previous chart
and change the x/y-axis scale and label, title and legend label to create the final chart

## Project setup

#### Step 1: Install Rust Environment
#### Step 2: Create a New Rust Project by ***Cargo new new_project***
#### Step 3: Add all the dependencies to ***Cargo.toml***
#### Step 4: Build the project and run

## Usage instructions
#### Step 1: Download the repo from GitHub
#### Step 2: Run the project by ***Cargo run*** and input stock code in the command line
#### Step 3: The generated plot will be in the project folder, open the image to see recent six monthes data