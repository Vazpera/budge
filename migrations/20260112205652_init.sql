CREATE TABLE IF NOT EXISTS budget (
		id     INTEGER PRIMARY KEY,
		amount INTEGER NOT NULL,
		month  TEXT    NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS payments  (
		id        INTEGER PRIMARY KEY,
		amount    INTEGER NOT NULL,
		kind      TEXT    NOT NULL,
		budget_id INTEGER NOT NULL,
		day_of    TEXT    NOT NULL,
		CONSTRAINT to_budget
				FOREIGN KEY (budget_id)
				REFERENCES budgets (id)

);


