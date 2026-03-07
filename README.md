# 🌌 Plutonic 🌌

Plutonic is an asynchronous algorithmic trading framework written in 🦀*Rust*🦀. Users can implement their own trading strategies and register it with Plutonic to automatically execute. Plutonic interfaces with [Alpaca](https://alpaca.markets/).

## Why Make Plutonic?

I have no desire to use this bot in an attempt to make money and only want to use it for paper trading. I see value in learning how to create a full stack application. I also have interests in Machine Learning and Artificial intelligence and I hope to one day use various models in Putonic. However, my main goal is to learn async Rust and design a fast initial framework to build upon.

## How to Run
Plutonic is written to be flexible. It provides the mechanisms to listen to a live feed of data, execute orders, and manage strategies concurrently. See `main.rs` for an example of how to structure your own bot. Plutonic does require that various environment variables be set. You **must** set `APCA_API_KEY_ID` and `APCA_API_SECRET_KEY` in order to establish a connection with Alpaca, otherwise plutonic will not be able to start. These values can be obtained from your Alpaca account. Optionally, you can set `APCA_API_BASE_URL` to change the Alpaca API endpoint. This allows you to use either paper trading or live trading. Plutonic should default to using paper trading, but it is recommended to explicitly set `APCA_API_BASE_URL` to the paper trading endpoint, `https://paper-api.alpaca.markets`.

# ⚠ Warning ⚠
This software is for educational purposes only. Do not risk money which you are afraid to lose. USE THE SOFTWARE AT YOUR OWN RISK. THE AUTHORS AND ALL AFFILIATES ASSUME NO RESPONSIBILITY FOR YOUR TRADING RESULTS.
