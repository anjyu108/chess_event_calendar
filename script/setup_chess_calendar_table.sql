CREATE TABLE IF NOT EXISTS chess_event (
  event_id INT AUTO_INCREMENT NOT NULL PRIMARY KEY,
  start_date TEXT,
  end_date TEXT,
  open_time TEXT,
  revenue TEXT,
  fee TEXT
);
