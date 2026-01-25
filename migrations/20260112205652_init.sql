CREATE TABLE IF NOT EXISTS budget (
		id     INTEGER PRIMARY KEY,
		amount REAL    NOT NULL,
		month  TEXT    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS payments  (
		id        INTEGER PRIMARY KEY,
		amount    REAL    NOT NULL,
		kind      TEXT    NOT NULL,
		budget_id INTEGER NOT NULL,
		day_of    TEXT    NOT NULL
				DEFAULT (datetime('now')),
		CONSTRAINT to_budget
				FOREIGN KEY (budget_id)
				REFERENCES budget (id)
				ON DELETE CASCADE

);


