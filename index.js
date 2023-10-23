import dotenv from 'dotenv';
import TelegramBot from 'node-telegram-bot-api';
import axios from 'axios';
import util from 'util';

dotenv.config();

const sleep = util.promisify(setTimeout);

const bot = new TelegramBot(process.env.TELOXIDE_TOKEN_SCREENER, {polling: true});

const subscribers = new Set();
const sentNotificationPrices = new Map();

bot.onText(/^\/subscribe/, (msg) => {
  const chatId = msg.chat.id;

  subscribers.add(chatId);

  bot.sendMessage(chatId, 'Subscribed!');
});

bot.onText(/^\/stop/, () => {
  subscribers.forEach((chatId) => {
    bot.sendMessage(chatId, 'Stopped!');
  })

  subscribers.clear();
  sentNotificationPrices.clear();
});

let pageNumber = 1;

while (true) {
  if (subscribers.size === 0) {
    await sleep(1000);
    continue;
  }

  if (pageNumber > 10) {
    pageNumber = 1;
  }

  try {
    const response = await axios.get(`https://proxy.scrapeops.io/v1/?api_key=${process.env.PROXY_API_KEY}&url=https%3A%2F%2Fapi.openloot.com%2Fv2%2Fmarket%2Flistings%2FBT0_Hourglass_Common%2Fitems%3FonSale%3Dtrue%26page%3D${pageNumber}%26pageSize%3D50%26sort%3Dprice%253Aasc&country=us&device_type=desktop&premium=true`);

    for (const item of response.data.items) {
      const itemPrice = parseFloat(item.price);
      const minutesLeft = parseFloat(item.item.extra?.attributes?.[0]?.value);

      if (minutesLeft >= 10 && sentNotificationPrices.get(item.id) !== itemPrice) {
        subscribers.forEach((chatId) => {
          bot.sendMessage(chatId, `Minutes: ${minutesLeft}\nPrice: ${itemPrice}\nID: ${item.item.issuedId}`).catch(console.error);
        });

        if (subscribers.size > 0) {
          sentNotificationPrices.set(item.id, itemPrice);
        }
      }
    }


    pageNumber += 1;
  } catch (error) {
    console.error(error);
  }
}