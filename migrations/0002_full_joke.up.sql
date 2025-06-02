alter table quotes rename to quotes_v1;

create table quotes (
  id varchar(200) unique primary key not null,
  quote varchar(200) not null,
  author varchar(200) not null
);

CREATE TABLE IF NOT EXISTS tags (
  quote_id VARCHAR(200) NOT NULL,
  tag VARCHAR(200) NOT NULL,
  FOREIGN KEY (quote_id) REFERENCES quotes(id)
);