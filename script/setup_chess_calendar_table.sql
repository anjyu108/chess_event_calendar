CREATE TABLE IF NOT EXISTS chess_event (
  event_id INT AUTO_INCREMENT NOT NULL PRIMARY KEY,
  name TEXT,
  start_date DATE,
  end_date DATE,
  open_time TEXT,
  revenue TEXT,
  fee TEXT
);
