/* FIXME: normalize this table */
CREATE TABLE IF NOT EXISTS chess_event (
  name CHAR(100) NOT NULL,
  organizer CHAR(100) NOT NULL,
  start_date DATE NOT NULL,
  end_date DATE,
  open_time TEXT,
  revenue TEXT,
  fee TEXT,
  PRIMARY KEY (name, organizer, start_date)
);
