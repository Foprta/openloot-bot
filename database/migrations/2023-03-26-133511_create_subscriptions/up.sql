CREATE TABLE subscriptions (
  chat_id VARCHAR NOT NULL,
  item_name VARCHAR NOT NULL,
  price DOUBLE NOT NULL,
  notificate INTEGER NOT NULL,
  CONSTRAINT id PRIMARY KEY(chat_id, item_name),
  FOREIGN KEY (item_name)
    REFERENCES items (option_name)
)