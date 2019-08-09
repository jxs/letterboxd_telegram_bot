# Letterboxd Search Bot
A Telegram inline bot to search Letterboxd films, available at [@lbdSBot](https://t.me/lbdSBot)

## Usage
To use it within Telegram, just type ```@lbdSBot <film>``` and the bot will show the list of films found.

## Build and run

- Clone the repository: ```git clone https://github.com/jxs/letterboxd_telegram_bot.git && cd letterboxd_telegram_bot```
- ```export TELEGRAM_BOT_TOKEN=<INSERT YOUR BOT TOKEN HERE>``` with your own bot token (you can get it from [@BotFather](https://t.me/BotFather)
- ```export LETTERBOXD_API_KEY=<INSERT API KEY HERE>``` with your own good reads api key (you can get it by sending an email to api@letterboxd.com)
- ```export LETTERBOXD_API_SECRET=<INSERT API KEY SECRET>``` with your own good reads api key (you can get it by sending an email to api@letterboxd.com)
- Finally, build the project and run it: ```cargo run```

_N.B. Remember to set the bot as an inline bot by issuing the command_ ```/setinline``` _to BotFather_
